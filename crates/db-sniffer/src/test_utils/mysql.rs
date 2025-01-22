use std::{process, thread};
use std::time::Duration;

pub fn start_container(){
    // docker build -t mysql:db-sniffer -f "mysql.dockerfile" .
    // docker run --name mysql_db_sniffer -p 3306:3306 mysql:db-sniffer
    
    let mut docker_command = process::Command::new("docker");
    
    docker_command.args(&["run", "--name", "mysql_db_sniffer", "-p", "3306:3306", "mysql:db-sniffer"]);
    
    docker_command.spawn().expect("Failed to start MySQL container for testing");
    thread::sleep(Duration::from_secs(30))
}

pub fn stop_container() {
    // docker stop mysql_db_sniffer
    // docker rm mysql_db_sniffer
    
    let mut docker_command = process::Command::new("docker");
    
    docker_command.args(&["stop", "mysql_db_sniffer"]);
    let mut com = docker_command.spawn().expect("Failed to stop MySQL container");
    com.wait().expect("Failed to wait for MySQL container to stop");
    
    let mut docker_command = process::Command::new("docker");
    
    docker_command.args(&["rm", "mysql_db_sniffer"]);
    com = docker_command.spawn().expect("Failed to remove MySQL container");
    com.wait().expect("Failed to wait for MySQL container to be removed");
}
