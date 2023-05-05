use serde::{Deserialize, Serialize};

use diesel::prelude::*;

#[derive(Deserialize, Serialize, std::fmt::Debug, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub title: String,
    pub value: String,
}