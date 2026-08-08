use std::{env, io::ErrorKind, net::SocketAddr};

use crate::errors::config_error::ConfigError;

pub struct Config {
    pub listen_addr: SocketAddr,
    pub database_url: String,
    pub max_db_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        load_dotenv()?;

        Ok(Self {
            listen_addr: required_var("LISTEN_ADDR")?.parse()?,
            database_url: required_var("DATABASE_URL")?,
            max_db_connections: required_var("MAX_DB_CONNECTIONS")?.parse()?,
        })
    }
}

fn required_var(name: &'static str) -> Result<String, ConfigError> {
    match env::var(name) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Err(ConfigError::Missing(name)),
        Err(error) => Err(error.into()),
    }
}

fn load_dotenv() -> Result<(), ConfigError> {
    match dotenvy::dotenv() {
        Ok(_) => Ok(()),
        Err(dotenvy::Error::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
