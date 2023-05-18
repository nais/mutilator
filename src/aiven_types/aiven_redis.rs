// WARNING: generated by kopium - manual changes will be overwritten
// kopium command: kopium -Af -
// kopium version: 0.15.0

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

/// RedisSpec defines the desired state of Redis
#[derive(CustomResource, Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[kube(group = "aiven.io", version = "v1alpha1", kind = "Redis", plural = "redis")]
#[kube(namespaced)]
#[kube(status = "RedisStatus")]
pub struct RedisSpec {
    /// Authentication reference to Aiven token in a secret
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "authSecretRef")]
    pub auth_secret_ref: Option<RedisAuthSecretRef>,
    /// Cloud the service runs in.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "cloudName")]
    pub cloud_name: Option<String>,
    /// Information regarding secret creation. Exposed keys: `REDIS_HOST`, `REDIS_PORT`, `REDIS_USER`, `REDIS_PASSWORD`
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "connInfoSecretTarget")]
    pub conn_info_secret_target: Option<RedisConnInfoSecretTarget>,
    /// The disk space of the service, possible values depend on the service type, the cloud provider and the project. Reducing will result in the service re-balancing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk_space: Option<String>,
    /// Day of week when maintenance operations should be performed. One monday, tuesday, wednesday, etc.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "maintenanceWindowDow")]
    pub maintenance_window_dow: Option<RedisMaintenanceWindowDow>,
    /// Time of day when maintenance operations should be performed. UTC time in HH:mm:ss format.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "maintenanceWindowTime")]
    pub maintenance_window_time: Option<String>,
    /// Subscription plan.
    pub plan: String,
    /// Target project.
    pub project: String,
    /// ProjectVPCRef reference to ProjectVPC resource to use its ID as ProjectVPCID automatically
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "projectVPCRef")]
    pub project_vpc_ref: Option<RedisProjectVpcRef>,
    /// Identifier of the VPC the service should be in, if any.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "projectVpcId")]
    pub project_vpc_id: Option<String>,
    /// Service integrations to specify when creating a service. Not applied after initial service creation
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "serviceIntegrations")]
    pub service_integrations: Option<Vec<RedisServiceIntegrations>>,
    /// Tags are key-value pairs that allow you to categorize services.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<BTreeMap<String, String>>,
    /// Prevent service from being deleted. It is recommended to have this enabled for all services.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "terminationProtection")]
    pub termination_protection: Option<bool>,
    /// Redis specific user configuration options
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "userConfig")]
    pub user_config: Option<RedisUserConfig>,
}

/// Authentication reference to Aiven token in a secret
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisAuthSecretRef {
    pub key: String,
    pub name: String,
}

/// Information regarding secret creation. Exposed keys: `REDIS_HOST`, `REDIS_PORT`, `REDIS_USER`, `REDIS_PASSWORD`
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisConnInfoSecretTarget {
    /// Annotations added to the secret
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
    /// Labels added to the secret
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
    /// Name of the secret resource to be created. By default, is equal to the resource name
    pub name: String,
    /// Prefix for the secret's keys. Added "as is" without any transformations. By default, is equal to the kind name in uppercase + underscore, e.g. `KAFKA_`, `REDIS_`, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
}

/// RedisSpec defines the desired state of Redis
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum RedisMaintenanceWindowDow {
    #[serde(rename = "monday")]
    Monday,
    #[serde(rename = "tuesday")]
    Tuesday,
    #[serde(rename = "wednesday")]
    Wednesday,
    #[serde(rename = "thursday")]
    Thursday,
    #[serde(rename = "friday")]
    Friday,
    #[serde(rename = "saturday")]
    Saturday,
    #[serde(rename = "sunday")]
    Sunday,
}

/// ProjectVPCRef reference to ProjectVPC resource to use its ID as ProjectVPCID automatically
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisProjectVpcRef {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}

/// Service integrations to specify when creating a service. Not applied after initial service creation
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisServiceIntegrations {
    #[serde(rename = "integrationType")]
    pub integration_type: RedisServiceIntegrationsIntegrationType,
    #[serde(rename = "sourceServiceName")]
    pub source_service_name: String,
}

/// Service integrations to specify when creating a service. Not applied after initial service creation
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum RedisServiceIntegrationsIntegrationType {
    #[serde(rename = "read_replica")]
    ReadReplica,
}

