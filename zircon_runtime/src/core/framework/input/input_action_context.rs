use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputActionContext {
    pub id: String,
    pub priority: i32,
    pub enabled: bool,
}

impl InputActionContext {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            priority: 0,
            enabled: true,
        }
    }

    pub fn disabled(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            priority: 0,
            enabled: false,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
