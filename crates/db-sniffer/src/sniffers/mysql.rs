use crate::db_objects::{Column, ColumnType, Database, KeyType, Table};
use crate::sniffers::{DatabaseSniffer, SniffResults};
use crate::ConnectionParams;
use crate::Error::MissingParamError;
use sqlx::{Connection, Executor, MySqlConnection, Row};
use std::ops::Deref;
use std::str::FromStr;
use sqlx::mysql::MySqlRow;

pub struct MySQLSniffer {
    conn_params: ConnectionParams,
    conn: MySqlConnection,
}

impl DatabaseSniffer for MySQLSniffer {
    async fn new(params: ConnectionParams) -> Result<Self, crate::Error> {
        // TODO: Remove the clone
        let params_clone = params.clone();

        let user = params.user.ok_or(MissingParamError("user".to_string()))?;
        let password = params
            .password
            .ok_or(MissingParamError("password".to_string()))?;
        let host = params.host.ok_or(MissingParamError("host".to_string()))?;
        let port = params.port.ok_or(MissingParamError("port".to_string()))?;
        let dbname = params
            .dbname
            .ok_or(MissingParamError("dbname".to_string()))?;

        let connection = MySqlConnection::connect(&format!(
            "mysql://{}:{}@{}:{}/{}",
            user, password, host, port, dbname
        ))
        .await?;

        let sniffer = MySQLSniffer {
            conn_params: params_clone,
            conn: connection,
        };

        Ok(sniffer)
    }

    async fn sniff(mut self) -> SniffResults {
        let database = self.introspect_database().await;

        SniffResults {
            metadata: Default::default(),
            database,
            conn_params: self.conn_params,
        }
    }
}

impl MySQLSniffer {
    async fn introspect_database(&mut self) -> Database
    {
        let db_name = self.conn_params.dbname.as_ref().unwrap().as_str();
        
        let mut database = Database::new(db_name);
        
        let tables = sqlx::query("show tables")
            .fetch_all(&mut self.conn)
            .await
            .unwrap();

        for table in tables {
            database.add_table(self.introspect_table(table.get(0)).await);
        }
        
        database
    }
    
    async fn introspect_table(&mut self, table_name: &str) -> Table {
        let mut table = Table::new(table_name);
        
        let columns = sqlx::query(format!("describe {}", table_name).as_str())
            .fetch_all(&mut self.conn)
            .await
            .unwrap();

        #[cfg(debug_assertions)]
        {
            println!("table: {}", table_name);
        }

        for column in columns {
            table.add_column(self.introspect_column(column).await);
        }
        
        table
    }
    
    async fn introspect_column(&mut self, column: MySqlRow) -> Column {
        let field_name: &str = column.get(0);
        let field_type: &[u8] = column.get(1);
        let field_type = String::from_utf8_lossy(field_type).to_string();
        let field_type = field_type.split("(").next().unwrap();
        let field_nullable: &str = column.get(2);
        let field_nullable: bool = field_nullable == "YES";
        let field_key: &[u8] = column.get(3);
        let field_key = String::from_utf8_lossy(field_key);
        let field_default: Option<&str> = column.get(4);
        let field_extra: &str = column.get(5);

        #[cfg(debug_assertions)]
        {
            println!(
                "name: {:?}, type: {:?}, nullable: {:?}, key: {:?}, default: {:?}, extra: {:?}",
                field_name,
                field_type,
                field_nullable,
                field_key,
                field_default.unwrap_or_default(),
                field_extra
            );
        }
        
        let key = match field_key.deref() {
            "PRI" => KeyType::Primary,
            "MUL" => KeyType::Foreign,
            "UNI" => KeyType::Unique,
            _ => KeyType::None,
        };

        Column::new(
            field_name.to_string(),
            ColumnType::from_str(&field_type.to_string()).unwrap(),
            field_nullable,
            key,
        )
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::mysql::simple_existing_db_conn_params;
    use super::*;

    #[tokio::test]
    async fn test_mysql_sniffer() {
        let conn_str = "mysql://root:abc123.@10.0.2.4:3306";
        let conn_params = conn_str.parse().unwrap();

        assert!(MySQLSniffer::new(conn_params).await.is_err());

        let conn_params = simple_existing_db_conn_params();

        let sniffer = MySQLSniffer::new(conn_params).await;
        assert!(sniffer.is_ok());

        let mut sniffer = sniffer.unwrap();

        let results = sniffer.sniff().await;
    }
}
