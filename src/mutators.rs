use std::sync::Arc;

use json_patch::PatchOperation;
use kube::Resource;
use serde_json::{json, Value};
use tracing::{info, instrument};

use crate::aiven_types::aiven_redis::Redis;
use crate::Config;

#[instrument(skip_all)]
pub fn add_location(location: String, obj: &Redis, patches: &mut Vec<PatchOperation>) {
    let cloud_name = Value::String(format!("google-{}", location));
    if obj.spec.cloud_name.is_none() {
        info!("Adding cloudName");
        patches.push(add_patch("/spec/cloudName".into(), cloud_name));
    } else {
        info!("Overwriting cloudName");
        patches.push(replace_patch("/spec/cloudName".into(), cloud_name));
    }
}

#[instrument(skip_all)]
pub fn add_tags(config: &Arc<Config>, obj: &Redis, patches: &mut Vec<PatchOperation>) {
    let environment = Value::String(config.tenant.environment.clone());
    let tenant = Value::String(config.tenant.name.clone());
    let team = Value::String(obj.meta().namespace.as_ref().unwrap().clone());
    if obj.spec.tags.is_none() {
        info!("Adding tags");
        patches.push(add_patch("/spec/tags".into(), json!({
            "environment": environment,
            "tenant": tenant,
            "team": team
        })));
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
pub fn add_termination_protection(obj: &Redis, patches: &mut Vec<PatchOperation>) {
    if obj.spec.termination_protection.is_none() {
        info!("Enabling terminationProtection");
        patches.push(add_patch("/spec/terminationProtection".into(), Value::Bool(true)));
    }
}

#[instrument(skip_all)]
pub fn add_project_vpc_id(project_vpc_id: String, obj: &Redis, patches: &mut Vec<PatchOperation>) {
    if obj.spec.project_vpc_id.is_none() {
        info!("Adding projectVpcId");
        patches.push(add_patch("/spec/projectVpcId".into(), Value::String(project_vpc_id)));
    }
}

#[instrument(skip_all)]
pub fn add_plan(plan: String, obj: &Redis, patches: &mut Vec<PatchOperation>) {
    if obj.spec.plan.is_none() {
        info!("Adding plan");
        patches.push(add_patch("/spec/plan".into(), Value::String(plan)));
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
