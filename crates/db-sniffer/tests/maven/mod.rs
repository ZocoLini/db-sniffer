use std::path::{Path, PathBuf};
use std::process;
use std::process::Output;

pub struct Dependencie {
    group_id: String,
    artifact_id: String,
    version: String,
}

impl Dependencie {
    pub fn new(group_id: &str, artifact_id: &str, version: &str) -> Self {
        Self {
            group_id: group_id.to_string(),
            artifact_id: artifact_id.to_string(),
            version: version.to_string(),
        }
    }

    pub fn as_string(&self) -> String {
        format!(
            r#"
            <dependency>
                <groupId>{group_id}</groupId>
                <artifactId>{artifact_id}</artifactId>
                <version>{version}</version>
            </dependency>"#,
            group_id = self.group_id,
            artifact_id = self.artifact_id,
            version = self.version
        )
    }
}

pub struct MavenProject {
    dir: PathBuf,
    dependencies: Vec<Dependencie>,
}

impl MavenProject {
    pub fn new<T: AsRef<Path>>(dir: T) -> Self {

        let mut dependencies = vec![];
        
        dependencies.push(Dependencie::new(
            "junit",
            "junit",
            "4.13.2")
        );
        dependencies.push(Dependencie::new(
            "org.hibernate",
            "hibernate-core",
            "4.3.11.Final",
        ));
        
        Self {
            dir: PathBuf::from(dir.as_ref()),
            dependencies,
        }
    }

    pub fn add_dependency(&mut self, dep: Dependencie) {
        self.dependencies.push(dep);
    }
}

