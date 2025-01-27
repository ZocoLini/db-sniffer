use crate::db_objects::{ColumnId, Dbms, GenerationType, KeyType, Metadata};
use crate::sniffers::{DatabaseSniffer, RowGetter};
use crate::ConnectionParams;
use sqlx::Row;
use std::future::Future;
use std::pin::Pin;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt, TokioAsyncWriteCompatExt};

pub(super) struct MSSQLSniffer {
    conn_params: ConnectionParams,
    client: Client<Compat<TcpStream>>,
}

impl MSSQLSniffer {
    pub async fn new(params: ConnectionParams) -> Result<Self, crate::Error> {
        let user = params
            .user
            .as_ref()
            .ok_or(crate::Error::MissingParamError("user".to_string()))?;
        let password = params
            .password
            .as_ref()
            .ok_or(crate::Error::MissingParamError("password".to_string()))?;
        let host = params
            .host
            .as_ref()
            .ok_or(crate::Error::MissingParamError("host".to_string()))?;
        let port = params
            .port
            .as_ref()
            .ok_or(crate::Error::MissingParamError("port".to_string()))?;
        let dbname = params
            .dbname
            .as_ref()
            .ok_or(crate::Error::MissingParamError("dbname".to_string()))?;

        let mut config = Config::new();

        config.host(host);
        config.port(*port);
        config.trust_cert();
        config.authentication(AuthMethod::sql_server(user, password));
        config.database(dbname);

        config.trust_cert();

        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(|e| crate::Error::DBConnectionError(e.to_string()))?;
        tcp.set_nodelay(true)
            .map_err(|e| crate::Error::DBConnectionError(e.to_string()))?;

        // To be able to use Tokio's tcp, we're using the `compat_write` from
        // the `TokioAsyncWriteCompatExt` to get a stream compatible with the
        // traits from the `futures` crate.
        let mut client = Client::connect(config, tcp.compat_write())
            .await
            .map_err(|e| crate::Error::DBConnectionError(e.to_string()))?;

        let sniffer = MSSQLSniffer {
            conn_params: params,
            client,
        };

        Ok(sniffer)
    }
}

// impl<'a> RowGet<'a> for tiberius::Row {
//     fn generic_get<R: FromSql<'a>>(&'a self, index: usize) -> R {
//         self.get(index).expect("Error fetching data")
//     }
// }
//
// impl<'a> DatabaseQuerier<'a, tiberius::Row> for MSSQLSniffer {
//     fn query(
//         &mut self,
//         query: &str,
//     ) -> Pin<Box<dyn Future<Output = Vec<tiberius::Row>> + Send + '_>> {
//         let query = query.to_string();
//
//         Box::pin(async move {
//             self.client
//                 .query(query.as_str(), &[])
//                 .await
//                 .expect("Error fetching data")
//                 .into_first_result()
//                 .await
//                 .expect("Error fetching data")
//         })
//     }
// }

