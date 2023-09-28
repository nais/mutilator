use std::collections::BTreeMap;
use std::sync::Arc;

use json_patch::PatchOperation;
use serde_json::{json, Value};
use tracing::{debug, info, instrument};

use crate::aiven_types::AivenObject;
use crate::settings::AppConfig;

#[instrument(skip_all)]
pub fn add_location(location: String, obj: &dyn AivenObject, patches: &mut Vec<PatchOperation>) {
	let cloud_name = Value::String(format!("google-{}", location));
	if obj.get_cloud_name().is_none() {
		info!("Adding cloudName");
		patches.push(add_patch(obj.cloud_name_path(), cloud_name));
	} else {
		info!("Overwriting cloudName");
		patches.push(replace_patch("/spec/cloudName".into(), cloud_name));
	}
}

#[instrument(skip_all)]
pub fn add_tags(config: &Arc<AppConfig>, obj: &dyn AivenObject, patches: &mut Vec<PatchOperation>) {
	let environment = config.tenant.environment.clone();
	let tenant = config.tenant.name.clone();
	let team = obj.get_team_name().unwrap();
	if obj.get_tags().is_none() {
		info!("Adding tags");
		patches.push(add_patch(
			obj.tags_path(),
			json!({
				"environment": environment,
				"tenant": tenant,
				"team": team
			}),
		));
	} else {
		let tags = obj.get_tags().unwrap();
		for (tag_name, tag_value) in [
			("environment", environment.clone()),
			("tenant", tenant.clone()),
			("team", team.clone()),
		] {
			if let Some(patch) = handle_tag(&tags, tag_name, tag_value, obj.tag_path(tag_name)) {
				patches.push(patch);
			}
		}
	}
}

fn handle_tag(
	tags: &BTreeMap<String, String>,
	tag_name: &str,
	tag_value: String,
	tag_path: String,
) -> Option<PatchOperation> {
	match tags.get(tag_name) {
		Some(value) if value.as_str() == tag_value => {
			debug!("{} tag already set to {}", tag_name, value);
			None
		},
		Some(value) => {
			info!("Overwriting {} tag: {} => {}", tag_name, value, tag_value);
			Some(replace_patch(tag_path, Value::String(tag_value)))
		},
		None => {
			info!("Adding {} tag: {}", tag_name, tag_value);
			Some(add_patch(tag_path, Value::String(tag_value)))
		},
	}
}

#[instrument(skip_all)]
pub fn add_termination_protection(obj: &dyn AivenObject, patches: &mut Vec<PatchOperation>) {
	if obj.get_termination_protection().is_none() {
		info!("Enabling terminationProtection");
		patches.push(add_patch(
			obj.termination_protection_path(),
			Value::Bool(true),
		));
	}
}

#[instrument(skip_all)]
pub fn add_project_vpc_id(
	project_vpc_id: String,
	obj: &dyn AivenObject,
	patches: &mut Vec<PatchOperation>,
) {
	if obj.get_project_vpc_id().is_none() {
		info!("Adding projectVpcId");
		patches.push(add_patch(
			obj.project_vpc_id_path(),
			Value::String(project_vpc_id),
		));
	}
}

fn add_patch(path: String, value: Value) -> PatchOperation {
	PatchOperation::Add(json_patch::AddOperation { path, value })
}

fn replace_patch(path: String, value: Value) -> PatchOperation {
	PatchOperation::Replace(json_patch::ReplaceOperation { path, value })
}

#[cfg(test)]
mod tests {
	use std::collections::{BTreeMap, BTreeSet};

	use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
	use pretty_assertions::assert_eq;
	use rstest::*;

	use crate::aiven_types::aiven_redis::Redis;
	use crate::aiven_types::aiven_redis::RedisSpec;
	use crate::settings::{LogLevel, Tenant, WebConfig};

	use super::*;

	const ENVIRONMENT: &str = "test-tenant-env";
	const TENANT: &str = "test-tenant-name";
	const NAMESPACE: &str = "test-namespace";
	const PROJECT_VPC_ID: &str = "test-vpc-id";
	const LOCATION: &str = "test-location";

	const TAG_PAIRS: [(&str, &str); 3] = [
		("environment", ENVIRONMENT),
		("tenant", TENANT),
		("team", NAMESPACE),
	];

	#[fixture]
	pub fn config() -> Arc<AppConfig> {
		Arc::new(AppConfig {
			log_format: Default::default(),
			log_level: LogLevel::Trace,
			web: WebConfig {
				bind_address: "".to_string(),
				certificate_path: None,
				private_key_path: None,
			},
			tenant: Tenant {
				environment: ENVIRONMENT.to_string(),
				name: TENANT.to_string(),
			},
			project_vpc_id: PROJECT_VPC_ID.to_string(),
			location: LOCATION.to_string(),
			otel_enabled: false,
		})
	}