/// Redis specific user configuration options
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisUserConfig {
    /// Additional Cloud Regions for Backup Replication
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_backup_regions: Option<Vec<String>>,
    /// Allow incoming connections from CIDR address block, e.g. '10.20.0.0/16'
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ip_filter: Option<Vec<RedisUserConfigIpFilter>>,
    /// Migrate data from existing server
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migration: Option<RedisUserConfigMigration>,
    /// Allow access to selected service ports from private networks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_access: Option<RedisUserConfigPrivateAccess>,
    /// Allow access to selected service components through Privatelink
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub privatelink_access: Option<RedisUserConfigPrivatelinkAccess>,
    /// Name of another project to fork a service from. This has effect only when a new service is being created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_to_fork_from: Option<String>,
    /// Allow access to selected service ports from the public Internet
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_access: Option<RedisUserConfigPublicAccess>,
    /// Name of the basebackup to restore in forked service
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_basebackup_name: Option<String>,
    /// Determines default pub/sub channels' ACL for new users if ACL is not supplied. When this option is not defined, all_channels is assumed to keep backward compatibility. This option doesn't affect Redis configuration acl-pubsub-default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_acl_channels_default: Option<RedisUserConfigRedisAclChannelsDefault>,
    /// Redis IO thread count
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_io_threads: Option<i64>,
    /// LFU maxmemory-policy counter decay time in minutes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_lfu_decay_time: Option<i64>,
    /// Counter logarithm factor for volatile-lfu and allkeys-lfu maxmemory-policies
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_lfu_log_factor: Option<i64>,
    /// Redis maxmemory-policy
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_maxmemory_policy: Option<RedisUserConfigRedisMaxmemoryPolicy>,
    /// Set notify-keyspace-events option
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_notify_keyspace_events: Option<String>,
    /// Set number of redis databases. Changing this will cause a restart of redis service.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_number_of_databases: Option<i64>,
    /// When persistence is 'rdb', Redis does RDB dumps each 10 minutes if any key is changed. Also RDB dumps are done according to backup schedule for backup purposes. When persistence is 'off', no RDB dumps and backups are done, so data can be lost at any moment if service is restarted for any reason, or if service is powered off. Also service can't be forked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_persistence: Option<RedisUserConfigRedisPersistence>,
    /// Set output buffer limit for pub / sub clients in MB. The value is the hard limit, the soft limit is 1/4 of the hard limit. When setting the limit, be mindful of the available memory in the selected service plan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_pubsub_client_output_buffer_limit: Option<i64>,
    /// Require SSL to access Redis
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_ssl: Option<bool>,
    /// Redis idle connection timeout in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis_timeout: Option<i64>,
    /// Name of another service to fork from. This has effect only when a new service is being created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_to_fork_from: Option<String>,
    /// Use static public IP addresses
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub static_ips: Option<bool>,
}

/// CIDR address block, either as a string, or in a dict with an optional description field
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisUserConfigIpFilter {
    /// Description for IP filter list entry
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// CIDR address block
    pub network: String,
}

