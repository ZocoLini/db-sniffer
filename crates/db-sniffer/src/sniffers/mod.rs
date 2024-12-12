pub(crate) mod mysql;

use crate::db_objects::{Database, Metadata};
use crate::ConnectionParams;
use getset::Getters;

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

pub trait DatabaseSniffer: Sized {
    async fn new(params: ConnectionParams) -> Result<Self, crate::Error>;
    async fn sniff(self) -> SniffResults;
}
