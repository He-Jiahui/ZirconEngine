use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use super::EditorI18nError;

const ENGLISH_TAG: &str = "en";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EditorLocale(Arc<str>);

impl EditorLocale {
    pub fn parse(value: impl Into<String>) -> Result<Self, EditorI18nError> {
        let value = value.into();
        let mut parts = value.split('-');
        let Some(language) = parts.next() else {
            return Err(EditorI18nError::InvalidLocale(value));
        };
        if !(2..=3).contains(&language.len())
            || !language.bytes().all(|byte| byte.is_ascii_alphabetic())
        {
            return Err(EditorI18nError::InvalidLocale(value));
        }

        let mut normalized = language.to_ascii_lowercase();
        for qualifier in parts {
            if !(2..=8).contains(&qualifier.len())
                || !qualifier.bytes().all(|byte| byte.is_ascii_alphanumeric())
            {
                return Err(EditorI18nError::InvalidLocale(value));
            }
            normalized.push('-');
            if qualifier.len() == 2 && qualifier.bytes().all(|byte| byte.is_ascii_alphabetic()) {
                normalized.push_str(&qualifier.to_ascii_uppercase());
            } else {
                normalized.push_str(&qualifier.to_ascii_lowercase());
            }
        }
        Ok(Self(Arc::from(normalized)))
    }

    pub fn english() -> Self {
        Self(Arc::from(ENGLISH_TAG))
    }

    pub(super) const fn english_tag() -> &'static str {
        ENGLISH_TAG
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for EditorLocale {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Display for EditorLocale {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
