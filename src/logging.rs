use anyhow::Result;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource;
use std::env;
use tracing::{info, Level};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{filter, Registry};

use crate::settings::{AppConfig, LogFormat};

pub fn init_logging(config: &AppConfig) -> Result<()> {
	let otel_layer = init_otel(config.otel_enabled)?;

	use tracing_subscriber::fmt as layer_fmt;
	let (plain_log_format, json_log_format) = match config.log_format {
		LogFormat::Plain => (Some(layer_fmt::layer().compact()), None),
		LogFormat::Json => (None, Some(layer_fmt::layer().json().flatten_event(true))),
	};

	Registry::default()
		.with(otel_layer)
		.with(plain_log_format)
		.with(json_log_format)
		.with(
			filter::Targets::new()
				.with_default(&config.log_level)
				.with_target("axum::rejection", Level::TRACE),
		)
		.init();
	info!("{:?} logger initialized", config.log_format);

	Ok(())
}

fn init_otel(enable: bool) -> Result<Option<OpenTelemetryLayer<Registry, Tracer>>> {
	if enable == false {
		return Ok(None);
	}

	let exporter = SpanExporter::builder().with_tonic().build()?;
	let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
		.with_resource(Resource::new(vec![
			KeyValue::new(
				resource::K8S_CLUSTER_NAME,
				env::var("NAIS_CLUSTER_NAME").unwrap_or("UNKNOWN_CLUSTER".to_string()),
			),
			KeyValue::new(
				resource::K8S_NAMESPACE_NAME,
				env::var("NAIS_NAMESPACE").unwrap_or("UNKNOWN_NAMESPACE".to_string()),
			),
			KeyValue::new(
				resource::K8S_DEPLOYMENT_NAME,
				env::var("NAIS_APP_NAME").unwrap_or("UNKNOWN_DEPLOYMENT".to_string()),
			),
			KeyValue::new(resource::SERVICE_NAME, env!("CARGO_BIN_NAME").to_string()),
		]))
		.with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
		.build();
	let otel_tracer = tracer_provider.tracer("mutilator");
	let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);
	Ok(Some(otel_layer))
}
