use std::fmt::format;
use std::fs;
use std::process::Output;
use db_sniffer::generators;
use db_sniffer::generators::hibernate::XMLGenerator;
use crate::{containers, hibernate, logs, maven, test_dir};
use crate::containers::DBContainer;
use crate::logs::LogLevel;
use crate::maven::MavenProject;

fn move_config_and_mapping_files_to_resources(maven_project: &MavenProject) {
    let mapping_files_dir = maven_project.get_package_src_dir().join("model");
    fs::create_dir_all(&mapping_files_dir).unwrap();

    let resources_target_path = maven_project.get_package_resources_dir().join("model");
    fs::create_dir_all(&resources_target_path).unwrap();

    mapping_files_dir.read_dir().unwrap().for_each(|entry| {
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
}

pub async fn start_hibernate_test(conn_str: &str, db_dependency: maven::Dependencie, container: DBContainer) {
    let test_dir = test_dir::get();

    container.start();
    
    let test_result = aux(conn_str, db_dependency, &test_dir).await;
    
    container.stop();

    let output = match test_result {
        Ok(r) => r,
        Err(e) => {
            container.stop();
            panic!("failed to package and execute the Maven project -> {}", e)
        }
    };

    assert!(output.status.success());

    fs::remove_dir_all(test_dir).expect("Error removing the test dir");
    
    async fn aux(conn_str: &str, db_dependency: maven::Dependencie, test_dir: &str) -> Result<Output, String> {
        // Creating a Maven archetype project
        let mut maven_project = MavenProject::new(test_dir::get());

        maven_project.add_dependency(db_dependency);

        if let Err(e) = maven_project.create_archetype(maven::MAIN_CONTENT) {
            panic!("Failed to create Maven archetype project -> {}", e)
        }

        let target_path = maven_project.get_package_src_dir().join("model");
        fs::create_dir_all(&target_path)
            .map_err(|e| format!("Failed to create target path -> {}", e))?;

        let time = std::time::Instant::now();
        logs::log("Starting introspection", LogLevel::Info);
        
        let sniff_results = db_sniffer::sniff(conn_str)
            .await
            .map_err(|e| format!("Failed to sniff the database -> {}", e))?;

        let elapsed = time.elapsed();
        logs::log(&format!("Introspection took {:?}", elapsed), LogLevel::Info);
        
        let generator = XMLGenerator::new(&sniff_results, &target_path)
            .ok_or("Failed to create XMLGenerator")?;

        let time = std::time::Instant::now();
        logs::log("Starting generation", LogLevel::Info);
        
        generator.generate();

        let elapsed = time.elapsed();
        logs::log(&format!("Generation took {:?}", elapsed), LogLevel::Info);

        // Move the resources to the resources folder
        move_config_and_mapping_files_to_resources(&maven_project);

        // Using maven and junit to validate
        maven_project
            .package_and_execute()
            .map_err(|e| format!("Failed to package and execute the Maven project -> {}", e))
    }
}
