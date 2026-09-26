//! 组件事件目标只携带域、主体和字段路径；具体解析与写入由宿主适配器负责。

use serde::{Deserialize, Serialize};

/// Identifies the runtime-owned domain and field path a component event targets.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiComponentBindingTarget {
    /// 路由域由宿主适配器解释，未知域必须由消费端拒绝。
    pub domain: String,
    pub subject: Option<String>,
    pub path: String,
}

impl UiComponentBindingTarget {
    pub fn new(domain: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            subject: None,
            path: path.into(),
        }
    }

    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    pub fn inspector(subject: impl Into<String>, field_path: impl Into<String>) -> Self {
        Self::new("inspector", field_path).with_subject(subject)
    }

    pub fn reflection(subject: impl Into<String>, field_path: impl Into<String>) -> Self {
        Self::new("reflection", field_path).with_subject(subject)
    }

    pub fn asset_editor(subject: impl Into<String>, field_path: impl Into<String>) -> Self {
        Self::new("asset_editor", field_path).with_subject(subject)
    }

    pub fn showcase(control_id: impl Into<String>) -> Self {
        Self::new("showcase", control_id)
    }
}
