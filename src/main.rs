use anyhow::Result;
use axum::{
    Router,
    routing::get,
};

fn main() -> Result<()> {
    app()?;
    Ok(())
}

#[tokio::main]
async fn app() -> Result<()> {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}