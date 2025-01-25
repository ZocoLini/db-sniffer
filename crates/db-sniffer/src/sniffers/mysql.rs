use crate::db_objects::{ColumnId, GenerationType, KeyType};
use crate::error::Error::MissingParamError;
use crate::sniffers::{
    DatabaseSniffer, RowGetter,
};
use crate::ConnectionParams;
use sqlx::{Connection, Executor, MySqlConnection, Row};
use std::future::Future;
use std::pin::Pin;
pub(super) struct MySQLSniffer {
    conn_params: ConnectionParams,
    conn: MySqlConnection,
}

impl MySQLSniffer {
    pub async fn new(params: ConnectionParams) -> Result<Self, crate::Error> {
        let user = params
            .user
            .as_ref()
            .ok_or(MissingParamError("user".to_string()))?;
        let password = params
            .password
            .as_ref()
            .ok_or(MissingParamError("password".to_string()))?;
        let host = params
            .host
            .as_ref()
            .ok_or(MissingParamError("host".to_string()))?;
        let port = params
            .port
            .as_ref()
            .ok_or(MissingParamError("port".to_string()))?;
        let dbname = params
            .dbname
            .as_ref()
            .ok_or(MissingParamError("dbname".to_string()))?;

        let connection = MySqlConnection::connect(&format!(
            "mysql://{}:{}@{}:{}/{}",
            user, password, host, port, dbname
        ))
        .await?;

        let sniffer = MySQLSniffer {
            conn_params: params,
            conn: connection,
        };

        Ok(sniffer)
    }
}

// impl<'a> RowGet<'a> for sqlx::mysql::MySqlRow {
//     fn generic_get<T: sqlx::Decode<'a, MySql> + sqlx::Type<MySql>>(&'a self, idx: usize) -> T {
//         self.get(idx)
//     }
// }
// 
// impl<'a> DatabaseQuerier<'a, sqlx::mysql::MySqlRow> for MySQLSniffer {
//     fn query(
//         &mut self,
//         query: &str,
//     ) -> Pin<Box<dyn Future<Output = Vec<sqlx::mysql::MySqlRow>> + Send + '_>> {
//         let query = query.to_string();
// 
//         Box::pin(async move {
//             sqlx::query(&query)
//                 .fetch_all(&mut self.conn)
//                 .await
//                 .expect("Error fetching data")
//         })
//     }
// }

impl DatabaseSniffer for MySQLSniffer {
    fn query(
        &mut self,
        query: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<RowGetter>> + Send + '_>> {
        let query = query.to_string();

        Box::pin(async move {
            sqlx::query(&query)
                .fetch_all(&mut self.conn)
                .await
                .expect("Error fetching data")
                .into_iter()
                .map(|row| RowGetter::MySQlRow(row))
                .collect()
        })
    }
    
    fn query_dbs_names(&mut self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(async move {
            vec![self
                .conn_params
                .dbname
                .as_ref()
                .unwrap()
                .as_str()
                .to_string()]
        })
    }

    fn query_tab_names(&mut self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(async move {
            self.query("show tables")
                .await
                .iter()
                .map(|row| String::from_utf8_lossy(row.get(0)).to_string())
                .collect()
        })
    }

