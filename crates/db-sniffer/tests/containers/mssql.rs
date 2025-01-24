use std::time::Duration;
use std::{process, thread};

pub fn start_container() {
    // docker run --name mssql_db_sniffer -p 3306:3306 mssql:db-sniffer

    build_container();

    process::Command::new("docker")
        .args(&[
            "run",
            "--name",
            "mssql_db_sniffer",
            "-p",
            "8000:1433",
            "mssql:db-sniffer",
        ])
        .spawn()
        .expect("Failed to start MySQL container for testing");
    thread::sleep(Duration::from_secs(15))
}

fn build_container() {
    // docker build -t mssql:db-sniffer -f "mssql.dockerfile" .
    process::Command::new("docker")
        .current_dir(super::containers_dir())
        .args(&[
            "build",
            "-t",
            "mssql:db-sniffer",
            "-f",
            "mssql.dockerfile",
            ".",
        ])
        .spawn()
        .expect("Failed to build MySQL container for testing")
        .wait()
        .expect("Failed to wait for MySQL container to be built");
}

pub fn stop_container() {
    // docker stop mssql_db_sniffer
    // docker rm mssql_db_sniffer

    process::Command::new("docker")
        .args(&["stop", "mssql_db_sniffer"])
        .spawn()
        .expect("Failed to stop MySQL container")
        .wait()
        .expect("Failed to wait for MySQL container to stop");

    process::Command::new("docker")
        .args(&["rm", "mssql_db_sniffer"])
        .spawn()
        .expect("Failed to remove MySQL container")
        .wait()
        .expect("Failed to wait for MySQL container to be removed");
}