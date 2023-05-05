use serde::{Deserialize, Serialize};

#[derive(diesel_derive_enum::DbEnum, Debug, Deserialize, Serialize)]
#[ExistingTypePath = "crate::schema::sql_types::VideoStatus"]
pub enum VideoStatus {
    Archived,
    ScheduledForArchival,
    BeingArchived,
    ArchivationFailed,
}