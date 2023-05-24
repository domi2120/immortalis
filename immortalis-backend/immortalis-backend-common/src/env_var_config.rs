use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct EnvVarConfig {
    pub database_url: String,
    pub file_storage_location: String,
    pub temp_file_storage_location: String,
    pub skip_download: bool,
    pub archiver_thread_count: u16,
    pub tracker_thread_count: u16,
    pub use_ipv6: bool,
}