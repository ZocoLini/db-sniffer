use crate::db_objects::{ColumnId, GenerationType, KeyType, RelationType};
use crate::error::Error::MissingParamError;
use crate::sniffers::DatabaseSniffer;
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

impl DatabaseSniffer for MySQLSniffer {
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
            sqlx::query("show tables")
                .fetch_all(&mut self.conn)
                .await
                .unwrap()
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
            sqlx::query(format!("describe {}", table_name).as_str())
                .fetch_all(&mut self.conn)
                .await
                .expect("Error describing table")
                .iter()
                .map(|row| row.get::<&str, _>(0).to_string())
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
            sqlx::query(format!("describe {}", table_name).as_str())
                .fetch_all(&mut self.conn)
                .await
                .expect("Error describing table")
                .iter()
                .filter_map(|row| {
                    if row.get::<&str, _>(0) == column_name {
                        Some(
                            String::from_utf8_lossy(row.get::<&[u8], _>(1))
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
            sqlx::query(format!("describe {}", table_name).as_str())
                .fetch_all(&mut self.conn)
                .await
                .expect("Error describing table")
                .iter()
                .filter_map(|row| {
                    if row.get::<&str, _>(0) == column_name {
                        Some(row.get::<&str, _>(2).to_string())
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
            sqlx::query(format!("describe {}", table_name).as_str())
                .fetch_all(&mut self.conn)
                .await
                .expect("Error describing table")
                .iter()
                .filter_map(|row| {
                    if row.get::<&str, _>(0) == column_name {
                        row.get::<Option<&str>, _>(4)
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
            let key: String = sqlx::query(format!("describe {}", table_name).as_str())
                .fetch_all(&mut self.conn)
                .await
                .expect("Error describing table")
                .iter()
                .filter_map(|row| {
                    if row.get::<&str, _>(0) == column_name {
                        Some(String::from_utf8_lossy(row.get::<&[u8], _>(3)).to_string())
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
            sqlx::query(format!("describe {}", table_name).as_str())
                .fetch_all(&mut self.conn)
                .await
                .expect("Error describing table")
                .iter()
                .filter_map(|row| {
                    if row.get::<&str, _>(0) == column_name {
                        Some(row.get::<&str, _>(5).to_string())
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

            let sql = "SELECT
                REFERENCED_TABLE_NAME,
                REFERENCED_COLUMN_NAME,
                COLUMN_NAME
            FROM
                INFORMATION_SCHEMA.KEY_COLUMN_USAGE
            WHERE
                TABLE_NAME = ? 
                AND REFERENCED_TABLE_NAME IS NOT NULL;";

            let rows = sqlx::query(sql)
                .bind(&table_name)
                .fetch_all(&mut self.conn)
                .await
                .unwrap();

            for row in rows {
                let ref_table_name: &str = &String::from_utf8_lossy(row.get(0));
                let ref_column_name: &str = row.get(1);
                let column_name: &str = row.get(2);

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
                TABLE_NAME,
                COLUMN_NAME,
                REFERENCED_COLUMN_NAME
            FROM
                information_schema.KEY_COLUMN_USAGE
            WHERE
                REFERENCED_TABLE_NAME = ?";

            let rows = sqlx::query(sql)
                .bind(&ref_table_name)
                .fetch_all(&mut self.conn)
                .await
                .unwrap();

            for row in rows {
                let table_name: &str = &String::from_utf8_lossy(row.get(0));
                let column_name: &str = row.get(1);
                let ref_column_name: &str = row.get(2);

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

    fn introspect_rel_type(
        &mut self,
        from: &Vec<ColumnId>,
        to: &Vec<ColumnId>,
        rel_owner: bool,
    ) -> Pin<Box<dyn Future<Output = RelationType> + Send + '_>> {
        // TODO: Make this work for multiple columns reference
        let from = from.clone();
        let to = to.clone();

        Box::pin(async move {
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

            let rows = sqlx::query(&sql)
                .fetch_all(&mut self.conn)
                .await
                .expect("Shouldn`t fail");

            if rows.is_empty() {
                return RelationType::Unknown;
            }

            let mut is_one_to_one = true;

            for row in rows {
                let count: i32 = row.get(0);
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
/*
impl MySQLSniffer {
    async fn introspect_database(&mut self) -> Database {
        let db_name = self.conn_params.dbname.as_ref().unwrap().as_str();

        let mut database = Database::new(db_name);

        let tables = sqlx::query("show tables")
            .fetch_all(&mut self.conn)
            .await
            .unwrap();

        for table in tables {
            database.add_table(
                self.introspect_table(&String::from_utf8_lossy(table.get(0)))
                    .await,
            );
        }

        database
    }

    async fn introspect_table(&mut self, table_name: &str) -> Table {
        let mut table = Table::new(table_name);

        let columns = sqlx::query(format!("describe {}", table_name).as_str())
            .fetch_all(&mut self.conn)
            .await
            .expect("Error describing table");

        #[cfg(test)]
        {
            println!("table: {}", table_name);
        }

        for column in columns {
            let column = self.introspect_row(column, table_name).await;
            table.add_column(column);
        }

        let references = {
            let mut relations: Vec<Relation> = Vec::new();

            let sql = "SELECT
                REFERENCED_TABLE_NAME,
                REFERENCED_COLUMN_NAME,
                COLUMN_NAME
            FROM
                INFORMATION_SCHEMA.KEY_COLUMN_USAGE
            WHERE
                TABLE_NAME = ?
                AND REFERENCED_TABLE_NAME IS NOT NULL;";

            let rows = sqlx::query(sql)
                .bind(table_name)
                .fetch_all(&mut self.conn)
                .await
                .unwrap();

            for row in rows {
                let ref_table_name: &str = &String::from_utf8_lossy(row.get(0));
                let ref_column_name: &str = row.get(1);
                let column_name: &str = row.get(2);

                #[cfg(test)]
                {
                    println!(
                        "Column {} references {} ({})",
                        column_name, ref_table_name, ref_column_name
                    );
                }

                let from = ColumnId::new(table_name, column_name);
                let to = ColumnId::new(ref_table_name, ref_column_name);

                let from = vec![from];
                let to = vec![to];

                let rel_type = self.introspect_rel_type(&from, &to, true).await;

                relations.push(Relation::new(from, to, rel_type));
            }

            relations
        };

        references
            .into_iter()
            .for_each(|rel: Relation| table.add_reference_to(rel));

        let referenced_by = {
            let mut relations = Vec::new();
            let ref_table_name = table_name;

            let sql = "SELECT
                TABLE_NAME,
                COLUMN_NAME,
                REFERENCED_COLUMN_NAME
            FROM
                information_schema.KEY_COLUMN_USAGE
            WHERE
                REFERENCED_TABLE_NAME = ?";

            let rows = sqlx::query(sql)
                .bind(ref_table_name)
                .fetch_all(&mut self.conn)
                .await
                .unwrap();

            for row in rows {
                let table_name: &str = &String::from_utf8_lossy(row.get(0));
                let column_name: &str = row.get(1);
                let ref_column_name: &str = row.get(2);

                #[cfg(test)]
                {
                    println!(
                        "Column {} is referenced by {} ({})",
                        ref_column_name, table_name, column_name
                    );
                }

                let from = ColumnId::new(table_name, column_name);
                let to = ColumnId::new(ref_table_name, ref_column_name);

                let from = vec![from];
                let to = vec![to];

                let rel_type = self.introspect_rel_type(&from, &to, false).await;

                relations.push(Relation::new(from, to, rel_type));
            }

            relations
        };

        referenced_by
            .into_iter()
            .for_each(|rel: Relation| table.add_referenced_by(rel));

        table
    }

    async fn introspect_row(&mut self, row: MySqlRow, table_name: &str) -> Column {
        let column_name: &str = row.get(0);
        let field_type: &[u8] = row.get(1);
        let field_type = String::from_utf8_lossy(field_type).to_string();
        let field_type = field_type.split("(").next().unwrap();
        let field_nullable: &str = row.get(2);
        let field_nullable: bool = field_nullable == "YES";
        let field_key: &[u8] = row.get(3);
        let field_key = String::from_utf8_lossy(field_key);
        let field_default: Option<&str> = row.get(4);
        let field_extra: &str = row.get(5);

        #[cfg(test)]
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

        Column::new(
            ColumnId::new(table_name, column_name),
            ColumnType::from_str(&field_type.to_string()).unwrap(),
            field_nullable,
            key,
        )
    }

    async fn introspect_rel_type(
        &mut self,
        from: &Vec<ColumnId>,
        to: &Vec<ColumnId>,
        rel_owner: bool,
    ) -> RelationType {
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

        let rows = sqlx::query(&sql)
            .fetch_all(&mut self.conn)
            .await
            .expect("Shouldn`t fail");

        if rows.is_empty() {
            return RelationType::Unknown;
        }

        let mut is_one_to_one = true;

        for row in rows {
            let count: i32 = row.get(0);
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
    }
}
*/
