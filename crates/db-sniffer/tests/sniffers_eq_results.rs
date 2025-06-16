#![allow(unused)]

use std::collections::HashSet;
use db_sniffer::{SniffResults, Table};

mod containers;

#[tokio::test]
async fn sinffers_eq_results() {
    let containers = containers::DBContainer;
    containers.start();
    
    let conn_str = "mssql://SA:D3fault&Pass@localhost:50001/test_db";
    let mssql_result = db_sniffer::sniff(conn_str)
        .await
        .expect("Failed to sniff the database");
    
    let conn_str = "mysql://test_user:abc123.@localhost:50000/test_db";
    let mysql_result = db_sniffer::sniff(conn_str)
        .await
        .expect("Failed to sniff the database");

    containers.stop();
    
    let mssql_db = mssql_result.database();
    let mysql_db = mysql_result.database();
    
    assert_eq!(
        mssql_db.tables(),
        mysql_db.tables()
    );
    
    async fn aux(conn_str: &str, dbcontainer: containers::DBContainer) -> SniffResults
    {
        dbcontainer.start();

        let sniff_results = db_sniffer::sniff(conn_str)
            .await
            .expect("Failed to sniff the database");

        dbcontainer.stop();
        
        sniff_results
    }
}