impl DatabaseSniffer for MSSQLSniffer {
    fn close_conn(mut self) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move{
            if let Err(e) = self.client.close().await {
                println!("Error closing db connection: {}", e)
            }
        })
    }

    fn query(&mut self, query: &str) -> Pin<Box<dyn Future<Output = Vec<RowGetter>> + Send + '_>> {
        let query = query.to_string();

        Box::pin(async move {
            self.client
                .query(query.as_str(), &[])
                .await
                .expect("Error fetching data")
                .into_first_result()
                .await
                .expect("Error fetching data")
                .into_iter()
                .map(RowGetter::MSSQLRow)
                .collect()
        })
    }

    fn query_metadata(&mut self) -> Pin<Box<dyn Future<Output = Option<Metadata>> + Send + '_>> {
        Box::pin(async move {
            Some(
                Metadata::new(Dbms::Mssql)
            )
        })
    }

    fn query_dbs_names(&mut self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(async move {
            let db_name = self.conn_params.dbname.as_ref().unwrap().as_str();

            vec![db_name.to_string()]
        })
    }

    fn query_tab_names(&mut self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(async move {
            self.query(
                r#"
                    select TABLE_NAME 
                    from INFORMATION_SCHEMA.TABLES  
                    where TABLE_TYPE = 'BASE TABLE';"#,
            )
            .await
            .iter()
            .map(|row| row.get::<&str>(0).to_string())
            .collect()
        })
    }

    fn query_col_names(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {
            self.query(&format!(
                "SELECT COLUMN_NAME
            FROM 
                INFORMATION_SCHEMA.COLUMNS
            WHERE 
                TABLE_NAME = '{table_name}';"
            ))
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
            self.query(&format!(
                "SELECT DATA_TYPE
            FROM 
                INFORMATION_SCHEMA.COLUMNS
            WHERE 
                TABLE_NAME = '{table_name}' AND COLUMN_NAME = '{column_name}'"
            ))
            .await
            .iter()
            .map(|row| row.get::<&str>(0).to_string())
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
            self.query(&format!(
                "SELECT IS_NULLABLE
            FROM 
                INFORMATION_SCHEMA.COLUMNS
            WHERE 
                TABLE_NAME = '{table_name}' AND COLUMN_NAME = '{column_name}'",
            ))
            .await
            .iter()
            .map(|row| row.get::<&str>(0))
            .collect::<String>()
                == *"YES"
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
            self.query(&format!(
                "SELECT COLUMN_DEFAULT
                        FROM 
                            INFORMATION_SCHEMA.COLUMNS
                        WHERE 
                            TABLE_NAME = '{table_name}' AND COLUMN_NAME = '{column_name}'"
            ))
            .await.first()?
            .opt_get::<&str>(0).map(|s| s.to_string())
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
            let field_key = self
                .query(&format!(
                    "WITH KeyColumns AS (
                    SELECT
                        t.name AS table_name,
                        c.name AS column_name,
                        CASE
                            WHEN pk.name IS NOT NULL THEN 'PRI'
                            WHEN uq.name IS NOT NULL THEN 'UNI'
                            WHEN fk_col.constraint_object_id IS NOT NULL THEN 'FK'
                            END AS key_type
                    FROM
                        sys.tables t
                            JOIN
                        sys.columns c ON c.object_id = t.object_id
                            LEFT JOIN
                        sys.index_columns ic ON ic.object_id = t.object_id AND ic.column_id = c.column_id
                            LEFT JOIN
                        sys.foreign_key_columns fk_col ON fk_col.parent_object_id = c.object_id AND fk_col.parent_column_id = c.column_id
                            LEFT JOIN
                        sys.indexes idx ON idx.object_id = t.object_id AND idx.index_id = ic.index_id
                            LEFT JOIN
                        sys.key_constraints pk ON pk.parent_object_id = t.object_id
                            AND pk.type = 'PK'
                            AND pk.unique_index_id = idx.index_id
                            LEFT JOIN
                        sys.indexes uq ON uq.object_id = t.object_id
                            AND uq.is_unique = 1
                            AND uq.is_primary_key = 0
                            AND uq.index_id = ic.index_id
                )
                SELECT
                    key_type
                FROM
                    KeyColumns
                WHERE table_name = '{table_name}' and column_name = '{column_name}'"
                )).await;

            let field_key = field_key.first()
                .expect("Error fetching key")
                .opt_get(0)
                .unwrap_or("NO KEY");

            match field_key {
                "PRI" => {
                    if self.query_is_col_auto_incr(&table_name, &column_name).await {
                        KeyType::Primary(GenerationType::AutoIncrement)
                    } else {
                        KeyType::Primary(GenerationType::None)
                    }
                }
                "FK" => KeyType::Foreign,
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
            let auto_increment = self.query(&format!("SELECT
                    CASE
                        WHEN ic.SEED_VALUE IS NOT NULL THEN 'auto_increment'
                        ELSE ''
                        END AS IS_IDENTITY
                FROM
                    sys.columns col
                        LEFT JOIN
                    sys.tables tab ON col.object_id = tab.object_id
                        LEFT JOIN
                    sys.identity_columns ic ON col.object_id = ic.object_id AND col.column_id = ic.column_id
                WHERE
                    tab.name = '{table_name}' and col.name = '{column_name}';"))
                                     .await;

            let auto_increment = if let Some(auto_increment) = auto_increment.first() {
                auto_increment.get(0)
            } else {
                ""
            };

            matches!(auto_increment, "auto_increment")
        })
    }

    fn query_table_references(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<(Vec<ColumnId>, Vec<ColumnId>)>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {

            let sql = &format!("SELECT
                pk_tab.name AS ReferencedTable,
                pk_col.name AS ReferencedColumn,
                fk_col.name AS ForeignKeyColumn,
                fk.object_id AS fk_id
            FROM
                sys.foreign_keys fk
                    INNER JOIN
                sys.foreign_key_columns fk_cols ON fk.object_id = fk_cols.constraint_object_id
                    INNER JOIN
                sys.tables fk_tab ON fk_tab.object_id = fk.parent_object_id
                    INNER JOIN
                sys.columns fk_col ON fk_col.column_id = fk_cols.parent_column_id AND fk_col.object_id = fk_tab.object_id
                    INNER JOIN
                sys.tables pk_tab ON pk_tab.object_id = fk.referenced_object_id
                    INNER JOIN
                sys.columns pk_col ON pk_col.column_id = fk_cols.referenced_column_id AND pk_col.object_id = pk_tab.object_id
            WHERE
                fk_tab.name = '{table_name}'
            ORDER BY fk.object_id;");

            let mut relations = Vec::new();
            
            let mut last_fk_id = None;
            let mut from = Vec::new();
            let mut to = Vec::new();
            
            for row in self.query(sql).await {
                let ref_table_name: &str = row.get(0);
                let ref_column_name: &str = row.get(1);
                let column_name: &str = row.get(2);
                let fk_id: i32 = row.get(3);

                if last_fk_id.is_some() && last_fk_id.unwrap() != fk_id {
                    relations.push((from, to));
                    from = Vec::new();
                    to = Vec::new();
                }

                from.push(ColumnId::new(&table_name, column_name));
                to.push(ColumnId::new(ref_table_name, ref_column_name));

                last_fk_id.replace(fk_id);
            }

            if !from.is_empty() { relations.push((from, to));  }
            
            relations
        })
    }

    fn query_table_referenced_by(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<(Vec<ColumnId>, Vec<ColumnId>)>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {
            let ref_table_name = table_name.as_str();
            
            let sql = &format!("SELECT
                fk_tab.name AS ReferencingTable,
                fk_col.name AS ReferencingColumn,
                pk_col.name AS ReferencedColumn,
                fk.object_id AS fk_id
            FROM
                sys.foreign_keys fk
                    INNER JOIN
                sys.foreign_key_columns fk_cols ON fk.object_id = fk_cols.constraint_object_id
                    INNER JOIN
                sys.tables fk_tab ON fk_tab.object_id = fk.parent_object_id
                    INNER JOIN
                sys.columns fk_col ON fk_col.object_id = fk_tab.object_id AND fk_col.column_id = fk_cols.parent_column_id
                    INNER JOIN
                sys.columns pk_col ON pk_col.object_id = fk.referenced_object_id AND pk_col.column_id = fk_cols.referenced_column_id
            WHERE
                fk.referenced_object_id = OBJECT_ID('{ref_table_name}')
            ORDER BY fk.object_id;");

            let mut relations = Vec::new();
            
            let mut last_fk_id = None;
            let mut from = Vec::new();
            let mut to = Vec::new();
            
            for row in self.query(sql).await {
                let table_name: &str = row.get(0);
                let column_name: &str = row.get(1);
                let ref_column_name: &str = row.get(2);
                let fk_id: i32 = row.get(3);

                if last_fk_id.is_some() && last_fk_id.unwrap() != fk_id {
                    relations.push((from, to));
                    from = Vec::new();
                    to = Vec::new();
                }
                
                from.push(ColumnId::new(table_name, column_name));
                to.push(ColumnId::new(ref_table_name, ref_column_name));

                last_fk_id.replace(fk_id);
            }

            if !from.is_empty() { relations.push((from, to));  }
            
            relations
        })
    }
}
