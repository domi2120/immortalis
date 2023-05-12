use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GetFileRequestData {
    pub file_id: Uuid,
    pub is_thumbnail: bool,
}

#[derive(Deserialize)]
pub struct ScheduleRequest {
    pub url: String,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub term: Option<String>,
}
