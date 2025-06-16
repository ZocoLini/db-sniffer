#![allow(unused)]

use db_sniffer::generators;
use std::fs;
use std::process::Output;

mod containers;
mod hibernate;
mod maven;
mod test_dir;
mod logs;

#[tokio::test]
async fn hibernate_xml_mssql() {
    let container = containers::DBContainer::new_mssql();

    hibernate::start_hibernate_test(
        "mssql://SA:D3fault&Pass@localhost:50001/test_db",
        maven::Dependency::new("com.microsoft.sqlserver", "mssql-jdbc", "12.6.4.jre11"),
        container,
    )
    .await;
}
