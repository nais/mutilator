use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::{debug_handler, Router};
use axum::routing::{get, post};
use axum_server::tls_rustls::RustlsConfig;
use json_patch::Patch;
use kube::core::admission::{AdmissionRequest, AdmissionResponse, AdmissionReview, Operation};
use kube::core::DynamicObject;
use kube::{Resource, ResourceExt};
use tracing::{info, instrument, info_span, warn};
use serde_json::Value;

use crate::aiven_types::aiven_redis::Redis;
use crate::Config;

#[instrument(skip_all)]
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

#[debug_handler]
#[instrument(skip_all)]
async fn mutate_handler(State(config): State<Arc<Config>>, Json(admission_review): Json<AdmissionReview<Redis>>) -> (StatusCode, Json<AdmissionReview<DynamicObject>>) {
    let req: AdmissionRequest<Redis> = match admission_review.try_into() {
        Ok(req) => req,
        Err(err) => {
            warn!("Unable to get request from AdmissionReview: {}", err.to_string());
            return (StatusCode::BAD_REQUEST, Json(AdmissionResponse::invalid("missing request").into_review()));
        }
    };

    let uid = req.uid.clone();
    let mut res = AdmissionResponse::from(&req);
    let req_span = info_span!("request", uid);
    let _req_guard = req_span.enter();

    if req.operation != Operation::Create {
        info!("Ignoring operation {:?}", req.operation);
        return (StatusCode::OK, Json(res.into_review()))
    }

    if let Some(obj) = &req.object {
        let name = obj.name_any();
        let namespace = obj.namespace().unwrap();
        let redis_span = info_span!("redis", name, namespace);
        let _redis_guard = redis_span.enter();
        info!("Processing redis resource");
        res = match mutate(res.clone(), &obj, &config) {
            Ok(res) => {
                info!("Processing complete");
                res
            }
            Err(err) => {
                warn!("Processing failed: {}", err.to_string());
                res.deny(err.to_string())
            }
        };
        (StatusCode::OK, Json(res.into_review()))
    } else {
        warn!("No object specified in AdmissionRequest: {:?}", req);
        (StatusCode::BAD_REQUEST, Json(AdmissionResponse::invalid("no object specified").into_review()))
    }
}

#[instrument(skip_all)]
fn mutate(res: AdmissionResponse, obj: &Redis, config: &Arc<Config>) -> Result<AdmissionResponse> {
    let mut patches = Vec::new();
    if obj.spec.project_vpc_id.is_none() {
        info!("Adding project_vpc_id");
        patches.push(json_patch::PatchOperation::Add(json_patch::AddOperation {
            path: "/spec/project_vpc_id".into(),
            value: Value::String(config.project_vpc_id.clone()),
        }));
    }

    // Test if patches work at all
    // Ensure labels exist before adding a key to it
    if obj.meta().labels.is_none() {
        patches.push(json_patch::PatchOperation::Add(json_patch::AddOperation {
            path: "/metadata/labels".into(),
            value: serde_json::json!({}),
        }));
    }
    // Add our label
    patches.push(json_patch::PatchOperation::Add(json_patch::AddOperation {
        path: "/metadata/labels/admission".into(),
        value: serde_json::Value::String("modified-by-admission-controller".into()),
    }));
    Ok(res.with_patch(Patch(patches))?)
}