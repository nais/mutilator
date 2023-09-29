use kube::core::DynamicObject;
use std::collections::BTreeMap;

use crate::aiven_types::aiven_opensearches::OpenSearch;
use kube::Resource;

use crate::aiven_types::aiven_redis::Redis;

pub mod aiven_opensearches;
pub mod aiven_redis;

pub trait AivenObject {
	fn get_cloud_name(&self) -> Option<String>;
	fn cloud_name_path(&self) -> String {
		"/spec/cloudName".into()
	}

	fn get_team_name(&self) -> Option<String>;
	fn get_tags(&self) -> Option<BTreeMap<String, String>>;
	fn tags_path(&self) -> String {
		"/spec/tags".into()
	}
	fn tag_path(&self, tag_name: &str) -> String {
		format!("{}/{}", self.tags_path(), tag_name)
	}

	fn get_termination_protection(&self) -> Option<bool>;
	fn termination_protection_path(&self) -> String {
		"/spec/terminationProtection".into()
	}

	fn get_project_vpc_id(&self) -> Option<String>;
	fn project_vpc_id_path(&self) -> String {
		"/spec/projectVpcId".into()
	}
}

impl AivenObject for DynamicObject {
	fn get_cloud_name(&self) -> Option<String> {
		self.data["spec"]["cloudName"]
			.as_str()
			.map(|s| s.to_string())
	}

	fn get_team_name(&self) -> Option<String> {
		self.metadata.namespace.clone()
	}

	fn get_tags(&self) -> Option<BTreeMap<String, String>> {
		self.data["spec"]["tags"].as_object().map(|o| {
			o.iter()
				.map(|(k, v)| (k.to_owned(), v.as_str().unwrap_or("").to_string()))
				.collect()
		})
	}

	fn get_termination_protection(&self) -> Option<bool> {
		self.data["spec"]["terminationProtection"]
			.as_bool()
			.map(|b| b.to_owned())
	}

	fn get_project_vpc_id(&self) -> Option<String> {
		self.data["spec"]["projectVpcId"]
			.as_str()
			.map(|s| s.to_string())
	}
}

impl AivenObject for Redis {
	fn get_cloud_name(&self) -> Option<String> {
		self.spec.cloud_name.clone()
	}

	fn get_team_name(&self) -> Option<String> {
		self.meta().namespace.clone()
	}

	fn get_tags(&self) -> Option<BTreeMap<String, String>> {
		self.spec.tags.clone()
	}

	fn get_termination_protection(&self) -> Option<bool> {
		self.spec.termination_protection.clone()
	}

	fn get_project_vpc_id(&self) -> Option<String> {
		self.spec.project_vpc_id.clone()
	}
}

impl AivenObject for OpenSearch {
	fn get_cloud_name(&self) -> Option<String> {
		self.spec.cloud_name.clone()
	}

	fn get_team_name(&self) -> Option<String> {
		self.meta().namespace.clone()
	}

	fn get_tags(&self) -> Option<BTreeMap<String, String>> {
		self.spec.tags.clone()
	}

	fn get_termination_protection(&self) -> Option<bool> {
		self.spec.termination_protection.clone()
	}

	fn get_project_vpc_id(&self) -> Option<String> {
		self.spec.project_vpc_id.clone()
	}
}
