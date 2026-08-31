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
        match validate_external_effect_id(&value) {
            Ok(()) => Ok(Self(value)),
            Err(ExternalEffectIdValidationFailure::EmptySegment) => {
                Err(DirtyExternalEffectIdError::EmptySegment { value })
            }
            Err(ExternalEffectIdValidationFailure::InvalidCharacter { index, character }) => {
                Err(DirtyExternalEffectIdError::InvalidCharacter {
                    value,
                    index,
                    character,
                })
            }
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExternalEffectIdValidationFailure {
    EmptySegment,
    InvalidCharacter { index: usize, character: char },
}

fn validate_external_effect_id(value: &str) -> Result<(), ExternalEffectIdValidationFailure> {
    let mut previous_was_separator = true;
    let mut has_empty_segment = false;
    let mut invalid_character = None;

    for (index, character) in value.char_indices() {
        if character == '.' {
            if previous_was_separator {
                has_empty_segment = true;
            }
            previous_was_separator = true;
            continue;
        }

        previous_was_separator = false;
        if invalid_character.is_none()
            && !(character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '_' | '-'))
        {
            invalid_character = Some((index, character));
        }
    }

    if previous_was_separator || has_empty_segment {
        return Err(ExternalEffectIdValidationFailure::EmptySegment);
    }
    if let Some((index, character)) = invalid_character {
        return Err(ExternalEffectIdValidationFailure::InvalidCharacter { index, character });
    }
    Ok(())
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

#[cfg(test)]
#[path = "external_effect_id/single_pass_tests.rs"]
mod single_pass_tests;
