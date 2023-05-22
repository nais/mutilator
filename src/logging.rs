use std::io::Write;

use anyhow::Result;
use json::JsonValue;
use log::{info, LevelFilter};
use log::kv::{Error, Key, Value};

use crate::{Config, LogFormat};

struct JsonVisitor<'a> {
    data: &'a mut JsonValue
}

impl JsonVisitor<'_> {
    fn new(data: &mut JsonValue) -> JsonVisitor {
        return JsonVisitor {
            data,
        };
    }
}

impl<'kvs> log::kv::Visitor<'kvs> for JsonVisitor<'kvs> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> std::result::Result<(), Error> {
        self.data[key.to_string()] = JsonValue::from(value.to_string());
        Ok(())
    }
}


pub fn init_logging(config: &Config) -> Result<()> {
    match config.log_format {
        LogFormat::Plain => {
            env_logger::builder()
                .filter_level(config.log_level)
                .filter_module("axum::rejection", LevelFilter::Trace)
                .target(env_logger::Target::Stdout)
                .default_format()
                .init();
        }
        LogFormat::Json => {
            env_logger::builder()
                .filter_level(config.log_level)
                .filter_module("axum::rejection", LevelFilter::Trace)
                .target(env_logger::Target::Stdout)
                .format(|buf, record| {
                    let mut data = json::object! {
                        timestamp: buf.timestamp_seconds().to_string(),
                        level: record.level().to_string(),
                        message: record.args().to_string(),
                    };
                    let mut visitor = JsonVisitor::new(&mut data);
                    record.key_values().visit(&mut visitor).unwrap();
                    writeln!(buf, "{}", json::stringify(data))
                })
                .init();
        }
    };
    info!("{:?} logger initialized", config.log_format);
    Ok(())
}
