use std::fs;
use crate::maven::MavenProject;

pub fn move_config_and_mapping_files_to_resources(maven_project: &MavenProject) {
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