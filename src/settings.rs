use schematic::{Config, ConfigEnum, ConfigLoader};
use serde::{Deserialize, Serialize};
use std::{io::IsTerminal, path::PathBuf};
use tracing::level_filters::LevelFilter;

#[derive(ConfigEnum, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum LogFormat {
	Plain,
	Json,
}

impl Default for LogFormat {
	fn default() -> Self {
		match std::io::stdout().is_terminal() {
			true => LogFormat::Plain,
			false => LogFormat::Json,
		}
	}
}

#[derive(ConfigEnum, Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum LogLevel {
	Trace = 0,
	Debug = 1,
	#[default]
	Info = 2,
	Warn = 3,
	Error_ = 4,
}

impl Into<LevelFilter> for &LogLevel {
	fn into(self) -> LevelFilter {
		match self {
			LogLevel::Trace => LevelFilter::TRACE,
			LogLevel::Debug => LevelFilter::DEBUG,
			LogLevel::Info => LevelFilter::INFO,
			LogLevel::Warn => LevelFilter::WARN,
			LogLevel::Error_ => LevelFilter::ERROR,
		}
	}
}

#[derive(Config, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[config(env_prefix = "MUTILATOR__WEB__")]
pub struct WebConfig {
	// The address:port to bind to
	#[setting(default = "0.0.0.0:9443", parse_env = schematic::env::ignore_empty)]
	pub bind_address: String,
	// Path to certificate for TLS
	pub certificate_path: Option<PathBuf>,
	// Path to private key for TLS
	pub private_key_path: Option<PathBuf>,
}

#[derive(Config, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[config(env_prefix = "MUTILATOR__TENANT__")]
pub struct Tenant {
	#[setting(default = "local", parse_env = schematic::env::ignore_empty)]
	pub environment: String,
	#[setting(default = "local", parse_env = schematic::env::ignore_empty)]
	pub name: String,
}

#[derive(Config, Debug, Deserialize, Serialize, Clone)]
#[config(env_prefix = "MUTILATOR__")]
pub struct AppConfig {
	// Logging format to use
	#[serde(default)]
	#[setting]
	pub log_format: LogFormat,
	// Log level
	#[serde(default)]
	#[setting]
	pub log_level: LogLevel,
	// Configure web server
	#[setting(nested)]
	pub web: WebConfig,
	// Tenant details
	#[setting(nested)]
	pub tenant: Tenant,
	// Aiven VPC ID
	#[setting(validate = schematic::validate::regex("^[a-z0-9]{8}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{4}-[a-z0-9]{12}"))]
	pub project_vpc_id: String,
	// Cloud location (eq. europe-north1)
	#[setting(default = "europe-north1", parse_env = schematic::env::ignore_empty)]
	pub location: String,
	// Enabled OpenTelemetry collector
	#[setting(default = false, env = "OTEL_EXPORTER_OTLP_ENDPOINT", parse_env = parse_otel)]
	pub otel_enabled: bool,
}

pub fn parse_otel(var: String) -> Result<Option<bool>, schematic::ConfigError> {
	let var = var.trim();

	if var.starts_with("https://") || var.starts_with("http://") {
		Ok(Some(true))
	} else {
		Ok(None)
	}
}

pub fn load_config() -> anyhow::Result<AppConfig> {
	let config_load_result = ConfigLoader::<AppConfig>::new().load()?;
	Ok(config_load_result.config)
}

#[cfg(test)]
mod tests {
	use super::*;
	use envtestkit::lock::lock_test;
	use envtestkit::set_env;
	use pretty_assertions::assert_eq;
	use rstest::*;
	use std::ffi::OsString;

	const BIND_ADDRESS: &'static str = "127.0.0.1:9443";
	const BIND_ADDRESS_KEY: &'static str = "MUTILATOR__WEB__BIND_ADDRESS";
	const LOCATION: &'static str = "my-location";
	const LOCATION_KEY: &'static str = "MUTILATOR__LOCATION";
	const PROJECT_VPC_ID: &'static str = "ba5eba11-dead-bea7-babe-decea5edbabe";
	const PROJECT_VPC_ID_KEY: &'static str = "MUTILATOR__PROJECT_VPC_ID";

	#[rstest]
	#[case::bind_address(BIND_ADDRESS_KEY, BIND_ADDRESS, BIND_ADDRESS)]
	#[case::location_set(LOCATION_KEY, LOCATION, LOCATION)]
	#[case::location_blank(LOCATION_KEY, "europe-north1", "")]
	#[case::project_vpc_id(PROJECT_VPC_ID_KEY, PROJECT_VPC_ID, PROJECT_VPC_ID)]
	pub fn test_load_config(#[case] key: &str, #[case] expected: &str, #[case] value: &str) {
		let _lock = lock_test();
		let _vpc_guard = set_env(OsString::from(PROJECT_VPC_ID_KEY), PROJECT_VPC_ID);
		let _guard = set_env(OsString::from(key), value);

		let config = load_config().unwrap();

		match key {
			LOCATION_KEY => {
				assert_eq!(config.location, expected)
			},
			PROJECT_VPC_ID_KEY => {
				assert_eq!(config.project_vpc_id, expected)
			},
			BIND_ADDRESS_KEY => {
				assert_eq!(config.web.bind_address, expected)
			},
			_ => {
				panic!("Unmatched configuration key in test")
			},
		}
	}

	#[rstest]
	#[should_panic]
	pub fn test_required_fields() {
		let _lock = lock_test();

		let _config = load_config().unwrap();
	}

	#[rstest]
	#[case::enabled("https://localhost:4317", true)]
	#[case::disabled("", false)]
	pub fn test_otel_setting(#[case] value: &str, #[case] expected: bool) {
		let _lock = lock_test();
		let _vpc_guard = set_env(OsString::from(PROJECT_VPC_ID_KEY), PROJECT_VPC_ID);
		let _guard = set_env(
			OsString::from(opentelemetry_otlp::OTEL_EXPORTER_OTLP_ENDPOINT),
			value,
		);

		let config = load_config().unwrap();

		assert_eq!(config.otel_enabled, expected)
	}
}
