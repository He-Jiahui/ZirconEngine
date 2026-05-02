use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiLocalizedTextRef {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

impl UiLocalizedTextRef {
    pub fn validate(&self, path: impl Into<String>) -> Option<String> {
        let path = path.into();
        if self.key.trim().is_empty() {
            Some(format!("localized text ref at {path} has an empty key"))
        } else {
            None
        }
    }
}
