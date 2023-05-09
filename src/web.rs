use anyhow::Result;
use axum::Router;
use axum::routing::get;
use axum_server::tls_rustls::RustlsConfig;
use log::info;

use crate::WebConfig;

pub async fn start_web_server(config: WebConfig) -> Result<()> {
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let addr = config.bind_address.parse().unwrap();
    if config.certificate_path.is_some() && config.private_key_path.is_some() {
        let tls_config = RustlsConfig::from_pem_file(
            config.certificate_path.unwrap(),
            config.private_key_path.unwrap())
            .await?;
        info!("Starting webserver on {} using https", addr);
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        info!("Starting webserver on {} using http", addr);
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await?;
    }

    Ok(())
}
