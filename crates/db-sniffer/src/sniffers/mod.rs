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

trait RowGet<'a> : Sized {
    fn generic_get<R: FromSql<'a> + sqlx::Decode<'a, MySql> + sqlx::Type<MySql>>(
        &'a self,
        index: usize,
    ) -> R;
}

trait DatabaseQuerier<'a, T: RowGet<'a> > {
    fn query(&mut self, query: &str) -> Pin<Box<dyn Future<Output = Vec<T>> + Send + '_>>;
}

enum RowGetEnum {
    MSSQLRow(tiberius::Row),
    MySQlRow(sqlx::mysql::MySqlRow)
}

impl RowGetEnum {
    fn generic_get<'a, T: FromSql<'a> + Decode<'a, MySql> + sqlx::Type<MySql>>(&'a self, i: usize) -> T {
        match self {
            RowGetEnum::MSSQLRow(a) => a.get::<'a>(i).unwrap(),
            RowGetEnum::MySQlRow(a) => a.get::<'a, T, _>(i)
        }
    }
}

trait DatabaseSniffer {
    // Query the db
    fn generic_get(&mut self, query: &str) -> Pin<Box<dyn Future<Output = Vec<RowGetEnum>> + Send + '_>>;
    
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
    fn query_table_referenced_by(
        &mut self,
        table_name: &str,
    ) -> Pin<Box<dyn Future<Output = Vec<(Vec<ColumnId>, Vec<ColumnId>)>> + Send + '_>>;
    fn introspect_rel_type(
        &mut self,
        from: &Vec<ColumnId>,
        to: &Vec<ColumnId>,
        rel_owner: bool,
    ) -> Pin<Box<dyn Future<Output = RelationType> + Send + '_>>;
}

pub async fn sniff(conn_params: ConnectionParams) -> Result<SniffResults, crate::Error> {
    // TODO: Remove the clone in the connection params
    let mut sniffer: Box<dyn DatabaseSniffer> = {
        let conn_params = conn_params.clone();

        match conn_params.db.to_lowercase().as_str() {
            "mysql" | "mariadb" => Box::new(mysql::MySQLSniffer::new(conn_params).await?),
            "mssql" | "sqlserver" => Box::new(mssql::MSSQLSniffer::new(conn_params).await?),
            _ => return Err(crate::Error::NotSupportedDBError),
        }
    };

    let database = introspect_database(sniffer.as_mut()).await;

    Ok(SniffResults::new(None, database, conn_params))
}

async fn introspect_database(sniffer: &mut (impl DatabaseSniffer + ?Sized)) -> Database {
    let mut database = Database::new(sniffer.query_dbs_names().await.get(0).unwrap());

    for table in sniffer.query_tab_names().await {
        database.add_table(introspect_table(sniffer, &table).await);
    }

    database
}

async fn introspect_table(
    sniffer: &mut (impl DatabaseSniffer + ?Sized),
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

    for (from, to) in sniffer.query_table_referenced_by(table_name).await {
        // All the columns in the 'to' of the relations should be in the actual table
        for x in to.iter() {
            assert_eq!(x.table(), table_name);
        }

        let rel = introspect_rel(sniffer, from, to, false).await;
        table.add_referenced_by(rel);
    }

    table
}

async fn introspect_column(
    sniffer: &mut (impl DatabaseSniffer + ?Sized),
    column_name: &str,
    table_name: &str,
) -> Column {
    let column_type = sniffer.query_col_type(table_name, &column_name).await;
    let nullable = sniffer
        .query_is_col_nullable(table_name, &column_name)
        .await;
    let _ = sniffer.query_col_default(table_name, &column_name).await;
    let key = sniffer.query_col_key(table_name, &column_name).await;

    Column::new(
        ColumnId::new(table_name, column_name),
        ColumnType::from_str(column_type.as_str()).unwrap(),
        nullable,
        key,
    )
}

async fn introspect_rel(
    sniffer: &mut (impl DatabaseSniffer + ?Sized),
    from: Vec<ColumnId>,
    to: Vec<ColumnId>,
    rel_owner: bool,
) -> Relation {
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
    
    let rows: Vec<RowGetEnum> = sniffer.generic_get(&sql).await;
    
    let rel_type = if rows.is_empty() {
        RelationType::Unknown
    } else {
        let mut is_one_to_one = true;

        for row in rows {
            let count: i32 = row.generic_get(0);
            if count != 1 {
                is_one_to_one = false;
                break;
            }
        }

        if is_one_to_one {
            RelationType::OneToOne
        } else {
            if rel_owner {
                RelationType::ManyToOne
            } else {
                RelationType::OneToMany
            }
        }
    };
    
    Relation::new(from, to, rel_type)
}
