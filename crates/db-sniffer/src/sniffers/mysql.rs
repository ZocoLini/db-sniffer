use crate::db_objects::{ColumnId, GenerationType, KeyType};
use crate::error::Error::MissingParamError;
use crate::sniffers::{DatabaseSniffer, RowGetter};
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
    fn close_conn(self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move { 
            if let Err(e) = self.conn.close().await {
                eprintln!("Error closing connection: {}", e);
            }
        })
    }

    fn query(&mut self, query: &str) -> Pin<Box<dyn Future<Output = Vec<RowGetter>> + Send + '_>> {
        let query = query.to_string();

        Box::pin(async move {
            sqlx::query(&query)
                .fetch_all(&mut self.conn)
                .await
                .expect("Error fetching data")
                .into_iter()
                .map(RowGetter::MySQlRow)
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

    // TODO: Refactor if possible this method logic.
    //  super should do the logic and the sniffers retrive the necessary data for this logic.
    //  Check that this method its almost identical for all sniffers
    fn query_table_references(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<(Vec<ColumnId>, Vec<ColumnId>)>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {
            let sql = &format!(
                "SELECT
                REFERENCED_TABLE_NAME,
                REFERENCED_COLUMN_NAME,
                COLUMN_NAME,
                CONSTRAINT_NAME
            FROM
                INFORMATION_SCHEMA.KEY_COLUMN_USAGE
            WHERE
                TABLE_NAME = '{table_name}'
                AND REFERENCED_TABLE_NAME IS NOT NULL
            ORDER BY CONSTRAINT_NAME;"
            );

            let mut relations = Vec::new();

            let mut last_constraint_name = None;
            let mut from = Vec::new();
            let mut to = Vec::new();

            for row in self.query(sql).await.iter() {
                let ref_table_name: &str = &String::from_utf8_lossy(row.get(0));
                let ref_column_name: &str = row.get(1);
                let column_name: &str = row.get(2);
                let constraint_name: &[u8] = row.get::<&[u8]>(3);

                if last_constraint_name.is_some()
                    && last_constraint_name.unwrap() != constraint_name
                {
                    relations.push((from, to));
                    from = Vec::new();
                    to = Vec::new();
                }

                from.push(ColumnId::new(&table_name, column_name));
                to.push(ColumnId::new(ref_table_name, ref_column_name));

                // TODO:
                //  Becasuse this is an option an doesn't have a default value,
                //  we cant't place this line inside the if,
                //  were it looks preattier.
                last_constraint_name.replace(constraint_name);
            }

            if !from.is_empty() {
                relations.push((from, to));
            }

            relations
        })
    }

    // TODO: This method shouldn't be necesary.
    //  We could iterate over the other tables when requesting
    //  the data this method instrospects
    //  and return the relations where the table is in the to part.
    fn query_table_referenced_by(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<(Vec<ColumnId>, Vec<ColumnId>)>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {
            let ref_table_name = table_name;

            let sql = &format!(
                "SELECT
                TABLE_NAME,
                COLUMN_NAME,
                REFERENCED_COLUMN_NAME,
                CONSTRAINT_NAME
            FROM
                information_schema.KEY_COLUMN_USAGE
            WHERE
                REFERENCED_TABLE_NAME = '{ref_table_name}'
            ORDER BY CONSTRAINT_NAME;"
            );

            let mut relations = Vec::new();

            let mut last_constraint_name = None;
            let mut from = Vec::new();
            let mut to = Vec::new();

            for row in self.query(sql).await.iter() {
                let table_name: &str = &String::from_utf8_lossy(row.get(0));
                let column_name: &str = row.get(1);
                let ref_column_name: &str = row.get(2);
                let constraint_name: &[u8] = row.get::<&[u8]>(3);

                if last_constraint_name.is_some()
                    && last_constraint_name.unwrap() != constraint_name
                {
                    relations.push((from, to));
                    from = Vec::new();
                    to = Vec::new();
                }

                from.push(ColumnId::new(table_name, column_name));
                to.push(ColumnId::new(&ref_table_name, ref_column_name));

                last_constraint_name.replace(constraint_name);
            }

            if !from.is_empty() {
                relations.push((from, to));
            }

            relations
        })
    }
}
