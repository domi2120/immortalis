

use crate::database_models::download;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::{videos};

// https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
use chrono::serde::ts_milliseconds;
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
    // pub downloads: Vec<Download> ,
    // pub selected_download: Download,
    pub original_url: String,
}

