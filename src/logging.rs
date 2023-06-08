use std::env;
use anyhow::Result;
use opentelemetry::KeyValue;
use opentelemetry::sdk::Resource;
use opentelemetry::sdk::trace::Tracer;
use opentelemetry_otlp::{HasExportConfig, WithExportConfig};
use opentelemetry_semantic_conventions::resource;
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
    let (otel_layer, otel_log_msg) = init_otel()?;
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
    info!("{}", otel_log_msg);
    Ok(())
}

fn init_otel<'a>() -> Result<(OpenTelemetryLayer<Registry, Tracer>, String)> {
    let mut otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_env();
    let otel_log_msg = format!("OpenTelemetry export config: {:?}", otlp_exporter.export_config());
    let otel_tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(
            opentelemetry::sdk::trace::config().with_resource(Resource::new(vec![
                KeyValue::new(resource::K8S_CLUSTER_NAME, env::var("NAIS_CLUSTER_NAME").unwrap_or("UNKNOWN_CLUSTER".to_string())),
                KeyValue::new(resource::K8S_NAMESPACE_NAME, env::var("NAIS_NAMESPACE").unwrap_or("UNKNOWN_NAMESPACE".to_string())),
                KeyValue::new(resource::K8S_DEPLOYMENT_NAME, env::var("NAIS_APP_NAME").unwrap_or("UNKNOWN_DEPLOYMENT".to_string())),
                KeyValue::new(resource::SERVICE_NAME, env!("CARGO_BIN_NAME").to_string()),
            ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);
    Ok((otel_layer, otel_log_msg))
}
