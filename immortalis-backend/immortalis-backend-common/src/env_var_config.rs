use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct EnvVarConfigGeneral {
    pub database_url: String,
}

#[derive(Deserialize, Debug)]
pub struct StorageConfig {
    pub file_storage_location: String,
    pub temp_file_storage_location: String,

    // s3 credentials, need to be optional (either s3, or disk storage must exist)
    #[serde(default)]
    pub s3_internal_url: String, // used to store data
    #[serde(default)]
    pub s3_external_url: String, // used for presigned links
    #[serde(default)]
    pub s3_bucket_name: String,
    #[serde(default)]
    pub s3_access_key: String,
    #[serde(default)]
    pub s3_secret_key: String,
}

#[derive(Deserialize, Debug)]
pub struct EnvVarConfigApi {
    #[serde(default)] // https://github.com/softprops/envy/issues/26
    pub use_s3: bool,

    #[serde(default = "s3_file_cache_duration_seconds_default")]
    pub s3_file_cache_duration_seconds: u32,
    #[serde(default = "disk_file_cache_duration_seconds_default")]
    pub disk_file_cache_duration_seconds: u32,

    #[serde(flatten)]
    pub general_config: EnvVarConfigGeneral,
    #[serde(flatten)]
    pub storage_config: StorageConfig,
    pub use_ipv6: bool,
}

const fn s3_file_cache_duration_seconds_default() -> u32 {
    60 * 60 * 24 * 7
}

const fn disk_file_cache_duration_seconds_default() -> u32 {
    60 * 60 * 24 * 365
}

#[derive(Deserialize, Debug)]
pub struct EnvVarConfigArchiver {
    #[serde(default)] // https://github.com/softprops/envy/issues/26
    pub use_s3: bool,

    #[serde(flatten)]
    pub general_config: EnvVarConfigGeneral,
    #[serde(flatten)]
    pub storage_config: StorageConfig,
    pub simulate_download: bool,
    pub simulated_download_duration_seconds: u64,
    pub archiver_thread_count: u16,
    pub archiver_archiving_timeout_seconds: i64,
    pub archiver_error_backoff_seconds: i64,
}

#[derive(Deserialize, Debug)]
pub struct EnvVarConfigTracker {
    #[serde(flatten)]
    pub general_config: EnvVarConfigGeneral,
    pub tracker_thread_count: u16,
}

#[derive(Deserialize, Debug)]
pub struct EnvVarConfigCommon {
    #[serde(flatten)]
    pub general_config: EnvVarConfigGeneral,
}
