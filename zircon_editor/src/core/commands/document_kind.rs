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
        let segments = value.split('.').collect::<Vec<_>>();
        let valid = !segments.is_empty()
            && segments.iter().all(|segment| {
                !segment.is_empty()
                    && segment.chars().all(|character| {
                        character.is_ascii_lowercase()
                            || character.is_ascii_digit()
                            || character == '_'
                            || character == '-'
                    })
            });
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
