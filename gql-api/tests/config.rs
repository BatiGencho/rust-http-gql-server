use displaydoc::Display as DisplayDoc;
use serde::Deserialize;
use std::{
    path::Path,
    str::{self, FromStr},
};
use thiserror::Error;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// Open config file: {0}
    OpenConfig(std::io::Error),
    /// Failed to parse config: {0}
    ParseConfig(toml::de::Error),
    /// Failed to parse config as utf-8: {0}
    ParseUtf8(std::str::Utf8Error),
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
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    pub api: ApiConfig,
}

impl Config {
    /// Initialize the `Config` from the supplied TOML file.
    pub async fn new<P: AsRef<Path> + Send>(toml: P) -> Result<Self, Error> {
        let mut file = File::open(toml).await.map_err(Error::OpenConfig)?;
        let meta = file.metadata().await.map_err(Error::ReadMeta)?;
        let mut contents = Vec::with_capacity(usize::try_from(meta.len()).unwrap_or(100));
        file.read_to_end(&mut contents)
            .await
            .map_err(Error::ReadConfig)?;

        str::from_utf8(&contents)
            .map_err(Error::ParseUtf8)
            .and_then(str::parse)
    }
}

impl FromStr for Config {
    type Err = Error;

    /// Parse the `Config` from TOML.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(Error::ParseConfig)
    }
}
