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
#[path = "document_kind/byte_scan_tests.rs"]
mod byte_scan_tests;

#[cfg(test)]
mod tests {
    #[test]
    fn document_kind_validation_streams_segments() {
        let source = include_str!("document_kind.rs");
        let collecting_shape = ["split('.')", ".collect::<Vec<_>>()"].concat();
        assert!(!source.contains(&collecting_shape));
    }
}
