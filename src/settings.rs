use serde::{Deserialize, Serialize};
use atty::Stream;
use std::path::PathBuf;
use figment::Figment;
use figment::providers::{Env, Format, Yaml};
use tracing::level_filters::LevelFilter;

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
    pub bind_address: String,
    // Path to certificate for TLS
    pub certificate_path: Option<PathBuf>,
    // Path to private key for TLS
    pub private_key_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tenant {
    pub environment: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // Logging format to use
    #[serde(default)]
    pub log_format: LogFormat,
    // Log level
    pub log_level: LogLevel,
    // Configure web server
    pub web: WebConfig,
    // Tenant details
    pub tenant: Tenant,
    // Aiven VPC ID
    pub project_vpc_id: String,
    // Cloud location (eq. europe-north1)
    pub location: String,
}

pub fn load_config() -> anyhow::Result<Config, figment::Error> {
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

#[cfg(test)]
mod tests {
    use figment::Jail;
    use pretty_assertions::assert_eq;
    use rstest::*;
    use super::*;

    const LOCATION: &'static str = "my-location";
    const PROJECT_VPC_ID: &'static str = "my-vpc-id";

    #[rstest]
    pub fn test_load_config() {
        Jail::expect_with(|jail| {
            jail.set_env("MUTILATOR__LOCATION", LOCATION);
            jail.set_env("MUTILATOR__PROJECT_VPC_ID", PROJECT_VPC_ID);

            let config = load_config()?;

            assert_eq!(config.location, LOCATION);
            assert_eq!(config.project_vpc_id, PROJECT_VPC_ID);

            Ok(())
        })
    }
}
