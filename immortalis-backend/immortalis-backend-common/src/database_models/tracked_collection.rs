use crate::schema::tracked_collections;
use chrono::Utc;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, std::fmt::Debug, Queryable, Identifiable, Selectable)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct TrackedCollection {
    pub id: i32,
    pub url: String,
    pub tracking_started_at: chrono::DateTime<Utc>,
    pub last_checked: Option<chrono::DateTime<Utc>>,
}
