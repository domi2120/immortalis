use serde::{Deserialize, Serialize};

use crate::database_models::video::Video;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoDto {
    #[serde(flatten)]
    pub video: Video,
    pub video_size: i64,
}
