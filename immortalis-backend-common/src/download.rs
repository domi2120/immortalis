use serde::{Deserialize, Serialize};

use diesel::prelude::*;

#[derive(Deserialize, Serialize, std::fmt::Debug, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub id: i32,
    pub video_id: i32,
    pub title: String,
    pub value: String,
}