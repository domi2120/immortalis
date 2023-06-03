use super::video_status::VideoStatus;
use crate::database_models::file::File;
use crate::schema::videos;
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
    pub upload_date: chrono::NaiveDateTime,
    pub archived_date: chrono::NaiveDateTime,
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
    pub upload_date: chrono::NaiveDateTime,
    pub archived_date: chrono::NaiveDateTime,
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
            upload_date: date_string_ymd_to_naive_date_time(&single_video.upload_date.unwrap()),
            archived_date: chrono::Utc::now().naive_utc(), 
            duration: single_video.duration.unwrap().as_i64().unwrap() as i32,
            original_url: single_video.webpage_url.unwrap(),
            status,
            file_id,
            thumbnail_id
        }
    }
}

/// creates a NaiveDateTime from a yyyy.mm.dd string
fn date_string_ymd_to_naive_date_time(upload_date: &str) -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(
            upload_date[0..=3].parse::<i32>().unwrap(),
            upload_date[4..=5].parse::<u32>().unwrap(),
            upload_date[6..=7].parse::<u32>().unwrap(),
        )
        .unwrap(),
        chrono::NaiveTime::from_num_seconds_from_midnight_opt(0, 0).unwrap())
}
