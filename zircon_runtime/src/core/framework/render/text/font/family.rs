use serde::{Deserialize, Serialize};

use super::face::FontFaceDescriptor;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FontFamilyName(pub String);

impl FontFamilyName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into().trim().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl From<&str> for FontFamilyName {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for FontFamilyName {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FontFamilyDescriptor {
    pub name: FontFamilyName,
    #[serde(default)]
    pub faces: Vec<FontFaceDescriptor>,
}
