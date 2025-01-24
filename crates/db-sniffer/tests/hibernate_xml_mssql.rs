#![allow(unused)]

use db_sniffer::generators;
use std::{env, fs};

mod containers;
mod hibernate;
mod maven;
mod test_dir;

#[tokio::test]
async fn integration_test_xml() {
    let test_dir = test_dir::get();

    // Creating a Maven archetype project
    let mut maven_project = maven::MavenProject::new(&test_dir);

    maven_project.add_dependency(maven::Dependencie::new(
        "com.microsoft.sqlserver",
        "mssql-jdbc",
        "12.6.4.jre11",
    ));

    if let Err(e) = maven_project.create_archetype(maven::MAIN_CONTENT) {
        panic!("Failed to create Maven archetype project: {}", e)
    }

    containers::mssql::start_container();

    let sniff_results =
        if let Ok(r) = db_sniffer::sniff("mssql://SA:D3fault&Pass@localhost:8000/test_db").await {
            r
        } else {
            containers::mssql::stop_container();
            panic!("Failed to sniff the database")
        };

    let target_path = maven_project.get_package_src_dir().join("model");
    fs::create_dir_all(&target_path).unwrap();

    let generator =
        if let Some(r) = generators::hibernate::XMLGenerator::new(&sniff_results, &target_path) {
            r
        } else {
            containers::mysql::stop_container();
            panic!("Failed to create XMLGenerator")
        };

    generator.generate();

    // Move the resources to the resources folder
    hibernate::move_config_and_mapping_files_to_resources(&maven_project);

    // Using maven and junit to validate
    let output = match maven_project.package_and_execute() {
        Ok(r) => r,
        Err(e) => {
            containers::mysql::stop_container();
            panic!("Failed to package and execute the Maven project: {}", e)
        }
    };

    containers::mssql::stop_container();

    assert!(output.status.success());

    fs::remove_dir_all(test_dir).expect("Error removing the test dir");
}
