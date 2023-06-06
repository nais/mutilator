use anyhow::Result;
use opentelemetry::sdk::export::trace::stdout;
use tracing::{info, Level};
use tracing_subscriber::{filter, Registry};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use crate::{Config, LogFormat};

pub fn init_logging(config: &Config) -> Result<()> {
    let filter = filter::Targets::new()
        .with_default(&config.log_level)
        .with_target("axum::rejection", Level::TRACE);
    let otel_tracer = stdout::new_pipeline().install_simple();
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);
    match config.log_format {
        LogFormat::Plain => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .compact();
            Registry::default()
                .with(otel_layer)
                .with(fmt_layer)
                .with(filter)
                .init();
        }
        LogFormat::Json => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .flatten_event(true);
            Registry::default()
                .with(otel_layer)
                .with(fmt_layer)
                .with(filter)
                .init();
        }
    };
    info!("{:?} logger initialized", config.log_format);
    Ok(())
}