    fn query_col_names(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {
            self.query(format!("describe {}", table_name).as_str())
                .await
                .iter()
                .map(|row| row.get::<&str>(0).to_string())
                .collect()
        })
    }

    fn query_col_type(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = String> + Send + '_>> {
        let table_name = table_name.to_string();
        let column_name = column_name.to_string();

        Box::pin(async move {
            self.query(format!("describe {}", table_name).as_str())
                .await
                .iter()
                .filter_map(|row| {
                    if row.get::<&str>(0) == column_name {
                        Some(
                            String::from_utf8_lossy(row.get::<&[u8]>(1))
                                .split("(")
                                .next()
                                .unwrap()
                                .to_string(),
                        )
                    } else {
                        None
                    }
                })
                .collect()
        })
    }

    fn query_is_col_nullable(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        let table_name = table_name.to_string();
        let column_name = column_name.to_string();

        Box::pin(async move {
            self.query(format!("describe {}", table_name).as_str())
                .await
                .iter()
                .filter_map(|row| {
                    if row.get::<&str>(0) == column_name {
                        Some(row.get::<&str>(2).to_string())
                    } else {
                        None
                    }
                })
                .collect::<String>()
                == "YES"
        })
    }

    fn query_col_default(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>> {
        let table_name = table_name.to_string();
        let column_name = column_name.to_string();

        Box::pin(async move {
            self.query(format!("describe {}", table_name).as_str())
                .await
                .iter()
                .filter_map(|row| {
                    if row.get::<&str>(0) == column_name {
                        row.opt_get::<&str>(4)
                    } else {
                        None
                    }
                })
                .next()
                .map(|s| s.to_string())
        })
    }

    fn query_col_key(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = KeyType> + Send + '_>> {
        let table_name = table_name.to_string();
        let column_name = column_name.to_string();

        Box::pin(async move {
            let key: String = self
                .query(format!("describe {}", table_name).as_str())
                .await
                .iter()
                .filter_map(|row| {
                    if row.get::<&str>(0) == column_name {
                        Some(String::from_utf8_lossy(row.get::<&[u8]>(3)).to_string())
                    } else {
                        None
                    }
                })
                .collect();

            match key.as_str() {
                "PRI" => {
                    if self.query_is_col_auto_incr(&table_name, &column_name).await {
                        KeyType::Primary(GenerationType::AutoIncrement)
                    } else {
                        KeyType::Primary(GenerationType::None)
                    }
                }
                "MUL" => KeyType::Foreign,
                "UNI" => KeyType::Unique,
                _ => KeyType::None,
            }
        })
    }

    fn query_is_col_auto_incr(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        let table_name = table_name.to_string();
        let column_name = column_name.to_string();

        Box::pin(async move {
            self.query(format!("describe {}", table_name).as_str())
                .await
                .iter()
                .filter_map(|row| {
                    if row.get::<&str>(0) == column_name {
                        Some(row.get::<&str>(5).to_string())
                    } else {
                        None
                    }
                })
                .collect::<String>()
                == "auto_increment"
        })
    }

    fn query_table_references(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<(Vec<ColumnId>, Vec<ColumnId>)>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {
            let mut relations = Vec::new();

            let sql = &format!(
                "SELECT
                REFERENCED_TABLE_NAME,
                REFERENCED_COLUMN_NAME,
                COLUMN_NAME
            FROM
                INFORMATION_SCHEMA.KEY_COLUMN_USAGE
            WHERE
                TABLE_NAME = '{table_name}'
                AND REFERENCED_TABLE_NAME IS NOT NULL;"
            );

            let rows = self.query(sql).await;

            for row in rows {
                let ref_table_name: &str = &String::from_utf8_lossy(row.get(0));
                let ref_column_name: &str = row.get(1);
                let column_name: &str = row.get(2);

                let from = ColumnId::new(&table_name, column_name);
                let to = ColumnId::new(ref_table_name, ref_column_name);

                let from = vec![from];
                let to = vec![to];

                relations.push((from, to));
            }

            relations
        })
    }

    fn query_table_referenced_by(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<(Vec<ColumnId>, Vec<ColumnId>)>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {
            let mut relations = Vec::new();
            let ref_table_name = table_name;

            let sql = &format!(
                "SELECT
                TABLE_NAME,
                COLUMN_NAME,
                REFERENCED_COLUMN_NAME
            FROM
                information_schema.KEY_COLUMN_USAGE
            WHERE
                REFERENCED_TABLE_NAME = '{ref_table_name}'"
            );

            let rows = self.query(sql).await;

            for row in rows {
                let table_name: &str = &String::from_utf8_lossy(row.get(0));
                let column_name: &str = row.get(1);
                let ref_column_name: &str = row.get(2);

                let from = ColumnId::new(table_name, column_name);
                let to = ColumnId::new(&ref_table_name, ref_column_name);

                let from = vec![from];
                let to = vec![to];

                relations.push((from, to));
            }

            relations
        })
    }
}
