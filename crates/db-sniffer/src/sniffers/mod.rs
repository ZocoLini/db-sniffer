pub(crate) mod mssql;
pub(crate) mod mysql;

use crate::db_objects::{Column, ColumnId, ColumnType, Database, Metadata, Relation, RelationType, Table};
use crate::{db_objects};
use getset::Getters;
use sqlx::{Decode, MySql, Row, Type};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use tiberius::FromSql;

#[derive(Getters)]
pub struct SniffResults {
    #[get = "pub"]
    metadata: Option<Metadata>,
    #[get = "pub"]
    database: Database,
    #[get = "pub"]
    conn_params: ConnectionParams,
}

impl SniffResults {
    pub fn new(
        metadata: Option<Metadata>,
        database: Database,
        conn_params: ConnectionParams,
    ) -> Self {
        SniffResults {
            metadata,
            database,
            conn_params,
        }
    }
}
#[derive(Clone, Getters)]
pub struct ConnectionParams {
    #[get = "pub"]
    db: String,
    #[get = "pub"]
    user: Option<String>,
    #[get = "pub"]
    password: Option<String>,
    #[get = "pub"]
    host: Option<String>,
    #[get = "pub"]
    port: Option<u16>,
    #[get = "pub"]
    dbname: Option<String>,
}

impl FromStr for ConnectionParams {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(crate::Error::InvalidConnStringError(
                "empty connection string".to_string(),
            ));
        }

        let regex = regex::Regex::new(
            r"(?P<db>[^:]+):/(/(?P<user>[^:^@]+):(?P<password>[^/^@]+))?(@(?P<host>[^:]+):(?P<port>\d+))?(/?|/(?P<dbname>[^/]+))$"
        ).expect("invalid regex");

        let conn_params = regex.captures(s).map(|captures| {
            let db = if let Some(db) = captures.name("db") {
                db.as_str()
            } else {
                return Err(crate::Error::InvalidConnStringError(
                    "missing db param in the connection string".to_string(),
                ));
            };

            let port = if let Some(port) = captures.name("port").map(|port| port.as_str()) {
                if let Ok(port) = port.parse::<u16>() {
                    Some(port)
                } else {
                    return Err(crate::Error::InvalidConnStringError(
                        "port is not a number".to_string(),
                    ));
                }
            } else {
                None
            };

            Ok(ConnectionParams {
                db: db.to_string(),
                user: captures.name("user").map(|user| user.as_str().to_string()),
                password: captures
                    .name("password")
                    .map(|pass| pass.as_str().to_string()),
                host: captures.name("host").map(|host| host.as_str().to_string()),
                port,
                dbname: captures
                    .name("dbname")
                    .map(|dbname| dbname.as_str().to_string()),
            })
        });

        if let Some(conn_params) = conn_params {
            conn_params
        } else {
            Err(crate::Error::InvalidConnStringError(
                "invalid connection string format".to_string(),
            ))
        }
    }
}

/// conn_str: db://user:password@host:port/[dbname]
pub async fn sniff(conn_str: &str) -> Result<SniffResults, crate::Error> {
    let conn_params = conn_str.parse::<ConnectionParams>()?;

    let mut sniffer = SnifferType::from_str(&conn_params.db)?
        .into_sniffer(&conn_params)
        .await?;

    let database = introspect_database(sniffer.as_mut()).await;
    let metadata = sniffer.query_metadata().await;

    drop(sniffer);

    Ok(SniffResults::new(metadata, database, conn_params))
}

/*
 * Tried to implement a generic way to get the rows from the db but failed to do so without enums
 *
 *  trait RowGet<'a> : Sized {
 *      fn generic_get<R: FromSql<'a> + sqlx::Decode<'a, MySql> + sqlx::Type<MySql>>(
 *          &'a self,
 *          index: usize,
 *      ) -> R;
 *  }
 *
 *  trait DatabaseQuerier<'a, T: RowGet<'a> > {
 *      fn query(&mut self, query: &str) -> Pin<Box<dyn Future<Output = Vec<T>> + Send + '_>>;
 *  }
 */

enum RowGetter {
    MSSQLRow(tiberius::Row),
    MySQlRow(sqlx::mysql::MySqlRow),
}

