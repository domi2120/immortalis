use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct EnvVarConfig {
    pub database_url: String,
    pub file_storage_location: String,
    pub temp_file_storage_location: String,

    // s3 credentials, need to be optional (either s3, or disk storage must exist)
    #[serde(default)]
    pub use_s3: bool,
    #[serde(default)]
    pub s3_url: String,
    #[serde(default)]
    pub s3_bucket_name: String,
    #[serde(default)]
    pub s3_access_key: String,
    #[serde(default)]
    pub s3_secret_key: String,

    pub simulate_download: bool,
    pub simulated_download_duration_seconds: u64,
    pub archiver_thread_count: u16,
    pub archiver_archiving_timeout_seconds: i64,
    pub archiver_error_backoff_seconds: i64,
    pub tracker_thread_count: u16,
    pub use_ipv6: bool,
}
