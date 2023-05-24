use crate::schema::files;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, std::fmt::Debug, Queryable, Identifiable, Selectable, Insertable)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct File {
    pub id: uuid::Uuid,
    pub file_name: String,
    pub file_extension: String,
    pub size: i64,
}
