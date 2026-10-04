//! 用稳定的文档族标识连接命令谓词和视图声明；序列化与显式解析共用校验，避免本地化名称进入可持久化契约。

use std::fmt;

use serde::{Deserialize, Serialize};

/// Stable, serialized document-family identifier used by command predicates.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DocumentKind(String);

impl DocumentKind {
    pub fn scene() -> Self {
        Self::builtin("scene")
    }

    pub fn prefab() -> Self {
        Self::builtin("prefab")
    }

    pub fn material() -> Self {
        Self::builtin("material")
    }

    pub fn ui_asset() -> Self {
        Self::builtin("ui_asset")
    }

    pub fn animation_sequence() -> Self {
        Self::builtin("animation_sequence")
    }

    pub fn animation_graph() -> Self {
        Self::builtin("animation_graph")
    }

    /// 接收来自声明或反序列化的规范标识；错误保留原输入，调用方应拒绝该声明而非回退为其他文档族。
    pub fn parse(value: impl Into<String>) -> Result<Self, DocumentKindError> {
        let value = value.into();
        let mut segment_has_value = false;
        let valid = !value.is_empty()
            && value.bytes().all(|byte| match byte {
                b'.' => {
                    segment_has_value && {
                        segment_has_value = false;
                        true
                    }
                }
                b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' => {
                    segment_has_value = true;
                    true
                }
                _ => false,
            })
            && segment_has_value;
        if valid {
            Ok(Self(value))
        } else {
            Err(DocumentKindError(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn builtin(value: &'static str) -> Self {
        Self::parse(value).expect("built-in editor document kind must be valid")
    }
}

impl TryFrom<String> for DocumentKind {
    type Error = DocumentKindError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl From<DocumentKind> for String {
    fn from(value: DocumentKind) -> Self {
        value.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentKindError(String);

impl fmt::Display for DocumentKindError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "editor document kind `{}` is invalid", self.0)
    }
}

impl std::error::Error for DocumentKindError {}

#[cfg(test)]
#[path = "document_kind/tests/byte_scan_tests.rs"]
mod byte_scan_tests;

#[cfg(test)]
#[path = "tests/document_kind.rs"]
mod tests;
