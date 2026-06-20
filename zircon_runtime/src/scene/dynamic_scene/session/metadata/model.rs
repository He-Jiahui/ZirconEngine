use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSessionMetadata {
    #[serde(default)]
    pub project_root: Option<String>,
    #[serde(default)]
    pub asset_uri: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub created_at_unix_millis: Option<u64>,
    #[serde(default)]
    pub updated_at_unix_millis: Option<u64>,
    #[serde(default)]
    pub tags: Vec<String>,
}
