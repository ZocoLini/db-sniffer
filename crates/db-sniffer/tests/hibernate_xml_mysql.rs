#![allow(unused)]

mod containers;
mod hibernate;
mod maven;
mod test_dir;

use db_sniffer::generators;
use std::fs;
use std::process::Output;

#[tokio::test]
async fn integration_test_xml() {
    let container = containers::DBContainer::new_mysql();

    hibernate::start_hibernate_test(
        "mysql://test_user:abc123.@localhost:8000/test_db",
        maven::Dependencie::new("mysql", "mysql-connector-java", "8.0.33"),
        container,
    )
    .await;
}
