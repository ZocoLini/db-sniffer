use crate::db_objects::{
    ColumnId, GenerationType, KeyType, RelationType,
};
use crate::sniffers::DatabaseSniffer;
use crate::ConnectionParams;
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

impl DatabaseSniffer for MSSQLSniffer {
    fn query_dbs_names(&mut self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(async move {
            let db_name = self.conn_params.dbname.as_ref().unwrap().as_str();

            vec![db_name.to_string()]
        })
    }

    fn query_tab_names(&mut self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        Box::pin(async move {
            self.client
                .query(
                    r#"
        select TABLE_NAME 
        from INFORMATION_SCHEMA.TABLES  
        where TABLE_TYPE = 'BASE TABLE';"#,
                    &[],
                )
                .await
                .expect("Error fetching tables")
                .into_first_result()
                .await
                .expect("Error fetching tables")
                .iter()
                .map(|row| {
                    row.get::<&str, _>(0)
                        .expect("Error fetching table name")
                        .to_string()
                })
                .collect()
        })
    }

    fn query_col_names(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {
            self.client
                .query(
                    "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_DEFAULT
            FROM 
                INFORMATION_SCHEMA.COLUMNS
            WHERE 
                TABLE_NAME = @P1;",
                    &[&table_name],
                )
                .await
                .expect("Error describing table")
                .into_first_result()
                .await
                .expect("Error describing table")
                .iter()
                .map(|row| {
                    row.get::<&str, _>(0)
                        .expect("Error fetching column name")
                        .to_string()
                })
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
            self.client
                .query(
                    "SELECT DATA_TYPE
            FROM 
                INFORMATION_SCHEMA.COLUMNS
            WHERE 
                TABLE_NAME = @P1 AND COLUMN_NAME = @P2",
                    &[&table_name, &column_name],
                )
                .await
                .expect("Error describing table")
                .into_first_result()
                .await
                .expect("Error describing table")
                .iter()
                .map(|row| {
                    row.get::<&str, _>(0)
                        .expect("Error fetching column name")
                        .to_string()
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
            self.client
                .query(
                    "SELECT IS_NULLABLE
            FROM 
                INFORMATION_SCHEMA.COLUMNS
            WHERE 
                TABLE_NAME = @P1 AND COLUMN_NAME = @P2",
                    &[&table_name, &column_name],
                )
                .await
                .expect("Error describing table")
                .into_first_result()
                .await
                .expect("Error describing table")
                .iter()
                .map(|row| row.get::<&str, _>(0).expect("Error fetching column name"))
                .collect::<String>()
                == "YES".to_string()
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
        self.client
            .query(
                "SELECT COLUMN_DEFAULT
            FROM 
                INFORMATION_SCHEMA.COLUMNS
            WHERE 
                TABLE_NAME = @P1 AND COLUMN_NAME = @P2",
                &[&table_name, &column_name],
            )
            .await
            .expect("Error describing table")
            .into_first_result()
            .await
            .expect("Error describing table")
            .iter()
            .next()?
            .get::<&str, _>(0)
            .and_then(|s| Some(s.to_string()))
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
                .client
                .query(
                    "WITH KeyColumns AS (
                    SELECT
                        SCHEMA_NAME(t.schema_id) AS schema_name,
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
                WHERE table_name = @P1 and column_name = @P2",
                    &[&table_name.to_string(), &column_name.to_string()],
                )
                .await
                .expect("Error fetching key")
                .into_first_result()
                .await
                .expect("Error fetching key");

            let field_key = field_key
                .get(0)
                .expect("Error fetching key")
                .get(0)
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
            let auto_increment = self.client.query("SELECT
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
                    tab.name = @P1 and col.name = @P1;", &[&table_name.to_string(), &column_name.to_string()])
                                     .await
                                     .expect("Error fetching key")
                                     .into_first_result()
                                     .await
                                     .expect("Error fetching key");

            let auto_increment = if let Some(auto_increment) = auto_increment.get(0) {
                auto_increment.get(0).expect("Error fetching key")
            } else {
                ""
            };

            match auto_increment {
                "auto_increment" => true,
                _ => false,
            }
        })
    }

    fn query_table_references(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<(Vec<ColumnId>, Vec<ColumnId>)>> + Send + '_>> {
        let table_name = table_name.to_string();

        Box::pin(async move {
            let mut relations = Vec::new();

            let sql = "SELECT
                pk_tab.name AS ReferencedTable,
                pk_col.name AS ReferencedColumn,
                fk_col.name AS ForeignKeyColumn
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
                fk_tab.name = @P1;";

            let rows = self
                .client
                .query(sql, &[&table_name])
                .await
                .expect("Error fetching references")
                .into_first_result()
                .await
                .expect("Error fetching references");

            for row in rows {
                let ref_table_name: &str = row.get(0).expect("Error fetching ref table name");
                let ref_column_name: &str = row.get(1).expect("Error fetching ref column name");
                let column_name: &str = row.get(2).expect("Error fetching column name");

                #[cfg(test)]
                {
                    println!(
                        "Column {} references {} ({})",
                        column_name, ref_table_name, ref_column_name
                    );
                }

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

            let sql = "SELECT
                fk_tab.name AS ReferencingTable,
                fk_col.name AS ReferencingColumn,
                pk_col.name AS ReferencedColumn
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
                fk.referenced_object_id = OBJECT_ID(@P1);";

            let rows = self
                .client
                .query(sql, &[&ref_table_name])
                .await
                .expect("Error fetching referenced by")
                .into_first_result()
                .await
                .expect("Error fetching referenced by");

            for row in rows {
                let table_name: &str = row.get(0).expect("Error fetching table name");
                let column_name: &str = row.get(1).expect("Error fetching column name");
                let ref_column_name: &str = row.get(2).expect("Error fetching ref column name");

                #[cfg(test)]
                {
                    println!(
                        "Column {} is referenced by {} ({})",
                        ref_column_name, table_name, column_name
                    );
                }

                let from = ColumnId::new(table_name, column_name);
                let to = ColumnId::new(&ref_table_name, ref_column_name);

                let from = vec![from];
                let to = vec![to];

                relations.push((from, to));
            }

            relations
        })
    }

    // TODO: Move to super
    fn introspect_rel_type(
        &mut self,
        from: &Vec<ColumnId>,
        to: &Vec<ColumnId>,
        rel_owner: bool,
    ) -> Pin<Box<dyn Future<Output = RelationType> + Send + '_>> {

        let from = (*from).clone();
        let to = (*to).clone();
        
        Box::pin(async move {
            // TODO: Make this work for multiple columns reference
            let from_table = from[0].table();
            let to_table = to[0].table();

            let from_col = from[0].name();
            let to_col = to[0].name();

            let sql = format!(
                r#"
        select count(*)
            from {from_table} f inner join {to_table} t on f.{from_col} = t.{to_col}
            group by t.{to_col};"#,
            );

            let rows = self
                .client
                .query(&sql, &[])
                .await
                .expect("Shouldn`t fail")
                .into_first_result()
                .await
                .expect("Shouldn`t fail");

            if rows.is_empty() {
                return RelationType::Unknown;
            }

            let mut is_one_to_one = true;

            for row in rows {
                let count: i32 = row.get(0).expect("Shouldn`t fail");
                if count != 1 {
                    is_one_to_one = false;
                    break;
                }
            }

            if is_one_to_one {
                return RelationType::OneToOne;
            }

            if rel_owner {
                RelationType::ManyToOne
            } else {
                RelationType::OneToMany
            }
        })
    }
}