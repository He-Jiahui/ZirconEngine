use serde::{Deserialize, Serialize};

use super::face::{FontStretch, FontStyle, FontWeight};
use crate::asset::assets::FontFamilyName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontFaceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstancedFaceId(pub u64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontQuery {
    pub families: Vec<FontFamilyName>,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
}

impl FontQuery {
    pub fn single_family(family: impl Into<FontFamilyName>) -> Self {
        Self {
            families: vec![family.into()],
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontMatch {
    pub face: FontFaceId,
    pub synthetic_bold: bool,
    pub synthetic_oblique: bool,
}
