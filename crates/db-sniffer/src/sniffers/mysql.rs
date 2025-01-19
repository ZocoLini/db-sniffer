use crate::db_objects::{
    Column, ColumnId, ColumnType, Database, GenerationType, KeyType, Relation, RelationType, Table,
};
use crate::sniffers::{DatabaseSniffer, SniffResults};
use crate::ConnectionParams;
use crate::Error::MissingParamError;
use sqlx::mysql::MySqlRow;
use sqlx::{Connection, Executor, MySqlConnection, Row};
use std::ops::Deref;
use std::str::FromStr;

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
                let ref_table_name: &str = row.get(0);
                let ref_column_name: &str = row.get(1);
                let column_name: &str = row.get(2);

                #[cfg(debug_assertions)] #[cfg(test)]
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
                let table_name: &str = row.get(0);
                let column_name: &str = row.get(1);
                let ref_column_name: &str = row.get(2);

                #[cfg(debug_assertions)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_mysql_sniffer() {
        let conn_str = "mysql://root:abc123.@10.0.2.4:3306";
        let conn_params = conn_str.parse().unwrap();

        assert!(MySQLSniffer::new(conn_params).await.is_err());

        let conn_params =
            ConnectionParams::from_str("mysql://root:abc123.@10.0.2.4:3306/bdempresa").unwrap();

        let sniffer = MySQLSniffer::new(conn_params).await;
        assert!(sniffer.is_ok());

        let mut sniffer = sniffer.unwrap();

        let results = sniffer.sniff().await;
    }
}
