use anyhow::Result;
use axum::{
    Router,
    routing::get,
};
use figment::{Figment, providers::Env};
use figment::providers::Serialized;
use serde::{Deserialize, Serialize};
use atty::Stream;

mod logging;

#[derive(Debug, Deserialize, Serialize)]
pub enum LogFormat {
    Plain,
    Json,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    // The address:port to bind to
    bind_address: String,
    // Logging format to use
    log_format: LogFormat,
    // Log level
    log_level: log::LevelFilter
}

impl Default for Config {
    fn default() -> Config {
        let log_format = match atty::is(Stream::Stdout) {
            true => LogFormat::Plain,
            false => LogFormat::Json,
        };
        Config {
            bind_address: "0.0.0.0:3000".into(),
            log_level: log::LevelFilter::Info,
            log_format,
        }
    }
}

fn main() -> Result<()> {
    let config: Config = Figment::from(Serialized::defaults(Config::default()))
        .merge(Env::prefixed("MUTILATOR_"))
        .extract()?;
    app(config)?;
    Ok(())
}

#[tokio::main]
async fn app(config: Config) -> Result<()> {
    logging::init_logging(&config)?;

    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&config.bind_address.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
