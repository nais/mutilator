use anyhow::Result;
use axum::{
    Router,
    routing::get,
};
use figment::{Figment, providers::Env};
use figment::providers::Serialized;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    // The address:port to bind to
    bind_address: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            bind_address: "0.0.0.0:3000".into(),
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
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&config.bind_address.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}