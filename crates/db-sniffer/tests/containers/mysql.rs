use std::time::Duration;
use std::{process, thread};

pub fn start_container() {
    // docker run --name mysql_db_sniffer -p 3306:3306 mysql:db-sniffer

    build_container();

    process::Command::new("docker")
        .args(&[
            "run",
            "--name",
            "mysql_db_sniffer",
            "-p",
            "3306:3306",
            "mysql:db-sniffer",
        ])
        .spawn()
        .expect("Failed to start MySQL container for testing");
    thread::sleep(Duration::from_secs(15))
}

fn build_container() {
    // docker build -t mysql:db-sniffer -f "mysql.dockerfile" .
    process::Command::new("docker")
        .current_dir(super::containers_dir())
        .args(&[
            "build",
            "-t",
            "mysql:db-sniffer",
            "-f",
            "mysql.dockerfile",
            ".",
        ])
        .spawn()
        .expect("Failed to build MySQL container for testing")
        .wait()
        .expect("Failed to wait for MySQL container to be built");
}

pub fn stop_container() {
    // docker stop mysql_db_sniffer
    // docker rm mysql_db_sniffer

    process::Command::new("docker")
        .args(&["stop", "mysql_db_sniffer"])
        .spawn()
        .expect("Failed to stop MySQL container")
        .wait()
        .expect("Failed to wait for MySQL container to stop");

    process::Command::new("docker")
        .args(&["rm", "mysql_db_sniffer"])
        .spawn()
        .expect("Failed to remove MySQL container")
        .wait()
        .expect("Failed to wait for MySQL container to be removed");
}
