use crate::db_objects::Database;
use crate::sniffers::{DatabaseSniffer, SniffResults};
use crate::ConnectionParams;
use sqlx::Row;
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

        let tables = self.client.query("show tables", &[])
            .await.expect("Error fetching tables")
            .into_first_result().await.expect("Error fetching tables");
        
        for table in tables.iter() {
            let table: &[u8] = table.get(0).expect("REASON");
            
            //database.add_table(
            //    self.introspect_table(&String::from_utf8_lossy(table.get(1..2).expect("REASON")).to_string())
            //        .await,
            //);
        }

        database
    }

    /*
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
   
   async fn introspect_column(&mut self, column: tiberius::Row, table_name: &str) -> Column {
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
   }*/
}
