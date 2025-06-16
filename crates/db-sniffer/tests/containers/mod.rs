#![allow(clippy::zombie_processes)]

use std::path::PathBuf;
use std::time::Duration;
use std::{process, thread};
use std::env::args;

pub struct DBContainer;

impl DBContainer {
    pub fn new_mysql() -> Self {
        DBContainer
    }

    pub fn new_mssql() -> Self {
        DBContainer
    }
}

impl DBContainer {
    pub fn start(&self) {
        self.stop();
        self.build();

        process::Command::new("docker-compose")
            .args(["up", "-d"])
            .spawn()
            .expect("Failed to start container for testing");

        thread::sleep(Duration::from_secs(15))
    }

    fn build(&self) {
        process::Command::new("docker-compose")
            .args(["build"])
            .spawn()
            .expect("Failed to build container for testing")
            .wait()
            .expect("Failed to wait for container to be built");
    }

    pub fn stop(&self) {
        process::Command::new("docker-compose")
            .args(["down", "-v"])
            .spawn()
            .expect("Failed to stop Docker container")
            .wait()
            .expect("Failed to wait for Docker container to stop");
    }
}
