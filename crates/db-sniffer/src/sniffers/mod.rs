pub(crate) mod mysql;

use crate::ConnectionParams;
use crate::db_objects::{Database, Metadata};

pub struct SniffResults
{
    metadata: Metadata,
    databases: Vec<Database>
}

pub trait DatabaseSniffer
{
    fn new(params: ConnectionParams) -> Self;
    fn sniff(&self) -> SniffResults;
}