#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Invalid connection string: {0}")]
    InvalidConnStringError(String),
    #[error("Not supported DB")]
    NotSupportedDBError,
    #[error("Not enough arguments introduced: {0}")]
    MissingParamError(String),
    #[error("Error introspecting the db: {0}")]
    IntrospectationError(String),
    #[error("Error connecting to the db: {0}")]
    DBConnectionError(String),
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error::IntrospectationError(value.to_string())
    }
}