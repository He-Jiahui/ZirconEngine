use serde::{Deserialize, Serialize};

use super::family::FontFamilyName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontScript {
    Latin,
    Cyrillic,
    Greek,
    Han,
    Hiragana,
    Katakana,
    Hangul,
    Arabic,
    Hebrew,
    Devanagari,
    Other(u32),
}

/// Normalized BCP-47 culture selector used to disambiguate script-equivalent
/// composite sub-fonts (for example, Han faces for zh-Hans, ja, and ko).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FontCultureTag(String);

impl FontCultureTag {
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into().trim().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn matches(&self, language: &str) -> bool {
        let configured = self.as_str();
        let language = language.trim();
        if configured.is_empty() || language.is_empty() {
            return false;
        }
        if configured.eq_ignore_ascii_case(language) {
            return true;
        }
        language
            .get(..configured.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(configured))
            && language.as_bytes().get(configured.len()) == Some(&b'-')
    }
}

impl From<&str> for FontCultureTag {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for FontCultureTag {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompositeFontDescriptor {
    pub default_family: FontFamilyName,
    #[serde(default)]
    pub sub_fonts: Vec<SubFontRange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubFontRange {
    pub family: FontFamilyName,
    #[serde(default)]
    pub scripts: Vec<FontScript>,
    #[serde(default)]
    pub ranges: Vec<(u32, u32)>,
    #[serde(default)]
    pub cultures: Vec<FontCultureTag>,
}
