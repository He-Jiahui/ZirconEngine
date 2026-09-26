use serde::{Deserialize, Serialize};

/// 源文档 `text_key` 的结构化引用；缺失键的严重度取决于是否提供回退文本。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiLocalizedTextRef {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

impl UiLocalizedTextRef {
    /// 此处只检查键非空；表和实际键的存在性留给目录校验阶段。
    pub fn validate(&self, path: impl Into<String>) -> Option<String> {
        let path = path.into();
        if self.key.trim().is_empty() {
            Some(format!("localized text ref at {path} has an empty key"))
        } else {
            None
        }
    }
}
