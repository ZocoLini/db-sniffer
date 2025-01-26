extern crate core;

use crate::sniffers::{SniffResults};
use getset::Getters;
use std::str::FromStr;

#[allow(unused)]
pub mod db_objects;
#[allow(unused)]
pub mod generators;
#[allow(unused)]
mod sniffers;

mod error;
mod naming;

pub use error::Error;

/// conn_str: db://user:password@host:port/[dbname]
pub async fn sniff(conn_str: &str) -> Result<SniffResults, Error> {
    let conn_params = conn_str.parse::<ConnectionParams>()?;
    sniffers::sniff(conn_params).await
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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error::InvalidConnStringError(
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
                return Err(Error::InvalidConnStringError(
                    "missing db param in the connection string".to_string(),
                ));
            };

            let port = if let Some(port) = captures.name("port").map(|port| port.as_str()) {
                if let Ok(port) = port.parse::<u16>() {
                    Some(port)
                } else {
                    return Err(Error::InvalidConnStringError("port is not a number".to_string()));
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
            Err(Error::InvalidConnStringError(
                "invalid connection string format".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ConnectionParams;

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
            assert_eq!(conn_params.port.clone().unwrap(), 3306);
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
