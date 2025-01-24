use crate::db_objects::{
    Column, ColumnId, ColumnType, Database, GenerationType, KeyType, Relation, RelationType, Table,
};
use crate::sniffers::{DatabaseSniffer, SniffResults};
use crate::ConnectionParams;
use std::str::FromStr;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt, TokioAsyncWriteCompatExt};

pub struct SQLServerSniffer {
    conn_params: ConnectionParams,
    client: Client<Compat<TcpStream>>,
}

impl DatabaseSniffer for SQLServerSniffer {
    async fn new(params: ConnectionParams) -> Result<Self, crate::Error> {
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

        let sniffer = SQLServerSniffer {
            conn_params: params,
            client,
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

impl SQLServerSniffer {
    async fn introspect_database(&mut self) -> Database {
        let db_name = self.conn_params.dbname.as_ref().unwrap().as_str();

        let mut database = Database::new(db_name);

        let tables = self
            .client
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
            .expect("Error fetching tables");

        for table in tables.iter() {
            let table_name: &str = table.get(0).expect("REASON");

            database.add_table(self.introspect_table(table_name).await);
        }

        database
    }

    async fn introspect_table(&mut self, table_name: &str) -> Table {
        let mut table = Table::new(table_name);

        let columns = self
            .client
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
            .expect("Error describing table");

        #[cfg(test)]
        {
            println!("table: {}", table_name);
        }

        for column in columns {
            let column = self.introspect_column(column, table_name).await;
            table.add_column(column);
        }

        let references = {
            let mut relations: Vec<Relation> = Vec::new();
            // TODO: Check TABLE_CONSTRAINTS to get the type of the relation and
            let sql = "SELECT
                pk_tab.name AS ReferencedTable,
                pk_col.name AS ReferencedColumn,
                fk_col.name AS ForeignKeyColumn,
                fk_tab.name AS ForeignKeyTable
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
                .query(sql, &[&table_name])
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

    async fn introspect_column(&mut self, column: tiberius::Row, table_name: &str) -> Column {
        let column_name: &str = column.get(0).expect("Error fetching column name");
        let field_type: &str = column.get(1).expect("Error fetching field type");
        let field_nullable = column
            .get::<&str, _>(2)
            .expect("Erro fetching nullable column state")
            == "YES";
        let field_default: Option<&str> = column.get(3);

        // TODO: Unique columns are beeing detected as Primary Keys. EX.: Deparment table
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

        #[cfg(test)]
        #[cfg(debug_assertions)]
        {
            println!(
                "name: {:?}, type: {:?}, nullable: {:?}, key: {:?}, default: {:?}",
                column_name,
                field_type,
                field_nullable,
                field_key,
                field_default.unwrap_or_default()
            );
        }

        let key = match field_key {
            "PRI" => {
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
                    "auto_increment" => KeyType::Primary(GenerationType::AutoIncrement),
                    _ => KeyType::Primary(GenerationType::None),
                }
            }
            "FK" => KeyType::Foreign,
            "UNI" => KeyType::Unique,
            _ => KeyType::None,
        };

        // Las PK o UQ también pueden ser FK

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
    }
}
