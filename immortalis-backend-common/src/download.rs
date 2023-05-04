use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub title: String,
    pub value: String,
}