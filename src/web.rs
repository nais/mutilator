use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::{debug_handler, Router};
use axum::routing::{get, post};
use axum_server::tls_rustls::RustlsConfig;
use json_patch::{Patch, PatchOperation};
use kube::core::admission::{AdmissionRequest, AdmissionResponse, AdmissionReview, Operation};
use kube::core::DynamicObject;
use kube::{Resource, ResourceExt};
use tracing::{info, instrument, info_span, warn, debug, error};
use serde_json::{json, Value};

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
        debug!("Ignoring operation {:?}", req.operation);
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
                error!("Processing failed: {}", err.to_string());
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

    add_project_vpc_id(config.project_vpc_id.clone(), obj, &mut patches);
    add_termination_protection(obj, &mut patches);
    add_tags(config, obj, &mut patches);

    Ok(res.with_patch(Patch(patches))?)
}

#[instrument(skip_all)]
fn add_tags(config: &Arc<Config>, obj: &Redis, patches: &mut Vec<PatchOperation>) {
    let environment = Value::String(config.tenant.environment.clone());
    let tenant = Value::String(config.tenant.name.clone());
    let team = Value::String(obj.meta().namespace.as_ref().unwrap().clone());
    if obj.spec.tags.is_none() {
        info!("Adding tags map");
        patches.push(add_patch("/spec/tags".into(), json!("{}")));
        info!("Adding environment tag: {}", environment);
        patches.push(add_patch("/spec/tags/environment".into(), environment));
        info!("Adding tenant tag: {}", tenant);
        patches.push(add_patch("/spec/tags/tenant".into(), tenant));
        info!("Adding team tag: {}", team);
        patches.push(add_patch("/spec/tags/team".into(), team));
    } else {
        let tags = obj.spec.tags.as_ref().unwrap();
        if tags.contains_key("environment") {
            info!("Overwriting environment tag: {}", environment);
            patches.push(replace_patch("/spec/tags/environment".into(), environment));
        } else {
            info!("Adding environment tag: {}", environment);
            patches.push(add_patch("/spec/tags/environment".into(), environment));
        }
        if tags.contains_key("tenant") {
            info!("Overwriting tenant tag: {}", tenant);
            patches.push(replace_patch("/spec/tags/tenant".into(), tenant));
        } else {
            info!("Adding tenant tag: {}", tenant);
            patches.push(add_patch("/spec/tags/tenant".into(), tenant));
        }
        if tags.contains_key("team") {
            info!("Overwriting team tag: {}", team);
            patches.push(replace_patch("/spec/tags/team".into(), team));
        } else {
            info!("Adding team tag: {}", team);
            patches.push(add_patch("/spec/tags/team".into(), team));
        }
    }
}

#[instrument(skip_all)]
fn add_termination_protection(obj: &Redis, patches: &mut Vec<PatchOperation>) {
    if obj.spec.termination_protection.is_none() {
        info!("Enabling terminationProtection");
        patches.push(add_patch("/spec/terminationProtection".into(), Value::Bool(true)));
    }
}

#[instrument(skip_all)]
fn add_project_vpc_id(project_vpc_id: String, obj: &Redis, patches: &mut Vec<PatchOperation>) {
    if obj.spec.project_vpc_id.is_none() {
        info!("Adding projectVpcId");
        patches.push(add_patch("/spec/projectVpcId".into(), Value::String(project_vpc_id)));
    }
}

fn add_patch(path: String, value: Value) -> PatchOperation {
    PatchOperation::Add(json_patch::AddOperation {
        path,
        value,
    })
}

fn replace_patch(path: String, value: Value) -> PatchOperation {
    PatchOperation::Replace(json_patch::ReplaceOperation {
        path,
        value,
    })
}