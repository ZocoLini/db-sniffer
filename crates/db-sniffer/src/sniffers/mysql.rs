use sqlx::{Connection, MySqlConnection};
use crate::ConnectionParams;
use crate::Error::MissingParamError;
use crate::sniffers::{DatabaseSniffer, SniffResults};

pub struct MySQLSniffer
{
    conn: MySqlConnection
}

impl DatabaseSniffer for MySQLSniffer {
    fn new(params: ConnectionParams) -> Result<Self, crate::Error> {
        let user = params.user.ok_or(MissingParamError("user".to_string()))?;
        let password = params.password.ok_or(MissingParamError("password".to_string()))?;
        let host = params.host.ok_or(MissingParamError("host".to_string()))?;
        let port = params.port.ok_or(MissingParamError("port".to_string()))?;

        let connection = MySqlConnection::connect(&format!(
            "mysql://{}:{}@{}:{}/{}",
            user,
            password,
            host,
            port,
            params.dbname.unwrap_or("".to_string())
        ))?;
        
        let sniffer = MySQLSniffer {
            conn: connection
        };
        
        Ok(sniffer)
    }

    async fn sniff(&self) -> SniffResults {
        SniffResults {
            metadata: Default::default(),
            databases: vec![]
        }
    }
}