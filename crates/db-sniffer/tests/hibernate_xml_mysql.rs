mod containers;
mod maven;

use db_sniffer::generators::hibernate;
use std::{env, fs};

#[tokio::test]
async fn integration_test_xml() {
    dotenvy::dotenv().ok();

    let test_dir = if let Ok(r) = env::var("TEST_DIR") {
        r
    } else {
        panic!("TEST_DIR env var not found")
    };

    if fs::exists(&test_dir).unwrap_or(false) {
        fs::remove_dir_all(&test_dir).expect("Should empty the test dir");
    }

    // Creating a Maven archetype project
    let mut maven_project = maven::MavenProject::new(&test_dir);

    maven_project.add_dependency(maven::Dependencie::new(
        "junit",
        "junit",
        "4.13.2")
    );
    maven_project.add_dependency(maven::Dependencie::new(
        "mysql",
        "mysql-connector-java",
        "8.0.33",
    ));
    maven_project.add_dependency(maven::Dependencie::new(
        "org.hibernate",
        "hibernate-core",
        "4.3.11.Final",
    ));

    if let Err(e) = maven_project.create_archetype(MAIN_CONTENT) {
        panic!("Failed to create Maven archetype project: {}", e)
    }

    containers::mysql::start_container();

    let sniff_results = if let Ok(r) =
        db_sniffer::sniff("mysql://test_user:abc123.@localhost:3306/test_db").await
    {
        r
    } else {
        containers::mysql::stop_container();
        panic!("Failed to sniff the database")
    };

    let target_path = maven_project.get_package_src_dir().join("model");
    fs::create_dir_all(&target_path).unwrap();
    
    let generator = if let Some(r) = hibernate::XMLGenerator::new(&sniff_results, &target_path) {
        r
    } else {
        containers::mysql::stop_container();
        panic!("Failed to create XMLGenerator")
    };

    generator.generate();

    // Move the resources to the resources folder
    let resources_target_path = maven_project.get_package_resources_dir().join("model");
    fs::create_dir_all(&resources_target_path).unwrap();

    target_path.read_dir().unwrap().for_each(|entry| {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let file_name = file_name.to_str().unwrap();

        if !file_name.ends_with(".hbm.xml") {
            return;
        }

        let target = resources_target_path.join(file_name);

        fs::rename(entry.path(), target).unwrap();
    });

    maven_project
        .get_source_dir()
        .read_dir()
        .unwrap()
        .for_each(|entry| {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap();

            if !file_name.ends_with(".cfg.xml") {
                return;
            }

            let target = maven_project.get_resources_dir().join(file_name);

            fs::rename(entry.path(), target).unwrap();
        });

    // Using maven and junit to validate
    let output = match maven_project.package_and_execute() {
        Ok(r) => r,
        Err(e) => {
            containers::mysql::stop_container();
            panic!("Failed to package and execute the Maven project: {}", e)
        }
    };

    containers::mysql::stop_container();

    assert!(output.status.success());
}

const MAIN_CONTENT: &str = r#"package com.example;

import com.example.model.Person;
import com.example.model.PersonProject;
import com.example.model.PersonProjectId;
import org.hibernate.Session;
import org.hibernate.SessionFactory;
import org.hibernate.cfg.Configuration;

import static org.junit.Assert.assertEquals;

public class Main
{
    public static void main(String[] args)
    {
        Configuration configuration = new Configuration();
        configuration.configure();

        SessionFactory sessionFactory = null;
        Session session = null;

        try
        {
            sessionFactory = configuration.buildSessionFactory();
            session = sessionFactory.openSession();
            Person person = (Person) session.get(Person.class, 1);

            assertEquals("John Smith", person.getName());
            assertEquals("Engineering", person.getDepartment().getName());

            System.out.println("Person found and asserted");

            final var id = new PersonProjectId();

            id.setPersonId(1);
            id.setProjectId(1);

            PersonProject personProject = (PersonProject) session.get(PersonProject.class, id);

            assertEquals("John Smith", personProject.getPerson().getName());
            assertEquals("Website Redesign", personProject.getProject().getName());

            System.out.println("PersonProject found and asserted");
        }
        catch (Exception exception)
        {
            exception.printStackTrace();
        } finally
        {
            session.close();
            sessionFactory.close();
        }
    }
}"#;