	#[rstest]
	fn add_tags_when_no_tags_before(config: Arc<AppConfig>) {
		let redis = create_redis(None);
		let mut patches = Vec::new();

		add_tags(&config, &redis, &mut patches);

		assert_eq!(patches.len(), 1);
		let patch = patches.pop().unwrap();
		match patch {
			PatchOperation::Add(add) => {
				assert_eq!(add.path, "/spec/tags");
				assert_eq!(
					add.value,
					json!({
						"environment": ENVIRONMENT.to_string(),
						"team": NAMESPACE.to_string(),
						"tenant": TENANT.to_string(),
					}),
					"incorrect json structure added"
				);
			},
			_ => {
				panic!("Wrong patch operation in patches");
			},
		}
	}

	#[rstest]
	fn add_tags_when_other_tags_exists(config: Arc<AppConfig>) {
		let redis = create_redis(Some(BTreeMap::from([(
			"cool".to_string(),
			"tag".to_string(),
		)])));
		let mut patches: Vec<PatchOperation> = Vec::new();

		add_tags(&config, &redis, &mut patches);
		let actual = make_comparable_set(&mut patches);

		let mut expected = BTreeSet::new();
		for (key, value) in TAG_PAIRS {
			expected.insert(("add", format!("/spec/tags/{}", key), value.to_string()));
		}
		assert_eq!(actual, expected, "contains expected patches");
	}

	#[rstest]
	fn replace_tags_when_wrong_values_are_set(config: Arc<AppConfig>) {
		let mut existing_tags = BTreeMap::new();
		for (key, _value) in TAG_PAIRS {
			existing_tags.insert(key.to_string(), "invalid".to_string());
		}
		let redis = create_redis(Some(existing_tags));
		let mut patches: Vec<PatchOperation> = Vec::new();

		add_tags(&config, &redis, &mut patches);
		let actual = make_comparable_set(&mut patches);

		let mut expected = BTreeSet::new();
		for (key, value) in TAG_PAIRS {
			expected.insert(("replace", format!("/spec/tags/{}", key), value.to_string()));
		}
		assert_eq!(actual, expected, "contains expected patches");
	}

	#[rstest]
	#[case(ENVIRONMENT, TENANT)]
	#[case(ENVIRONMENT, NAMESPACE)]
	#[case(TENANT, NAMESPACE)]
	#[case(TENANT, ENVIRONMENT)]
	#[case(NAMESPACE, ENVIRONMENT)]
	#[case(NAMESPACE, TENANT)]
	fn add_or_replace_as_needed(
		config: Arc<AppConfig>,
		#[case] correct: &str,
		#[case] invalid: &str,
	) {
		let mut existing_tags = BTreeMap::new();
		for (key, value) in TAG_PAIRS {
			match value {
				c if c == correct => {
					existing_tags.insert(key.to_string(), correct.to_string());
				},
				i if i == invalid => {
					existing_tags.insert(key.to_string(), "invalid".to_string());
				},
				_ => {},
			}
		}
		let redis = create_redis(Some(existing_tags));
		let mut patches: Vec<PatchOperation> = Vec::new();

		add_tags(&config, &redis, &mut patches);
		let actual = make_comparable_set(&mut patches);

		let mut expected = BTreeSet::new();
		for (key, value) in TAG_PAIRS {
			match value {
				v if v == correct => {},
				v if v == invalid => {
					expected.insert(("replace", format!("/spec/tags/{}", key), value.to_string()));
				},
				_ => {
					expected.insert(("add", format!("/spec/tags/{}", key), value.to_string()));
				},
			}
		}
		assert_eq!(actual, expected, "contains expected patches");
	}

	fn make_comparable_set(patches: &mut Vec<PatchOperation>) -> BTreeSet<(&str, String, String)> {
		patches
			.clone()
			.into_iter()
			.map(|p| match p {
				PatchOperation::Add(add) => {
					("add", add.path, add.value.as_str().unwrap().to_string())
				},
				PatchOperation::Replace(replace) => (
					"replace",
					replace.path,
					replace.value.as_str().unwrap().to_string(),
				),
				_ => ("invalid", "invalid".to_string(), String::new()),
			})
			.collect()
	}

	fn create_redis(tags: Option<BTreeMap<String, String>>) -> Redis {
		Redis {
			metadata: ObjectMeta {
				annotations: None,
				cluster_name: None,
				creation_timestamp: None,
				deletion_grace_period_seconds: None,
				deletion_timestamp: None,
				finalizers: None,
				generate_name: None,
				generation: None,
				labels: None,
				managed_fields: None,
				name: Some("test-name".to_string()),
				namespace: Some("test-namespace".to_string()),
				owner_references: None,
				resource_version: None,
				self_link: None,
				uid: None,
			},
			spec: RedisSpec {
				auth_secret_ref: None,
				cloud_name: None,
				conn_info_secret_target: None,
				disk_space: None,
				maintenance_window_dow: None,
				maintenance_window_time: None,
				plan: "test-plan".to_string(),
				project: "test-project".to_string(),
				project_vpc_ref: None,
				project_vpc_id: None,
				service_integrations: None,
				tags,
				termination_protection: None,
				user_config: None,
			},
			status: None,
		}
	}
}
