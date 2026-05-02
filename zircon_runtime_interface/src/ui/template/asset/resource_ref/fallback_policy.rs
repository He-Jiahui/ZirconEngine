use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiResourceFallbackMode {
    #[default]
    None,
    Placeholder,
    Optional,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiResourceFallbackPolicy {
    #[serde(default)]
    pub mode: UiResourceFallbackMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}
