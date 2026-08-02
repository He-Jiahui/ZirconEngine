use serde::{Deserialize, Serialize};

use crate::asset::assets::FontFamilyName;

use super::face::FontFaceDescriptor;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontFamilyDescriptor {
    pub name: FontFamilyName,
    #[serde(default)]
    pub faces: Vec<FontFaceDescriptor>,
}
