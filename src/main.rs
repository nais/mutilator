use anyhow::{anyhow, Result};
use tracing::info;

use settings::AppConfig;

mod logging;
mod web;
mod aiven_types;
mod mutators;
mod settings;

fn main() -> Result<()> {
    let config = settings::load_config().map_err(|e| anyhow!(e))?;
    app(config)?;
    Ok(())
}

#[tokio::main]
async fn app(config: AppConfig) -> Result<()> {
    logging::init_logging(&config)?;
    info!("Configuration loaded: {:?}", config);

    web::start_web_server(config).await?;

    Ok(())
}
