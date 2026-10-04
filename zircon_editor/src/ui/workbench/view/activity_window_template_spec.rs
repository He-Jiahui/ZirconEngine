//! 独立活动窗口的模板资源声明，供宿主窗口创建时定位内容；构造器不加载资源。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityWindowTemplateSpec {
    pub document_id: String,
}

impl ActivityWindowTemplateSpec {
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
        }
    }
}
