use serde::{Deserialize, Serialize};

use crate::database_models::{download::Download, video::Video};

#[derive(Serialize, Deserialize)]
pub struct VideoWithDownload {
    #[serde(flatten)]
    pub video: Video,
    pub downloads: Vec<Download>,
}
