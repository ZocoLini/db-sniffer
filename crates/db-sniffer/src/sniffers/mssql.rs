use crate::sniffers::{DatabaseSniffer, SniffResults};
use crate::ConnectionParams;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt, TokioAsyncWriteCompatExt};

pub struct SQLServerSniffer {
    conn_params: ConnectionParams,
    client: Client<Compat<TcpStream>>,
}

impl DatabaseSniffer for SQLServerSniffer {
    async fn new(params: ConnectionParams) -> Result<Self, crate::Error> {
        let user = params
            .user
            .as_ref()
            .ok_or(crate::Error::MissingParamError("user".to_string()))?;
        let password = params
            .password
            .as_ref()
            .ok_or(crate::Error::MissingParamError("password".to_string()))?;
        let host = params
            .host
            .as_ref()
            .ok_or(crate::Error::MissingParamError("host".to_string()))?;
        let port = params
            .port
            .as_ref()
            .ok_or(crate::Error::MissingParamError("port".to_string()))?;
        let dbname = params
            .dbname
            .as_ref()
            .ok_or(crate::Error::MissingParamError("dbname".to_string()))?;

        let mut config = Config::new();

        config.host(host);
        config.port(*port);

        config.authentication(AuthMethod::sql_server(user, password));

        config.trust_cert();

        let tcp = TcpStream::connect(config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        // To be able to use Tokio's tcp, we're using the `compat_write` from
        // the `TokioAsyncWriteCompatExt` to get a stream compatible with the
        // traits from the `futures` crate.
        let mut client = Client::connect(config, tcp.compat_write()).await?;

        let sniffer = SQLServerSniffer {
            conn_params: params,
            client: client,
        };

        Ok(sniffer)
    }

    async fn sniff(mut self) -> SniffResults {
        let database = self.introspect_database().await;

        SniffResults {
            metadata: Default::default(),
            database,
            conn_params: self.conn_params,
        }
    }
}
