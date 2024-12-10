pub(crate) mod mysql;

use crate::ConnectionParams;
use crate::db_objects::{Database, Metadata};

pub struct SniffResults
{
    metadata: Option<Metadata>,
    databases: Vec<Database>
}

pub trait DatabaseSniffer: Sized
{
    async fn new(params: ConnectionParams) -> Result<Self, crate::Error>;
    async fn sniff(&mut self) -> SniffResults;
}