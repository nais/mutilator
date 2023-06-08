use anyhow::Result;
use opentelemetry::sdk::trace::Tracer;
use opentelemetry_otlp::WithExportConfig;
use tracing::{info, Level};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{filter, Registry};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use crate::{Config, LogFormat};

pub fn init_logging(config: &Config) -> Result<()> {
    let filter = filter::Targets::new()
        .with_default(&config.log_level)
        .with_target("axum::rejection", Level::TRACE);
    let otel_layer = init_otel()?;
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

fn init_otel() -> Result<OpenTelemetryLayer<Registry, Tracer>> {
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_env();
    let otel_tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .install_batch(opentelemetry::runtime::Tokio)?;
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);
    Ok(otel_layer)
}