impl MavenProject {
    pub fn create_archetype(&self, main_content: &str) -> Result<(), &str> {
        std::fs::create_dir_all(self.get_package_src_dir())
            .map_err(|_| "Failed to create package source directory")?;
        std::fs::create_dir_all(self.get_package_resources_dir())
            .map_err(|_| "Failed to create package resources directory")?;
        std::fs::create_dir_all(self.get_resources_dir().join("META-INF"))
            .map_err(|_| "Failed to create META-INF directory")?;

        let dependencies_string = self
            .dependencies
            .iter()
            .map(|dep| dep.as_string())
            .collect::<Vec<String>>()
            .join("\n");

        let pom_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8" ?>

<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>

    <groupId>com.example</groupId>
    <artifactId>maven-project</artifactId>
    <version>0.0.0</version>

    <properties>
        <maven.compiler.source>11</maven.compiler.source>
        <maven.compiler.target>11</maven.compiler.target>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
    </properties>

    <dependencies>
        {dependencies_string}
    </dependencies>

    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-compiler-plugin</artifactId>
                <version>3.11.0</version>
                <configuration>
                    <source>11</source>
                    <target>11</target>
                </configuration>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-jar-plugin</artifactId>
                <version>3.2.0</version>
                <configuration>
                    <outputDirectory>${{project.basedir}}</outputDirectory>
                    <finalName>${{project.artifactId}}</finalName>
                    <archive>
                        <manifest>
                            <addClasspath>true</addClasspath>
                            <mainClass>com.example.Main</mainClass>
                        </manifest>
                    </archive>
                </configuration>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-shade-plugin</artifactId>
                <version>3.2.4</version>
                <executions>
                    <execution>
                        <phase>package</phase>
                        <goals>
                            <goal>shade</goal>
                        </goals>
                        <configuration>
                            <createDependencyReducedPom>false</createDependencyReducedPom>
                            <filters>
                                <filter>
                                    <artifact>*:*</artifact>
                                    <excludes>
                                        <exclude>module-info.class</exclude>
                                        <exclude>META-INF/*.SF</exclude>
                                        <exclude>META-INF/*.DSA</exclude>
                                        <exclude>META-INF/*.RSA</exclude>
                                    </excludes>
                                </filter>
                            </filters>
                        </configuration>
                    </execution>
                </executions>
            </plugin>
            <plugin>
                <groupId>org.openjfx</groupId>
                <artifactId>javafx-maven-plugin</artifactId>
                <version>0.0.8</version>
                <configuration>
                    <mainClass>
                        com.example.Main
                    </mainClass>
                </configuration>
            </plugin>
        </plugins>
    </build>
</project>"#
        );

        std::fs::write(self.dir.join("pom.xml"), pom_content).expect("Failed to write to pom.xml");

        std::fs::write(self.get_package_src_dir().join("Main.java"), main_content)
            .map_err(|_| "Failed to write to Main.java")?;

        std::fs::write(
            self.get_resources_dir().join("META-INF/MANIFEST.MF"),
            "Manifest-Version: 1.0\nMain-Class: com.example.Main\n",
        )
        .map_err(|_| "Failed to write to MANIFEST.MF")
    }

    pub fn get_source_dir(&self) -> PathBuf {
        self.dir.join("src/main/java")
    }

    pub fn get_package_src_dir(&self) -> PathBuf {
        self.get_source_dir().join("com/example")
    }

    pub fn get_resources_dir(&self) -> PathBuf {
        self.dir.join("src/main/resources")
    }

    pub fn get_package_resources_dir(&self) -> PathBuf {
        self.get_resources_dir().join("com/example")
    }

    pub fn package_and_execute(&self) -> Result<Output, &str> {
        let output = process::Command::new("mvn")
            .arg("clean")
            .arg("package")
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .current_dir(self.dir.clone())
            .output()
            .map_err(|_| "Failed to run mvn package")?;

        if !output.status.success() {
            return Err("Failed to run mvn package");
        }

        process::Command::new("java")
            .arg("-jar")
            .arg("maven-project.jar")
            .stdout(process::Stdio::inherit())
            .stderr(process::Stdio::inherit())
            .current_dir(self.dir.clone())
            .output()
            .map_err(|_| "Failed to run java -jar")
    }
}

pub const MAIN_CONTENT: &str = r#"package com.example;

import com.example.model.*;
import org.hibernate.Session;
import org.hibernate.SessionFactory;
import org.hibernate.cfg.Configuration;

import java.util.List;

import static org.junit.Assert.*;

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

            // This method will query the database using hibernate to verify the correctness of the mapping
            session = sessionFactory.openSession();
            assertQueries(session);
            session.close();

            // This method will update the database using hibernate to verify the correctness of the mapping
            session = sessionFactory.openSession();
            updates(session);
            session.close();

            // This method will query the database using hibernate to verify the correctness of the updates
            session = sessionFactory.openSession();
            verifyUpdates(session);

        }
        catch (Exception exception)
        {
            exception.printStackTrace();
            fail("No exception should be thrown");
        }
        finally
        {
            if (session != null) session.close();
            sessionFactory.close();
        }
    }

    private static void assertQueries(Session session)
    {
        Person person = (Person) session.get(Person.class, 1);

        assertEquals("John Smith", person.getName());
        assertEquals("Engineering", person.getDepartment().getName());
        assertEquals(2, person.getPersonProjects().size());

        final var id = new PersonProjectId();

        id.setPersonId(1);
        id.setProjectId(1);

        PersonProject personProject = (PersonProject) session.get(PersonProject.class, id);

        assertEquals("John Smith", personProject.getPerson().getName());
        assertEquals("Website Redesign", personProject.getProject().getName());

        Department department = (Department) session.get(Department.class, 2);
        assertEquals("Marketing", department.getName());
        assertEquals(2, department.getPersons().size());

        List<Project> project =
                (List<Project>) session.createQuery("from Project p where p.personProjects.size = 2").list();
        assertEquals(3, project.size());

        System.out.println("All queries are correct");
    }

    private static void updates(Session session)
    {
        session.beginTransaction();

        Person person = (Person) session.get(Person.class, 1);
        person.setName("Test");
        person.getAddress().setCity("Test");

        List<Project> projects =
                (List<Project>) session.createQuery("from Project p where p.personProjects.size = 2").list();
        projects.forEach(p -> p.setName("Test"));

        projects =
                (List<Project>) session
                        .createQuery("from Project p where p.id not in " +
    "(select pp.project.id from PersonProject pp where pp.person.id = :personId)")
                        .setInteger("personId", person.getId())
                        .list();

        projects.forEach(p -> {
            PersonProject personProject = new PersonProject();
            personProject.setPerson(person);
            personProject.setProject(p);
            
            PersonProjectId id = new PersonProjectId();
            id.setPersonId(person.getId());
            id.setProjectId(p.getId());
            
            personProject.setId(id);
            
            session.save(personProject);
        });

        session.getTransaction().commit();

        System.out.println("All updates are correct");
    }

    private static void verifyUpdates(Session session)
    {
        Person person = (Person) session.get(Person.class, 1);

        assertEquals("Test", person.getName());
        assertEquals("Test", person.getAddress().getCity());
        
        List<Project> projects =
                (List<Project>) session
                        .createQuery("from Project p where p.id not in " +
    "(select pp.project.id from PersonProject pp where pp.person.id = :personId)")
                        .setInteger("personId", person.getId())
                        .list();
        
        assertEquals(0, projects.size());
        
        System.out.println("All updates checks are correct");
    }
}"#;
