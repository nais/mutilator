use std::collections::BTreeMap;

use kube::core::DynamicObject;

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
