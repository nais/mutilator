use std::path::PathBuf;
use anyhow::Result;
use figment::{Figment, providers::Env};
use figment::providers::Serialized;
use serde::{Deserialize, Serialize};
use atty::Stream;
use log::info;

mod logging;
mod web;
mod aiven_types;

#[derive(Debug, Deserialize, Serialize)]
pub enum LogFormat {
    Plain,
    Json,
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
pub struct Config {
    // Logging format to use
    log_format: LogFormat,
    // Log level
    log_level: log::LevelFilter,

    web: WebConfig,
}

impl Default for Config {
    fn default() -> Config {
        let log_format = match atty::is(Stream::Stdout) {
            true => LogFormat::Plain,
            false => LogFormat::Json,
        };
        Config {
            log_level: log::LevelFilter::Info,
            log_format,
            web: WebConfig {
                bind_address: "0.0.0.0:3000".into(),
                certificate_path: None,
                private_key_path: None,
            }
        }
    }
}

fn main() -> Result<()> {
    let config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Env::prefixed("MUTILATOR__").split("__"))
        .extract()?;
    app(config)?;
    Ok(())
}

#[tokio::main]
async fn app(config: Config) -> Result<()> {
    logging::init_logging(&config)?;
    info!("Configuration loaded: {:?}", config);

    web::start_web_server(config.web).await?;

    Ok(())
}
