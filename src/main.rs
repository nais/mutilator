use std::path::PathBuf;

use anyhow::Result;
use atty::Stream;
use figment::Figment;
use figment::providers::{Env, Yaml, Format};
use log::info;
use serde::{Deserialize, Serialize};

mod logging;
mod web;
mod aiven_types;

#[derive(Debug, Deserialize, Serialize)]
pub enum LogFormat {
    Plain,
    Json,
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
    log_level: log::LevelFilter,
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
    let defaults = "\
log_level: Info
web:
    bind_address: 0.0.0.0:3000
tenant:
    environment: local
    name: local
location: europe-north1
    ";
    let config: Config = Figment::new()
        .merge(Yaml::string(defaults))
        .merge(Env::prefixed("MUTILATOR__").split("__"))
        .extract()?;
    app(config)?;
    Ok(())
}

#[tokio::main]
async fn app(config: Config) -> Result<()> {
    logging::init_logging(&config)?;
    info!("Configuration loaded: {:?}", config);

    web::start_web_server(config).await?;

    Ok(())
}
