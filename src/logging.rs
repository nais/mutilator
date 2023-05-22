use anyhow::Result;
use tracing::{info, Level};
use tracing_subscriber::filter;
use tracing_subscriber::prelude::*;

use crate::{Config, LogFormat};

pub fn init_logging(config: &Config) -> Result<()> {
    let filter = filter::Targets::new()
        .with_default(&config.log_level)
        .with_target("axum::rejection", Level::TRACE);
    match config.log_format {
        LogFormat::Plain => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .compact();
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Json => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .flatten_event(true);
            tracing_subscriber::registry()
                .with(fmt_layer)
                .with(filter)
                .init();
        }
    };
    info!("{:?} logger initialized", config.log_format);
    Ok(())
}
