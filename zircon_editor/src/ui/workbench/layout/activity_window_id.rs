use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActivityWindowId(pub(crate) String);

impl ActivityWindowId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn workbench() -> Self {
        Self::new("window:workbench")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
