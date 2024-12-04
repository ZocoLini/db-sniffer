use crate::ConnectionParams;
use crate::sniffers::{DatabaseSniffer, SniffResults};

pub struct MySQLSniffer
{
    
}

impl DatabaseSniffer for MySQLSniffer {
    fn new(params: ConnectionParams) -> Self {
        todo!()
    }

    fn sniff(&self) -> SniffResults {
        todo!()
    }
}