use super::video_status::VideoStatus;
use crate::database_models::file::File;
use crate::schema::videos;
use chrono::{Utc, NaiveTime, NaiveDate, NaiveDateTime, DateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
#[derive(
    Deserialize, Serialize, Identifiable, Selectable, std::fmt::Debug, Queryable, Associations,
)]
#[diesel(belongs_to(File))]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: i32,
    pub title: String,
    pub channel: String,
    pub views: i64,
    pub upload_date: DateTime<Utc>,
    pub archived_date: DateTime<Utc>,
    pub duration: i32,
    pub original_url: String,
    pub status: VideoStatus,
    pub file_id: uuid::Uuid,
    pub thumbnail_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Selectable, std::fmt::Debug, Insertable)]
#[diesel(table_name=videos)]
pub struct InsertableVideo {
    pub title: String,
    pub channel: String,
    pub views: i64,
    pub upload_date: DateTime<Utc>,
    pub archived_date: DateTime<Utc>,
    pub duration: i32,
    pub original_url: String,
    pub status: VideoStatus,
    pub file_id: uuid::Uuid,
    pub thumbnail_id: uuid::Uuid,
}


impl InsertableVideo {
    pub fn new(single_video: youtube_dl::SingleVideo, status: VideoStatus, file_id: uuid::Uuid, thumbnail_id: uuid::Uuid) -> InsertableVideo {
        InsertableVideo {
            title: single_video.title,
            channel: single_video.channel.unwrap(),
            views: single_video.view_count.unwrap(),
            upload_date: DateTime::from_utc(NaiveDateTime::new(NaiveDate::parse_from_str(&single_video.upload_date.unwrap(), "%Y%m%d").unwrap(), NaiveTime::default()), Utc),
            archived_date: Utc::now(), 
            duration: single_video.duration.unwrap().as_i64().unwrap() as i32,
            original_url: single_video.webpage_url.unwrap(),
            status,
            file_id,
            thumbnail_id
        }
    }
}
