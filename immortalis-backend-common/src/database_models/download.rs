use serde::{Deserialize, Serialize};
use crate::database_models::video::Video;
use crate::schema::{downloads};
use diesel::prelude::*;

#[derive(Deserialize, Serialize, std::fmt::Debug, Queryable, Associations, Identifiable, Selectable)]
#[diesel(belongs_to(Video))]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub id: i32,
    pub video_id: i32,
    pub title: String,
    pub value: String,
}