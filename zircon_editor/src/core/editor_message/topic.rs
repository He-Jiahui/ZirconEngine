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

    pub fn parse(value: impl Into<String>) -> Result<Self, EditorTopicError> {
        let value = value.into();
        if value.is_empty() {
            return Err(EditorTopicError::Empty);
        }
        if !value.contains('.') {
            return Err(EditorTopicError::MissingSeparator { value });
        }
        for (index, segment) in value.split('.').enumerate() {
            if segment.is_empty() {
                return Err(EditorTopicError::EmptySegment { index });
            }
            if !segment_is_valid(segment) {
                return Err(EditorTopicError::InvalidSegment {
                    segment: segment.to_string(),
                });
            }
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

fn segment_is_valid(segment: &str) -> bool {
    segment.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
    })
}

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
}
