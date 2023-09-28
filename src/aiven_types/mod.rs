use std::collections::BTreeMap;

use kube::Resource;

use crate::aiven_types::aiven_redis::Redis;

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
