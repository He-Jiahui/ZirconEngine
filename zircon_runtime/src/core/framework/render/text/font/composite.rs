use serde::{Deserialize, Serialize};

use super::family::FontFamilyName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
}
