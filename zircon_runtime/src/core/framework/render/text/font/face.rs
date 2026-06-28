use serde::{Deserialize, Serialize};

use super::family::FontFamilyName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FontWeight(pub u16);

impl FontWeight {
    pub const NORMAL: Self = Self(400);
    pub const BOLD: Self = Self(700);

    pub fn clamped(value: u16) -> Self {
        Self(value.clamp(100, 900))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique(f32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FontStretch(pub u16);

impl FontStretch {
    pub const NORMAL: Self = Self(100);

    pub fn clamped(value: u16) -> Self {
        Self(value.clamp(50, 200))
    }
}

pub type FaceIndex = u32;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VariationCoords(pub Vec<(u32, f32)>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontFaceDescriptor {
    pub family: FontFamilyName,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub stretch: FontStretch,
    pub face_index: FaceIndex,
    #[serde(default)]
    pub variations: VariationCoords,
}

impl FontFaceDescriptor {
    pub fn regular(family: impl Into<FontFamilyName>) -> Self {
        Self {
            family: family.into(),
            weight: FontWeight::NORMAL,
            style: FontStyle::Normal,
            stretch: FontStretch::NORMAL,
            face_index: 0,
            variations: VariationCoords::default(),
        }
    }
}
