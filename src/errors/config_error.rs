use std::{env::VarError, net::AddrParseError, num::ParseIntError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("required environment variable {0} is missing")]
    Missing(&'static str),
    #[error("invalid environment variable: {0}")]
    Environment(#[from] VarError),
    #[error("invalid LISTEN_ADDR: {0}")]
    Address(#[from] AddrParseError),
    #[error("invalid MAX_DB_CONNECTIONS: {0}")]
    Number(#[from] ParseIntError),
    #[error("cannot load .env: {0}")]
    Dotenv(#[from] dotenvy::Error),
}
