use std::fmt::{Display, Formatter};

use thiserror::Error;

const MAX_EXTERNAL_EFFECT_ID_LEN: usize = 96;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirtyExternalEffectId(String);

impl DirtyExternalEffectId {
    pub fn ui_source_buffer() -> Self {
        Self("ui.source_buffer".to_string())
    }

    pub fn animation_document() -> Self {
        Self("animation.document".to_string())
    }

    pub fn parse(value: impl Into<String>) -> Result<Self, DirtyExternalEffectIdError> {
        let value = value.into();
        if value.is_empty() {
            return Err(DirtyExternalEffectIdError::Empty);
        }
        if value.len() > MAX_EXTERNAL_EFFECT_ID_LEN {
            return Err(DirtyExternalEffectIdError::TooLong {
                len: value.len(),
                max: MAX_EXTERNAL_EFFECT_ID_LEN,
            });
        }
        if value.split('.').any(str::is_empty) {
            return Err(DirtyExternalEffectIdError::EmptySegment { value });
        }
        if let Some((index, character)) = value.char_indices().find(|(_, character)| {
            !(character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '_' | '-' | '.'))
        }) {
            return Err(DirtyExternalEffectIdError::InvalidCharacter {
                value,
                index,
                character,
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for DirtyExternalEffectId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DirtyExternalEffectIdError {
    #[error("dirty external-effect id cannot be empty")]
    Empty,
    #[error("dirty external-effect id length {len} exceeds {max}")]
    TooLong { len: usize, max: usize },
    #[error("dirty external-effect id contains an empty namespace segment: {value}")]
    EmptySegment { value: String },
    #[error(
        "dirty external-effect id contains invalid character {character:?} at byte {index}: {value}"
    )]
    InvalidCharacter {
        value: String,
        index: usize,
        character: char,
    },
}
