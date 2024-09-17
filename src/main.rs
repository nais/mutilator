use anyhow::{anyhow, Result};
use rustls::crypto;
use tracing::info;

use settings::AppConfig;

mod aiven_object;
mod logging;
mod mutators;
mod settings;
mod web;

fn main() -> Result<()> {
	let config = settings::load_config().map_err(|e| anyhow!(e))?;
	app(config)?;
	Ok(())
}

#[tokio::main]
async fn app(config: AppConfig) -> Result<()> {
	crypto::ring::default_provider()
		.install_default()
		.expect("Failed to install default crypto provider");
	logging::init_logging(&config)?;
	info!("Configuration loaded: {:?}", config);

	web::start_web_server(config).await?;

	Ok(())
}