/// Migrate data from existing server
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisUserConfigMigration {
    /// Database name for bootstrapping the initial connection
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dbname: Option<String>,
    /// Hostname or IP address of the server where to migrate data from
    pub host: String,
    /// Comma-separated list of databases, which should be ignored during migration (supported by MySQL and PostgreSQL only at the moment)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore_dbs: Option<String>,
    /// The migration method to be used (currently supported only by Redis, MySQL and PostgreSQL service types)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<RedisUserConfigMigrationMethod>,
    /// Password for authentication with the server where to migrate data from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Port number of the server where to migrate data from
    pub port: i64,
    /// The server where to migrate data from is secured with SSL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssl: Option<bool>,
    /// User name for authentication with the server where to migrate data from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// Migrate data from existing server
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum RedisUserConfigMigrationMethod {
    #[serde(rename = "dump")]
    Dump,
    #[serde(rename = "replication")]
    Replication,
}

/// Allow access to selected service ports from private networks
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisUserConfigPrivateAccess {
    /// Allow clients to connect to prometheus with a DNS name that always resolves to the service's private IP addresses. Only available in certain network locations
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prometheus: Option<bool>,
    /// Allow clients to connect to redis with a DNS name that always resolves to the service's private IP addresses. Only available in certain network locations
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis: Option<bool>,
}

/// Allow access to selected service components through Privatelink
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisUserConfigPrivatelinkAccess {
    /// Enable prometheus
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prometheus: Option<bool>,
    /// Enable redis
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis: Option<bool>,
}

/// Allow access to selected service ports from the public Internet
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisUserConfigPublicAccess {
    /// Allow clients to connect to prometheus from the public internet for service nodes that are in a project VPC or another type of private network
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prometheus: Option<bool>,
    /// Allow clients to connect to redis from the public internet for service nodes that are in a project VPC or another type of private network
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redis: Option<bool>,
}

/// Redis specific user configuration options
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum RedisUserConfigRedisAclChannelsDefault {
    #[serde(rename = "allchannels")]
    Allchannels,
    #[serde(rename = "resetchannels")]
    Resetchannels,
}

/// Redis specific user configuration options
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum RedisUserConfigRedisMaxmemoryPolicy {
    #[serde(rename = "noeviction")]
    Noeviction,
    #[serde(rename = "allkeys-lru")]
    AllkeysLru,
    #[serde(rename = "volatile-lru")]
    VolatileLru,
    #[serde(rename = "allkeys-random")]
    AllkeysRandom,
    #[serde(rename = "volatile-random")]
    VolatileRandom,
    #[serde(rename = "volatile-ttl")]
    VolatileTtl,
    #[serde(rename = "volatile-lfu")]
    VolatileLfu,
    #[serde(rename = "allkeys-lfu")]
    AllkeysLfu,
}

/// Redis specific user configuration options
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum RedisUserConfigRedisPersistence {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "rdb")]
    Rdb,
}

/// ServiceStatus defines the observed state of service
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisStatus {
    /// Conditions represent the latest available observations of a service state
    pub conditions: Vec<RedisStatusConditions>,
    /// Service state
    pub state: String,
}

/// Condition contains details for one aspect of the current state of this API Resource. --- This struct is intended for direct use as an array at the field path .status.conditions.  For example, 
///  type FooStatus struct{ // Represents the observations of a foo's current state. // Known .status.conditions.type are: "Available", "Progressing", and "Degraded" // +patchMergeKey=type // +patchStrategy=merge // +listType=map // +listMapKey=type Conditions []metav1.Condition `json:"conditions,omitempty" patchStrategy:"merge" patchMergeKey:"type" protobuf:"bytes,1,rep,name=conditions"` 
///  // other fields }
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct RedisStatusConditions {
    /// lastTransitionTime is the last time the condition transitioned from one status to another. This should be when the underlying condition changed.  If that is not known, then using the time when the API field changed is acceptable.
    #[serde(rename = "lastTransitionTime")]
    pub last_transition_time: String,
    /// message is a human readable message indicating details about the transition. This may be an empty string.
    pub message: String,
    /// observedGeneration represents the .metadata.generation that the condition was set based upon. For instance, if .metadata.generation is currently 12, but the .status.conditions[x].observedGeneration is 9, the condition is out of date with respect to the current state of the instance.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "observedGeneration")]
    pub observed_generation: Option<i64>,
    /// reason contains a programmatic identifier indicating the reason for the condition's last transition. Producers of specific condition types may define expected values and meanings for this field, and whether the values are considered a guaranteed API. The value should be a CamelCase string. This field may not be empty.
    pub reason: String,
    /// status of the condition, one of True, False, Unknown.
    pub status: RedisStatusConditionsStatus,
    /// type of condition in CamelCase or in foo.example.com/CamelCase. --- Many .condition.type values are consistent across resources like Available, but because arbitrary conditions can be useful (see .node.status.conditions), the ability to deconflict is important. The regex it matches is (dns1123SubdomainFmt/)?(qualifiedNameFmt)
    #[serde(rename = "type")]
    pub r#type: String,
}

/// Condition contains details for one aspect of the current state of this API Resource. --- This struct is intended for direct use as an array at the field path .status.conditions.  For example, 
///  type FooStatus struct{ // Represents the observations of a foo's current state. // Known .status.conditions.type are: "Available", "Progressing", and "Degraded" // +patchMergeKey=type // +patchStrategy=merge // +listType=map // +listMapKey=type Conditions []metav1.Condition `json:"conditions,omitempty" patchStrategy:"merge" patchMergeKey:"type" protobuf:"bytes,1,rep,name=conditions"` 
///  // other fields }
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum RedisStatusConditionsStatus {
    True,
    False,
    Unknown,
}
