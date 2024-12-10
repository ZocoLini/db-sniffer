use std::ops::Deref;
use sqlx::{Connection, Executor, MySqlConnection, Row};
use crate::ConnectionParams;
use crate::Error::MissingParamError;
use crate::sniffers::{DatabaseSniffer, SniffResults};

pub struct MySQLSniffer
{
    conn_params: ConnectionParams,
    conn: MySqlConnection
}

impl DatabaseSniffer for MySQLSniffer {
    async fn new(params: ConnectionParams) -> Result<Self, crate::Error> {
        // TODO: Remove the clone
        let params_clone = params.clone();
        
        let user = params.user.ok_or(MissingParamError("user".to_string()))?;
        let password = params.password.ok_or(MissingParamError("password".to_string()))?;
        let host = params.host.ok_or(MissingParamError("host".to_string()))?;
        let port = params.port.ok_or(MissingParamError("port".to_string()))?;
        let dbname = params.dbname.unwrap_or("".to_string());

        let connection = MySqlConnection::connect(&format!(
            "mysql://{}:{}@{}:{}/{}",
            user,
            password,
            host,
            port,
            dbname
        )).await?;
        
        let sniffer = MySQLSniffer {
            conn_params: params_clone,
            conn: connection
        };
        
        Ok(sniffer)
    }

    async fn sniff(&mut self) -> SniffResults {
        #[cfg(test)]
        {
            let rows = sqlx::query("SHOW DATABASES")
                .fetch_all(&mut self.conn)
                .await.unwrap();

            for row in rows {
                let database: &str = row.get(0);
                println!("{}", database);
            }
        }
        
        SniffResults {
            metadata: Default::default(),
            databases: vec![]
        }
    }
}

#[cfg(test)]
mod test
{
    use super::*;

    #[tokio::test]
    async fn test_mysql_sniffer()
    {
        let conn_str = "mysql://user:password@host:3306/dbname";
        let conn_params = conn_str.parse().unwrap();
        
        let sniffer = MySQLSniffer::new(conn_params).await;
    }
}