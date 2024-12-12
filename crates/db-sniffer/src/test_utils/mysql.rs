use crate::db_objects::{Column, ColumnType, Database, KeyType, Table};
use crate::sniffers::SniffResults;
use crate::ConnectionParams;
use std::str::FromStr;

pub fn existing_db_conn_params() -> ConnectionParams {
    ConnectionParams::from_str("mysql://root:abc123.@10.0.2.4:3306/bdempresa").unwrap()
}

pub fn not_existing_db_conn_params() -> ConnectionParams {
    ConnectionParams::from_str("mysql://user:password@localhost:3306/test_db").unwrap()
}

pub fn trivial_sniff_results() -> SniffResults {
    let columns = vec![
        Column::new(
            "id".to_string(),
            ColumnType::Integer,
            false,
            KeyType::Primary,
        ),
        Column::new("name".to_string(), ColumnType::Text, false, KeyType::None),
    ];

    let mut table = Table::new("users");
    for column in columns {
        table.add_column(column);
    }

    let mut database = Database::new("test_db");

    database.add_table(table);

    let conn_params = not_existing_db_conn_params();

    SniffResults::new(None, database, conn_params)
}
