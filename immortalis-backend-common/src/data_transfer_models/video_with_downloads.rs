use serde::{Deserialize, Serialize};

use crate::database_models::{video::Video, download::Download};

#[derive(Serialize, Deserialize)]
pub struct VideoWithDownload {
    #[serde(flatten)]
    pub video: Video,
    pub downloads: Vec<Download>,
}