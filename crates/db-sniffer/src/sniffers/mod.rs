pub(crate) mod mssql;
pub(crate) mod mysql;

use crate::db_objects::{
    Column, ColumnId, ColumnType, Database, Metadata, Relation, RelationType, Table,
};
use crate::{db_objects, ConnectionParams};
use getset::Getters;
use sqlx::{Decode, MySql, Row, Type};
use std::future::Future;
use std::pin::Pin;
use std::process::Output;
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
    ) -> Pin<Box<dyn Future<Output = String> + Send + '_>>;
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
    async fn into_sniffer(
        self,
        conn_params: &ConnectionParams,
    ) -> Result<Box<dyn Sniffer>, crate::Error> {
        // TODO: Remove the clone in the connection params
        let conn_params = conn_params.clone();

        match self {
            SnifferType::MySQL => Ok(Box::new(mysql::MySQLSniffer::new(conn_params).await?)),
            SnifferType::MsSQL => Ok(Box::new(mssql::MSSQLSniffer::new(conn_params).await?)),
        }
    }
}

pub async fn sniff(conn_params: ConnectionParams) -> Result<SniffResults, crate::Error> {
    let mut sniffer = SnifferType::from_str(&conn_params.db)?
        .into_sniffer(&conn_params)
        .await?;

    let database = introspect_database(sniffer.as_mut()).await;
    let metadata = sniffer.query_metadata().await;

    Ok(SniffResults::new(metadata, database, conn_params))
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
        ColumnType::from_str(column_type.as_str()).unwrap(),
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
    // TODO: Add multiple column support
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
            let count: i32 = row.get(0);
            if count != 1 {
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
