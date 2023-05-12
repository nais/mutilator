use anyhow::Result;
use axum::extract::Json;
use axum::http::StatusCode;
use axum::Router;
use axum::routing::{get, post};
use axum_server::tls_rustls::RustlsConfig;
use kube::core::admission::{AdmissionResponse, AdmissionReview};
use kube::core::DynamicObject;
use kube::ResourceExt;
use log::{info, warn};

use crate::aiven_types::aiven_redis::Redis;
use crate::WebConfig;

pub async fn start_web_server(config: WebConfig) -> Result<()> {
    let app = Router::new()
        .route("/is_alive", get(|| async { "I'm alive!" }))
        .route("/is_ready", get(|| async { "Ready for action!" }))
        .route("/mutate", post(mutate_handler))
        ;

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

async fn mutate_handler(Json(admission_review): Json<AdmissionReview<Redis>>) -> (StatusCode, Json<AdmissionReview<DynamicObject>>) {
    match admission_review.request {
        None => {
            warn!("No request present in AdmissionReview object");
            (StatusCode::BAD_REQUEST, Json(AdmissionResponse::invalid("missing request").into_review()))
        }
        Some(req) => {
            let mut res = AdmissionResponse::from(&req);
            if let Some(obj) = &req.object {
                let name = obj.name_any();
                let namespace = obj.namespace().unwrap();
                info!(name = name, namespace = namespace; "Processing redis resource");
                res = match mutate(res.clone(), &obj) {
                    Ok(res) => {
                        info!(name = name, namespace = namespace; "Processing complete");
                        res
                    },
                    Err(err) => {
                        warn!(name = name, namespace = namespace; "Processing failed");
                        res.deny(err.to_string())
                    }
                };
                (StatusCode::OK, Json(res.into_review()))
            } else {
                (StatusCode::BAD_REQUEST, Json(AdmissionResponse::invalid("no object specified").into_review()))
            }
        }
    }
}

fn mutate(res: AdmissionResponse, obj: &Redis) -> Result<AdmissionResponse> {
    Ok(res)
}