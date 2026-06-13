use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputAction {
    pub id: String,
    pub context: Option<String>,
    pub display_name: Option<String>,
}

impl InputAction {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            context: None,
            display_name: None,
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }
}
