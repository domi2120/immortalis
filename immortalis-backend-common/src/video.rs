

use crate::download::Download;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use chrono::serde::ts_milliseconds;
#[derive(Deserialize, Serialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub title: String,
    pub channel: String,
    pub views: u64,
    #[serde(with = "ts_milliseconds")]
    pub upload_date: chrono::DateTime<Utc>,
    #[serde(with = "ts_milliseconds")]
    pub archived_date: chrono::DateTime<Utc>,
    pub duration: u32,
    pub thumbnail_address: String,
    pub downloads: Vec<Download> ,
    pub selected_download: Download,
    pub original_url: String,
}

