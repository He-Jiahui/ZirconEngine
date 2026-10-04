use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
/// 总线按完整字符串匹配的命名通道；parse 接受含点分隔符的小写 ASCII 名段，内置入口使用约定常量。
// TODO: [CR-EDITOR-SERVICES-0001] 确认反序列化边界是否也必须执行主题校验：派生 Deserialize 可绕过 parse，当前生产调用未发现外部解码入口。
// Editor1036: deserialization now validates through parse; the TODO above records the prior gap.
pub struct EditorTopic(String);

impl EditorTopic {
    pub(crate) fn document() -> Self {
        Self(super::topics::TOPIC_DOCUMENT.to_owned())
    }

    pub(crate) fn transaction() -> Self {
        Self(super::topics::TOPIC_TRANSACTION.to_owned())
    }

    pub(crate) fn log() -> Self {
        Self(super::topics::TOPIC_LOG.to_owned())
    }

    pub(crate) fn i18n() -> Self {
        Self(super::topics::TOPIC_I18N.to_owned())
    }

    pub(crate) fn tool() -> Self {
        Self(super::topics::TOPIC_TOOL.to_owned())
    }

    /// 校验命名空间与名段；保持缺少分隔符优先、随后首个名段错误的错误选择顺序，供调用方诊断。
    pub fn parse(value: impl Into<String>) -> Result<Self, EditorTopicError> {
        let value = value.into();
        if value.is_empty() {
            return Err(EditorTopicError::Empty);
        }
        let mut has_separator = false;
        let mut segment_start = 0;
        let mut segment_index = 0;
        let mut segment_invalid = false;
        let mut first_error = None;
        for (offset, byte) in value.bytes().enumerate() {
            if byte == b'.' {
                has_separator = true;
                if first_error.is_none() {
                    if offset == segment_start {
                        first_error = Some(EditorTopicError::EmptySegment {
                            index: segment_index,
                        });
                    } else if segment_invalid {
                        first_error = Some(EditorTopicError::InvalidSegment {
                            segment: value[segment_start..offset].to_string(),
                        });
                    }
                }
                segment_start = offset + 1;
                segment_index += 1;
                segment_invalid = false;
            } else if !topic_segment_byte_is_valid(byte) {
                segment_invalid = true;
            }
        }
        if !has_separator {
            return Err(EditorTopicError::MissingSeparator { value });
        }
        if first_error.is_none() {
            if segment_start == value.len() {
                first_error = Some(EditorTopicError::EmptySegment {
                    index: segment_index,
                });
            } else if segment_invalid {
                first_error = Some(EditorTopicError::InvalidSegment {
                    segment: value[segment_start..].to_string(),
                });
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for EditorTopic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for EditorTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorTopicError {
    Empty,
    MissingSeparator { value: String },
    EmptySegment { index: usize },
    InvalidSegment { segment: String },
}

impl fmt::Display for EditorTopicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("editor topic is empty"),
            Self::MissingSeparator { value } => {
                write!(
                    f,
                    "editor topic `{value}` must contain a namespace separator"
                )
            }
            Self::EmptySegment { index } => {
                write!(f, "editor topic segment {index} is empty")
            }
            Self::InvalidSegment { segment } => {
                write!(f, "editor topic segment `{segment}` is invalid")
            }
        }
    }
}

impl std::error::Error for EditorTopicError {}

fn topic_segment_byte_is_valid(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
}

#[cfg(test)]
#[path = "topic/tests/single_scan_tests.rs"]
mod single_scan_tests;

#[cfg(test)]
#[path = "topic/tests/deserialize_tests.rs"]
mod deserialize_tests;

#[cfg(test)]
#[path = "tests/topic.rs"]
mod tests;
