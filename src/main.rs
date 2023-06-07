use std::fmt::Debug;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use atty::Stream;
use figment::Figment;
use figment::providers::{Env, Format, Yaml};
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing::level_filters::LevelFilter;

mod logging;
mod web;
mod aiven_types;
mod mutators;

#[derive(Debug, Deserialize, Serialize)]
pub enum LogFormat {
    Plain,
    Json,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl Into<LevelFilter> for &LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
        }
    }
}

impl Default for LogFormat {
    fn default() -> Self {
        match atty::is(Stream::Stdout) {
            true => LogFormat::Plain,
            false => LogFormat::Json,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WebConfig {
    // The address:port to bind to
    bind_address: String,
    // Path to certificate for TLS
    certificate_path: Option<PathBuf>,
    // Path to private key for TLS
    private_key_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tenant {
    environment: String,
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // Logging format to use
    #[serde(default)]
    log_format: LogFormat,
    // Log level
    log_level: LogLevel,
    // Configure web server
    web: WebConfig,
    // Tenant details
    tenant: Tenant,
    // Aiven VPC ID
    project_vpc_id: String,
    // Cloud location (eq. europe-north1)
    location: String,
}

fn main() -> Result<()> {
    let config = load_config().map_err(|e| anyhow!(e))?;
    app(config)?;
    Ok(())
}

fn load_config() -> Result<Config, figment::Error> {
    let defaults = "\
log_level: Info
web:
    bind_address: 0.0.0.0:9443
tenant:
    environment: local
    name: local
location: europe-north1
    ";
    Figment::new()
        .merge(Yaml::string(defaults))
        .merge(Env::prefixed("MUTILATOR__").split("__"))
        .extract()
}

#[tokio::main]
async fn app(config: Config) -> Result<()> {
    logging::init_logging(&config)?;
    info!("Configuration loaded: {:?}", config);

    web::start_web_server(config).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use figment::Jail;
    use pretty_assertions::assert_eq;
    use rstest::*;

    use super::*;

    #[rstest]
    pub fn test_load_config() {
        Jail::expect_with(|jail| {
            jail.set_env("MUTILATOR__LOCATION", "my-location");
            jail.set_env("MUTILATOR__PROJECT_VPC_ID", "my-vpc-id");

            let config = load_config()?;

            assert_eq!(config.location, "my-location");

            Ok(())
        })
    }
}