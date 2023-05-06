
use crate::schema::scheduled_archivals;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Deserialize, Serialize, std::fmt::Debug, Queryable, Identifiable, Selectable
)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledArchival {
    pub id: i32,
    pub url: String,
    pub scheduled_at: chrono::NaiveDateTime,
    pub not_before: chrono::NaiveDateTime
}
