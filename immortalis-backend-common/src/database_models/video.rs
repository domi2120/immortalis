use crate::schema::videos;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::video_status::VideoStatus;

// https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
#[derive(Deserialize, Serialize, Identifiable, Selectable, std::fmt::Debug, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: i32,
    pub title: String,
    pub channel: String,
    pub views: i64,
    pub upload_date: chrono::NaiveDateTime,
    pub archived_date: chrono::NaiveDateTime,
    pub duration: i32,
    pub thumbnail_address: String,
    pub original_url: String,
    pub status: VideoStatus,
}

#[derive(Deserialize, Serialize, Selectable, std::fmt::Debug, Insertable)]
#[diesel(table_name=videos)]
pub struct InsertableVideo {
    pub title: String,
    pub channel: String,
    pub views: i64,
    pub upload_date: chrono::NaiveDateTime,
    pub archived_date: chrono::NaiveDateTime,
    pub duration: i32,
    pub thumbnail_address: String,
    pub original_url: String,
    pub status: VideoStatus,
}
