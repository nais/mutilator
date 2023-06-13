use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{debug_handler, Router};
use axum_server::tls_rustls::RustlsConfig;
use json_patch::Patch;
use kube::core::admission::{AdmissionRequest, AdmissionResponse, AdmissionReview, Operation};
use kube::core::DynamicObject;
use kube::ResourceExt;
use tracing::{debug, error, info, info_span, instrument, warn};

use crate::aiven_types::aiven_redis::Redis;
use crate::mutators;
use crate::settings::AppConfig;

#[instrument(skip_all)]
pub async fn start_web_server(config: AppConfig) -> Result<()> {
	let certificate_path = config.web.certificate_path.clone();
	let private_key_path = config.web.private_key_path.clone();
	let addr = config.web.bind_address.parse().unwrap();

	let state = Arc::new(config);
	let router = create_router(state);

	if certificate_path.is_some() && private_key_path.is_some() {
		let tls_config =
			RustlsConfig::from_pem_file(certificate_path.unwrap(), private_key_path.unwrap())
				.await?;
		info!("Starting webserver on {} using https", addr);
		axum_server::bind_rustls(addr, tls_config)
			.serve(router.into_make_service())
			.await?;
	} else {
		info!("Starting webserver on {} using http", addr);
		axum_server::bind(addr)
			.serve(router.into_make_service())
			.await?;
	}

	Ok(())
}

fn create_router(state: Arc<AppConfig>) -> Router {
	let router = Router::new()
		.route("/is_alive", get(|| async { "I'm alive!" }))
		.route("/is_ready", get(|| async { "Ready for action!" }))
		.route("/mutate", post(mutate_handler))
		.with_state(state);
	router
}

#[debug_handler]
#[instrument(skip_all)]
async fn mutate_handler(
	State(config): State<Arc<AppConfig>>,
	Json(admission_review): Json<AdmissionReview<Redis>>,
) -> (StatusCode, Json<AdmissionReview<DynamicObject>>) {
	let req: AdmissionRequest<Redis> = match admission_review.try_into() {
		Ok(req) => req,
		Err(err) => {
			warn!(
				"Unable to get request from AdmissionReview: {}",
				err.to_string()
			);
			return (
				StatusCode::BAD_REQUEST,
				Json(AdmissionResponse::invalid("missing request").into_review()),
			);
		},
	};

	let uid = req.uid.clone();
	let mut res = AdmissionResponse::from(&req);
	let req_span = info_span!("request", request_uid = uid);
	let _req_guard = req_span.enter();

	if req.operation != Operation::Create {
		debug!("Ignoring operation {:?}", req.operation);
		return (StatusCode::OK, Json(res.into_review()));
	}

	if let Some(obj) = &req.object {
		let name = obj.name_any();
		let namespace = obj.namespace().unwrap();
		let redis_span = info_span!(
			"redis",
			resource_name = name,
			resource_namespace = namespace
		);
		let _redis_guard = redis_span.enter();
		info!("Processing redis resource");
		res = match mutate(res.clone(), &obj, &config) {
			Ok(res) => {
				info!("Processing complete");
				res
			},
			Err(err) => {
				error!("Processing failed: {}", err.to_string());
				res.deny(err.to_string())
			},
		};
		(StatusCode::OK, Json(res.into_review()))
	} else {
		warn!("No object specified in AdmissionRequest: {:?}", req);
		(
			StatusCode::BAD_REQUEST,
			Json(AdmissionResponse::invalid("no object specified").into_review()),
		)
	}
}

#[instrument(skip_all)]
fn mutate(
	res: AdmissionResponse,
	obj: &Redis,
	config: &Arc<AppConfig>,
) -> Result<AdmissionResponse> {
	let mut patches = Vec::new();

	mutators::add_project_vpc_id(config.project_vpc_id.clone(), obj, &mut patches);
	mutators::add_termination_protection(obj, &mut patches);
	mutators::add_tags(config, obj, &mut patches);
	mutators::add_location(config.location.clone(), obj, &mut patches);

	Ok(res.with_patch(Patch(patches))?)
}

#[cfg(test)]
mod tests {
	use std::fs::File;
	use std::io::BufReader;
	use std::path::PathBuf;
	use std::sync::Arc;

	use axum_test::TestServer;
	use json_patch::Patch;
	use kube::core::admission::AdmissionReview;
	use kube::core::DynamicObject;
	use pretty_assertions::assert_eq;
	use rstest::*;
	use serde::{Deserialize, Serialize};

	use serde_yaml;

	use crate::settings::{AppConfig, LogLevel, Tenant, WebConfig};
	use crate::web::create_router;

	#[derive(Serialize, Deserialize, Debug)]
	pub struct Asserts {
		status_code: u16,
		num_patches: usize,
	}

	#[derive(Serialize, Deserialize, Debug)]
	pub struct TestData {
		admission_review: AdmissionReview<DynamicObject>,
		asserts: Asserts,
	}

	#[fixture]
	pub fn test_server() -> TestServer {
		let state = Arc::new(AppConfig {
			log_format: Default::default(),
			log_level: LogLevel::Trace,
			web: WebConfig {
				bind_address: "".to_string(),
				certificate_path: None,
				private_key_path: None,
			},
			tenant: Tenant {
				environment: "test-tenant-env".to_string(),
				name: "test-tenant-name".to_string(),
			},
			project_vpc_id: "test-vpc-id".to_string(),
			location: "test-location".to_string(),
			otel_enabled: false,
		});
		let router = create_router(state);
		TestServer::new(router.into_make_service()).unwrap()
	}

	#[fixture]
	pub fn test_dir() -> PathBuf {
		let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test_data/");
		root
	}

	fn test_data(path: PathBuf, file_name: &str) -> TestData {
		let file_path = path.join(file_name);
		serde_yaml::from_reader(BufReader::new(
			File::open(file_path.clone())
				.expect(format!("Unable to read '{}'", file_path.display()).as_str()),
		))
		.expect(format!("Unable to deserialize '{}'", file_path.display()).as_str())
	}

	#[rstest]
	#[case("golden_redis.yaml")]
	#[case("redis_with_all_tags.yaml")]
	#[tokio::test]
	async fn test_mutate(test_server: TestServer, test_dir: PathBuf, #[case] file_name: &str) {
		let test_data = test_data(test_dir, file_name);
		let resp = test_server
			.post("/mutate")
			.content_type(&"application/json")
			.json(&test_data.admission_review)
			.await;
		assert_eq!(resp.status_code(), test_data.asserts.status_code);
		let admission_result: AdmissionReview<DynamicObject> = resp.json();
		let admission_response = admission_result.response.as_ref().unwrap();
		println!("{:?}", &admission_result);
		assert!(admission_response.allowed);
		let patch = admission_response.patch.as_ref();
		if test_data.asserts.num_patches > 0 {
			assert!(patch.is_some());
			let patches: Patch = serde_json::from_slice(patch.unwrap().as_slice()).unwrap();
			assert_eq!(patches.len(), test_data.asserts.num_patches);
		} else {
			assert!(patch.is_none());
		}
	}

	#[rstest]
	#[case::liveness("/is_alive")]
	#[case::readiness("/is_ready")]
	#[tokio::test]
	async fn test_probes(test_server: TestServer, #[case] path: &str) {
		let resp = test_server.get(path).await;
		assert_eq!(resp.status_code(), 200);
	}
}
