use crate::db_objects::{Column, ColumnId, ColumnType, Database, GenerationType, KeyType, ReferenceType, Table};
use crate::sniffers::{DatabaseSniffer, SniffResults};
use crate::ConnectionParams;
use crate::Error::MissingParamError;
use sqlx::mysql::MySqlRow;
use sqlx::{Connection, Executor, MySqlConnection, Row};
use std::ops::Deref;
use std::str::FromStr;
#[cfg(test)] use crate::test_utils::mysql::simple_existing_db_conn_params;

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
    async fn introspect_database(&mut self) -> Database {
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
            let column = self.introspect_column(column, table_name).await;
            table.add_column(column);
        }

        table
    }
    
    async fn introspect_column(&mut self, column: MySqlRow, table_name: &str) -> Column {
        let column_name: &str = column.get(0);
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
                column_name,
                field_type,
                field_nullable,
                field_key,
                field_default.unwrap_or_default(),
                field_extra
            );
        }

        let key = match field_key.deref() {
            "PRI" => match field_extra {
                "auto_increment" => KeyType::Primary(GenerationType::AutoIncrement),
                _ => KeyType::Primary(GenerationType::None),
            },
            "MUL" => KeyType::Foreign,
            "UNI" => KeyType::Unique,
            _ => KeyType::None,
        };

        let references = {
            let sql = "SELECT
                REFERENCED_TABLE_NAME,
                REFERENCED_COLUMN_NAME
            FROM
                INFORMATION_SCHEMA.KEY_COLUMN_USAGE
            WHERE
                TABLE_NAME = ? 
                AND COLUMN_NAME = ? 
                AND REFERENCED_TABLE_NAME IS NOT NULL;";

            let ref_column = sqlx::query(sql)
                .bind(table_name)
                .bind(column_name)
                .fetch_optional(&mut self.conn)
                .await
                .unwrap();

            match ref_column {
                Some(row) => {
                    let ref_table_name: &str = row.get(0);
                    let ref_column_name: &str = row.get(1);
                    
                    #[cfg(debug_assertions)]
                    {
                        println!("Column {} references {} ({})", column_name, ref_table_name, ref_column_name);
                    }

                    let ref_type = self.introspect_ref_type(
                        &ColumnId::new(table_name, column_name),
                        &ColumnId::new(ref_table_name, ref_column_name)
                    ).await;
                    
                    Some((ColumnId::new(ref_table_name, ref_column_name), ref_type))
                }
                None => None,
            }
        };
        
        let referenced_by = {
            let mut ref_by = Vec::new();
            
            let sql = "SELECT
                TABLE_NAME,
                COLUMN_NAME
            FROM
                information_schema.KEY_COLUMN_USAGE
            WHERE
                REFERENCED_TABLE_NAME = ?
              AND REFERENCED_COLUMN_NAME = ?;";
            
            let columns_ref_this = sqlx::query(sql)
                .bind(table_name)
                .bind(column_name)
                .fetch_all(&mut self.conn)
                .await
                .unwrap();
            
            for column_ref_this in columns_ref_this {
                let ref_table_name: &str = column_ref_this.get(0);
                let ref_column_name: &str = column_ref_this.get(1);
                
                #[cfg(debug_assertions)]
                {
                    println!("Column {} is referenced by {} ({})", column_name, ref_table_name, ref_column_name);
                }
                
                let ref_type = self.introspect_ref_type(
                    &ColumnId::new(ref_table_name, ref_column_name),
                    &ColumnId::new(table_name, column_name),
                ).await;
                
                ref_by.push((ColumnId::new(ref_table_name, ref_column_name), ref_type));
            }

            ref_by
        };
        
        Column::new(
            ColumnId::new(table_name, column_name),
            ColumnType::from_str(&field_type.to_string()).unwrap(),
            field_nullable,
            key,
            references,
            referenced_by
        )
    }
    
    async fn introspect_ref_type(&self, from_col: &ColumnId, to_col: &ColumnId) -> ReferenceType
    {
        ReferenceType::Unknown
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::mysql::simple_existing_db_conn_params;

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
