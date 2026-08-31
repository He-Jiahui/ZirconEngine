use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
#[path = "topic/single_scan_tests.rs"]
mod single_scan_tests;

#[cfg(test)]
mod tests {
    use super::EditorTopic;

    #[test]
    fn built_in_transaction_topic_is_canonical_and_valid() {
        assert_eq!(EditorTopic::transaction().as_str(), "editor.transaction");
    }

    #[test]
    fn built_in_document_topic_is_canonical_and_valid() {
        assert_eq!(EditorTopic::document().as_str(), "editor.document");
    }

    #[test]
    fn built_in_log_topic_is_canonical_and_valid() {
        assert_eq!(EditorTopic::log().as_str(), "editor.log");
    }

    #[test]
    fn built_in_i18n_topic_is_canonical_and_valid() {
        assert_eq!(EditorTopic::i18n().as_str(), "editor.i18n");
    }

    #[test]
    fn built_in_tool_topic_is_canonical_and_valid() {
        assert_eq!(EditorTopic::tool().as_str(), "editor.tool");
    }
}
