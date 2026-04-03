use anyhow::{Context, Ok, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub log_level: String,
    pub app_env: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            host: std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse::<u16>()
                .context("APP_PORT must be a valid port number")?,
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            app_env: std::env::var("APP_ENV").unwrap_or_else(|_| "development".into()),
        })
    }
}
