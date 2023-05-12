use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::Router;
use axum::routing::{get, post};
use axum_server::tls_rustls::RustlsConfig;
use kube::core::admission::{AdmissionResponse, AdmissionReview};
use kube::core::DynamicObject;
use kube::ResourceExt;
use log::{info, warn};
use serde_json::Value;

use crate::aiven_types::aiven_redis::Redis;
use crate::Config;

pub async fn start_web_server(config: Config) -> Result<()> {
    let certificate_path = config.web.certificate_path.clone();
    let private_key_path = config.web.private_key_path.clone();
    let addr = config.web.bind_address.parse().unwrap();

    let state = Arc::new(config);
    let app = Router::new()
        .route("/is_alive", get(|| async { "I'm alive!" }))
        .route("/is_ready", get(|| async { "Ready for action!" }))
        .route("/mutate", post(mutate_handler))
        .with_state(state)
        ;

    if certificate_path.is_some() && private_key_path.is_some() {
        let tls_config = RustlsConfig::from_pem_file(
            certificate_path.unwrap(),
            private_key_path.unwrap())
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

async fn mutate_handler(State(config): State<Arc<Config>>, Json(admission_review): Json<AdmissionReview<Redis>>) -> (StatusCode, Json<AdmissionReview<DynamicObject>>) {
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
                res = match mutate(res.clone(), &obj, &config) {
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

fn mutate(res: AdmissionResponse, obj: &Redis, config: &Arc<Config>) -> Result<AdmissionResponse> {
    let mut patches = Vec::new();
    if obj.spec.project_vpc_id.is_none() {
        patches.push(json_patch::PatchOperation::Add(json_patch::AddOperation {
            path: "/spec/project_vpc_id".to_string(),
            value: Value::from(config.project_vpc_id.clone()),
        }))
    }
    Ok(res.with_patch(json_patch::Patch(patches))?)
}