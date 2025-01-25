pub(crate) mod mssql;
pub(crate) mod mysql;

use crate::db_objects::{
    Column, ColumnId, ColumnType, Database, Metadata, Relation, RelationType, Table,
};
use crate::{db_objects, ConnectionParams};
use getset::Getters;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

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

trait RowGet<'a, T: ?Sized> {
    fn get<R>(&'a self, index: usize) -> R;
}

trait DatabaseQuerier<T> {
    fn query(&mut self, query: &str) -> Pin<Box<dyn Future<Output = Vec<T>> + Send + '_>>;
}

trait DatabaseSniffer {
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
    let rel_type = sniffer.introspect_rel_type(&from, &to, rel_owner).await;
    Relation::new(from, to, rel_type)
}
