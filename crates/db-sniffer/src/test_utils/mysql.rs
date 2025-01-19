use std::process;

pub fn start_container(){
    // docker build -t mysql:simple -f "test_resources/simple_hibernate_xml_and_pojos/mysql.dockerfile" .
    // docker run --name mysql -p 3306:3306 mysql:simple
    
    let mut docker_command = process::Command::new("docker");
    
    docker_command.args(&["run", "--name", "mysql", "-p", "3306:3306", "mysql:simple"]);
    
    docker_command.spawn().expect("Failed to start MySQL container for testing");
}

pub fn stop_container() {
    // docker stop mysql
    // docker rm mysql
    
    let mut docker_command = process::Command::new("docker");
    
    docker_command.args(&["stop", "mysql"]);
    docker_command.spawn().expect("Failed to stop MySQL container");
    
    let mut docker_command = process::Command::new("docker");
    
    docker_command.args(&["rm", "mysql"]);
    docker_command.spawn().expect("Failed to remove MySQL container");
}
