use std::path::PathBuf;
use std::time::Duration;
use std::{process, thread};

fn containers_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../containers")
}

const CONTAINER_NAME: &str = "integration_test_db_sniffer";

pub struct DBContainer {
    image: String,
    port: u16,
    build_file: String,
}

impl DBContainer {
    pub fn new_mysql() -> Self {
        DBContainer {
            image: "mysql:db-sniffer".to_string(),
            port: 3306,
            build_file: "mysql.dockerfile".to_string(),
        }
    }

    pub fn new_mssql() -> Self {
        DBContainer {
            image: "mssql:db-sniffer".to_string(),
            port: 1433,
            build_file: "mssql.dockerfile".to_string(),
        }
    }
}

impl DBContainer {
    pub fn start(&self) {
        // docker run --name <name> -p 8000:3306 <image>
        self.build();

        process::Command::new("docker")
            .args(&[
                "run",
                "--name",
                CONTAINER_NAME,
                "-p",
                format!("8000:{}", self.port).as_str(),
                &self.image,
            ])
            .spawn()
            .expect("Failed to start container for testing");

        thread::sleep(Duration::from_secs(15))
    }

    fn build(&self) {
        // docker build -t <image> -f <file> .
        process::Command::new("docker")
            .current_dir(containers_dir())
            .args(&["build", "-t", &self.image, "-f", &self.build_file, "."])
            .spawn()
            .expect("Failed to build container for testing")
            .wait()
            .expect("Failed to wait for container to be built");
    }

    pub fn stop(&self) {
        // docker stop <name>
        // docker rm <name>
        process::Command::new("docker")
            .args(&["stop", CONTAINER_NAME])
            .spawn()
            .expect("Failed to stop Docker container")
            .wait()
            .expect("Failed to wait for Docker container to stop");

        process::Command::new("docker")
            .args(&["rm", CONTAINER_NAME])
            .spawn()
            .expect("Failed to remove Docker container")
            .wait()
            .expect("Failed to wait for Docker container to be removed");
    }
}
