use displaydoc::Display as DisplayDoc;
use pusher_client::config::PusherConfig;
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};
use tokio_postgres::tls::NoTlsStream;
use tokio_postgres::{Client, Config as TokioPgConfig, Connection, NoTls, Socket};
use twilio_client::config::{TwilioApiConfig, TwilioSmsConfig};

#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// Open config file: {0}
    OpenConfig(std::io::Error),
    /// Failed to parse config: {0}
    ParseConfig(toml::de::Error),
    /// Failed to parse config as utf-8: {0}
    ParseUtf8(std::string::FromUtf8Error),
    /// Failed to read config file: {0}
    ReadConfig(std::io::Error),
    /// Failed to read config metadata: {0}
    ReadMeta(std::io::Error),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct ApiConfig {
    pub bind_host: String,
    pub bind_port: u32,
    pub tls: Option<TlsConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct GrpcConfig {
    pub bind_host: String,
    pub bind_port: u32,
    pub tls: Option<TlsConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct PostgresConfig {
    pub db_host: String,
    pub db_port: u32,
    pub db_name: String,
    pub db_user: String,
    pub db_pwd: String,
}

impl PostgresConfig {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.db_user, self.db_pwd, self.db_host, self.db_port, self.db_name
        )
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct S3Config {
    pub bucket: String,
    pub prefix: Option<String>,
    pub region: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct TlsConfig {
    pub private_key: PathBuf,
    pub certificate: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct TwilioConfig {
    pub api: TwilioApiConfig,
    pub sms: TwilioSmsConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    pub api: ApiConfig,
    pub postgres: PostgresConfig,
    pub near_api: GrpcConfig,
    pub pusher: PusherConfig,
    pub twilio: TwilioConfig,
    pub s3: S3Config,
}

impl Config {
    pub async fn new(path: impl AsRef<Path> + Send) -> Result<Self, Error> {
        read_to_string(path).await?.parse()
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::ParseConfig)
    }
}

async fn read_to_string(path: impl AsRef<Path> + Send) -> Result<String, Error> {
    let mut file = File::open(path).await.map_err(Error::OpenConfig)?;
    let meta = file.metadata().await.map_err(Error::ReadMeta)?;
    let mut contents = Vec::with_capacity(usize::try_from(meta.len()).unwrap_or(0));
    file.read_to_end(&mut contents)
        .await
        .map_err(Error::ReadConfig)?;
    Ok(String::from_utf8(contents).map_err(Error::ParseUtf8)?)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ServerEnv {
    Dev,
    Release,
}

impl ServerEnv {
    pub fn from_str(env: &str) -> ServerEnv {
        let env = env.to_lowercase();
        match env.as_str() {
            "dev" => ServerEnv::Dev,
            "release" => ServerEnv::Release,
            _ => ServerEnv::Dev,
        }
    }
}

pub async fn db_client_from_config(
    config: &PostgresConfig,
) -> Result<(Client, Connection<Socket, NoTlsStream>), crate::error::Error> {
    let mut pg_conn = TokioPgConfig::new();
    pg_conn.user(config.db_user.as_str());
    pg_conn.password(config.db_pwd.as_str());
    pg_conn.port(config.db_port as u16);
    pg_conn.host(config.db_host.as_str());
    pg_conn.dbname(config.db_name.as_str());
    pg_conn.keepalives(true);
    pg_conn.connect_timeout(Duration::new(5, 0));

    log::info!("Db connection info = {:?}", pg_conn);

    pg_conn
        .connect(NoTls)
        .await
        .map_err(crate::error::Error::Postgres)
}
