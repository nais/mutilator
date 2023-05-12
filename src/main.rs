use std::path::PathBuf;

use anyhow::Result;
use atty::Stream;
use figment::{Figment, providers::Env};
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

impl Default for WebConfig {
    fn default() -> Self {
        WebConfig {
            bind_address: "0.0.0.0:3000".into(),
            certificate_path: None,
            private_key_path: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tenant {
    environment: String,
    name: String,
}

impl Default for Tenant {
    fn default() -> Self {
        Tenant {
            environment: "local".to_string(),
            name: "local".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // Logging format to use
    #[serde(default)]
    log_format: LogFormat,
    // Log level
    #[serde(default = "default_log_level")]
    log_level: log::LevelFilter,
    #[serde(default)]
    web: WebConfig,
    #[serde(default)]
    tenant: Tenant,

    // Aiven VPC ID
    project_vpc_id: String,

    // Cloud location (eq. europe-north1)
    location: String,
}

fn default_log_level() -> log::LevelFilter {
    log::LevelFilter::Info
}

fn main() -> Result<()> {
    let config: Config = Figment::new()
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
