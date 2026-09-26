use serde::{Deserialize, Serialize};

/// 动作映射中的稳定逻辑标识；绑定按 `id` 查找，`display_name` 只供界面显示。
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
