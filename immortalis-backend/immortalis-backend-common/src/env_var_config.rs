use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct EnvVarConfig {
    pub database_url: String,
    pub file_storage_location: String,
    pub temp_file_storage_location: String,
    pub simulate_download: bool,
    pub simulated_download_duration_seconds: u64,
    pub archiver_thread_count: u16,
    pub archiver_archiving_timeout_seconds: i64,
    pub tracker_thread_count: u16,
    pub use_ipv6: bool,
}