impl RowGetter {
    fn get<'a, T: FromSql<'a> + Decode<'a, MySql> + sqlx::Type<MySql>>(&'a self, i: usize) -> T {
        match self {
            RowGetter::MSSQLRow(a) => a.get::<'a>(i).unwrap(),
            RowGetter::MySQlRow(a) => a.get::<'a, T, _>(i),
        }
    }

    fn opt_get<'a, T: FromSql<'a> + Decode<'a, MySql> + sqlx::Type<MySql>>(
        &'a self,
        i: usize,
    ) -> Option<T> {
        match self {
            RowGetter::MSSQLRow(a) => a.get::<'a>(i),
            RowGetter::MySQlRow(a) => a.get::<'a, Option<T>, _>(i),
        }
    }
}

trait Sniffer {
    // Close db connection
    fn close_conn(self) -> Pin<Box<dyn Future<Output = ()> + Send>>;

    // Query the db
    fn query(&mut self, query: &str) -> Pin<Box<dyn Future<Output = Vec<RowGetter>> + Send + '_>>;

    // Obtein specific metadata
    fn query_metadata(&mut self) -> Pin<Box<dyn Future<Output = Option<Metadata>> + Send + '_>>;

    // Obtein specific metadata
    fn query_dbs_names(&mut self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>>;
    fn query_tab_names(&mut self) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>>;
    fn query_col_names(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<String>> + Send + '_>>;
    fn query_col_type(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = ColumnType> + Send + '_>>;
    fn query_is_col_nullable(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;
    fn query_col_default(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = Option<String>> + Send + '_>>;
    fn query_col_key(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = db_objects::KeyType> + Send + '_>>;
    fn query_is_col_auto_incr(
        &mut self,
        table_name: &str,
        column_name: &str,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;
    fn query_table_references(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<(Vec<ColumnId>, Vec<ColumnId>)>> + Send + '_>>;
}

enum SnifferType {
    MySQL,
    MsSQL,
}

impl FromStr for SnifferType {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mysql" | "mariadb" => Ok(SnifferType::MySQL),
            "mssql" | "sqlserver" => Ok(SnifferType::MsSQL),
            _ => Err(crate::Error::NotSupportedDBError),
        }
    }
}

impl SnifferType {
    async fn into_sniffer<'a>(
        self,
        conn_params: &'a ConnectionParams,
    ) -> Result<Box<dyn Sniffer + 'a>, crate::Error> {
        match self {
            SnifferType::MySQL => Ok(Box::new(mysql::MySQLSniffer::new(conn_params).await?)),
            SnifferType::MsSQL => Ok(Box::new(mssql::MSSQLSniffer::new(conn_params).await?)),
        }
    }
}

async fn introspect_database(sniffer: &mut (impl Sniffer + ?Sized)) -> Database {
    let mut database = Database::new(sniffer.query_dbs_names().await.first().unwrap());

    for table in sniffer.query_tab_names().await {
        database.add_table(introspect_table(sniffer, &table).await);
    }

    database
}

async fn introspect_table(
    sniffer: &mut (impl Sniffer + ?Sized),
    table_name: &str,
) -> Table {
    let mut table = Table::new(table_name);

    for column in sniffer.query_col_names(table_name).await {
        let column = introspect_column(sniffer, &column, table_name).await;
        table.add_column(column);
    }

    for (from, to) in sniffer.query_table_references(table_name).await {
        // All the columns in the 'from' of the relations should be in the actual table
        for x in from.iter() {
            assert_eq!(x.table(), table_name);
        }

        let rel = introspect_rel(sniffer, from, to, true).await;
        table.add_reference_to(rel);
    }

    table
}

async fn introspect_column(
    sniffer: &mut (impl Sniffer + ?Sized),
    column_name: &str,
    table_name: &str,
) -> Column {
    let column_type = sniffer.query_col_type(table_name, column_name).await;
    let nullable = sniffer.query_is_col_nullable(table_name, column_name).await;
    let _ = sniffer.query_col_default(table_name, column_name).await;
    let key = sniffer.query_col_key(table_name, column_name).await;
    
    Column::new(
        ColumnId::new(table_name, column_name),
        column_type,
        nullable,
        key,
    )
}

async fn introspect_rel(
    sniffer: &mut (impl Sniffer + ?Sized),
    from: Vec<ColumnId>,
    to: Vec<ColumnId>,
    rel_owner: bool,
) -> Relation {
    let from_table = from[0].table();
    let to_table = to[0].table();

    let from_col = from[0].name();
    let to_col = to[0].name();
    
    assert_eq!(from.len(), to.len());
    
    // TODO: Add multiple column support
    // let mut on_string = "".to_string();
    //
    // from.iter().enumerate().for_each(|(i, _)| {
    //     if i != 0 {
    //         on_string.push_str(" and ");
    //     }
    //     on_string.push_str(&format!("f.{} = t.{}", from[i].name(), to[i].name()));
    // });
    // let mut by_string = "".to_string();
    // 
    // from.iter().enumerate().for_each(|(i, _)| {
    //     if i != 0 {
    //         by_string.push_str(", ");
    //     }
    //     by_string.push_str(&format!("t.{}", to[i].name()));
    // });
    
    let sql = format!(
        r#"
        select count(*) 
            from {from_table} f inner join {to_table} t on f.{from_col} = t.{to_col}
            group by t.{to_col};"#,
    );

    let rows: Vec<RowGetter> = sniffer.query(&sql).await;

    let rel_type = if rows.is_empty() {
        if rel_owner {
            RelationType::ManyToOne
        } else {
            RelationType::OneToMany
        }
    } else {
        let mut is_one_to_one = true;

        for row in rows {
            if row.get::<i32>(0) != 1 {
                is_one_to_one = false;
                break;
            }
        }

        if is_one_to_one {
            RelationType::OneToOne
        } else if rel_owner {
            RelationType::ManyToOne
        } else {
            RelationType::OneToMany
        }
    };

    Relation::new(from, to, rel_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_params_from_valid_str() {
        // Usual db with a user, a host, and a specific db
        let conn_str = "db://user:password@localhost:3306/dbname";
        let conn_params = conn_str.parse::<ConnectionParams>().unwrap();
        validate_obl_params(&conn_params);
        assert_eq!(conn_params.dbname, Some("dbname".to_string()));

        // Embedded db without a user
        let conn_str = "sqlite://dbname";
        let conn_params = conn_str.parse::<ConnectionParams>().unwrap();
        assert_eq!(conn_params.db, "sqlite");
        assert_eq!(conn_params.user, None);
        assert_eq!(conn_params.password, None);
        assert_eq!(conn_params.host, None);
        assert_eq!(conn_params.port, None);
        assert_eq!(conn_params.dbname, Some("dbname".to_string()));

        // Embedded db with user
        let conn_str = "sqlite://user:password/dbname";
        let conn_params = conn_str.parse::<ConnectionParams>().unwrap();
        assert_eq!(conn_params.db, "sqlite");
        assert_eq!(conn_params.user.clone().unwrap(), "user");
        assert_eq!(conn_params.password.clone().unwrap(), "password");
        assert_eq!(conn_params.host, None);
        assert_eq!(conn_params.port, None);
        assert_eq!(conn_params.dbname, Some("dbname".to_string()));

        // Usual db with a user and a host
        let conn_str = "db://user:password@localhost:3306";
        let conn_params = conn_str.parse::<ConnectionParams>().unwrap();
        validate_obl_params(&conn_params);
        assert_eq!(conn_params.dbname, None);

        // Usual db with a user and a host - 2
        let conn_str = "db://user:password@localhost:3306/";
        let conn_params = conn_str.parse::<ConnectionParams>().unwrap();
        validate_obl_params(&conn_params);
        assert_eq!(conn_params.dbname, None);

        fn validate_obl_params(conn_params: &ConnectionParams) {
            assert_eq!(conn_params.db, "db");
            assert_eq!(conn_params.user.clone().unwrap(), "user");
            assert_eq!(conn_params.password.clone().unwrap(), "password");
            assert_eq!(conn_params.host.clone().unwrap(), "localhost");
            assert_eq!(conn_params.port.unwrap(), 3306);
        }
    }

    #[test]
    fn test_connection_params_from_invalid_str() {
        let conn_str = "db://userpassword@localhost:3306/dbname/";
        assert!(conn_str.parse::<ConnectionParams>().is_err());

        let conn_str = "db://user:password@localhost:d306/";
        assert!(conn_str.parse::<ConnectionParams>().is_err());

        let conn_str = "db://user:password@localhost:3306/dbname//";
        assert!(conn_str.parse::<ConnectionParams>().is_err());

        let conn_str = "db://user:password@localhost:3306//dbname";
        assert!(conn_str.parse::<ConnectionParams>().is_err());

        let conn_str = "db:/user:password@localhost:3306/dbname";
        assert!(conn_str.parse::<ConnectionParams>().is_err());

        let conn_str = "db://a:b@localhost3306/dbname";
        assert!(conn_str.parse::<ConnectionParams>().is_err());

        let conn_str = "a:b";
        assert!(conn_str.parse::<ConnectionParams>().is_err());
    }
}