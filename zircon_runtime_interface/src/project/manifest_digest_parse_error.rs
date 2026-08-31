use thiserror::Error;

/// Reports malformed manifest-digest wire text before it reaches a session boundary.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
#[error(
    "project manifest digest must be exactly {expected_hex_characters} lowercase hexadecimal characters, received `{value}`"
)]
pub struct ProjectManifestDigestParseError {
    pub value: String,
    expected_hex_characters: usize,
}

impl ProjectManifestDigestParseError {
    pub(crate) fn new(value: String, expected_hex_characters: usize) -> Self {
        Self {
            value,
            expected_hex_characters,
        }
    }
}
