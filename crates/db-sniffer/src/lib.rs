use crate::sniffers::{DatabaseSniffer, SniffResults};
use getset::Getters;
use std::str::FromStr;

#[allow(unused)]
pub mod db_objects;
#[allow(unused)]
pub mod generators;
#[allow(unused)]
mod sniffers;

mod naming;

#[derive(Debug)]
pub enum Error {
    InvalidConnStringError,
    NotSupportedDBError,
    MissingParamError(String),
    SQLxError(sqlx::Error),
    DBConnectionError(String),
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        match value {
            _ => Error::SQLxError(value),
        }
    }
}

/// conn_str: db://user:password@host:port/[dbname]
pub async fn sniff(conn_str: &str) -> Result<SniffResults, Error> {
    let conn_params = conn_str.parse::<ConnectionParams>()?;

    match conn_params.db.to_lowercase().as_str() {
        "mysql" => {
            let sniffer = sniffers::mysql::MySQLSniffer::new(conn_params).await?;
            Ok(sniffer.sniff().await)
        }
        "mssql" | "sqlserver" => {
            let sniffer = sniffers::mssql::SQLServerSniffer::new(conn_params).await?;
            Ok(sniffer.sniff().await)
        }

        _ => Err(Error::NotSupportedDBError),
    }
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
        let regex = regex::Regex::new(
            r"(?P<db>[^:]+):/(/(?P<user>[^:^@]+):(?P<password>[^/^@]+))?(@(?P<host>[^:]+):(?P<port>\d+))?(/?|/(?P<dbname>[^/]+))$"
        ).expect("Should be a valid regex");

        regex
            .captures(s)
            .map(|captures| ConnectionParams {
                db: captures.name("db").unwrap().as_str().to_string(),
                user: captures.name("user").map(|user| user.as_str().to_string()),
                password: captures
                    .name("password")
                    .map(|pass| pass.as_str().to_string()),
                host: captures.name("host").map(|host| host.as_str().to_string()),
                port: captures
                    .name("port")
                    .map(|port| port.as_str().parse().expect("Isn't a number")),
                dbname: captures
                    .name("dbname")
                    .map(|dbname| dbname.as_str().to_string()),
            })
            .ok_or(Error::InvalidConnStringError)
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
