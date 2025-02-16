#![allow(unused)]

use std::collections::HashSet;
use db_sniffer::{SniffResults, Table};

mod containers;

#[tokio::test]
async fn sinffers_eq_results() {
    let mssql = containers::DBContainer::new_mssql();
    let conn_str = "mssql://SA:D3fault&Pass@localhost:8000/test_db";
    
    let mssql_result = aux(conn_str, mssql).await;
    
    let mysql = containers::DBContainer::new_mysql();
    let conn_str = "mysql://test_user:abc123.@localhost:8000/test_db";
    
    let mysql_result = aux(conn_str, mysql).await;

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