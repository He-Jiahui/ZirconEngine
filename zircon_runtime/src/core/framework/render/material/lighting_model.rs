use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderMaterialLightingModel {
    Pbr,
    BlinnPhong,
    Unlit,
    Custom { name: String },
}

impl RenderMaterialLightingModel {
    pub const fn is_unlit(&self) -> bool {
        matches!(self, Self::Unlit)
    }

    pub fn as_token(&self) -> String {
        match self {
            Self::Pbr => "pbr".to_string(),
            Self::BlinnPhong => "blinn_phong".to_string(),
            Self::Unlit => "unlit".to_string(),
            Self::Custom { name } => format!("custom:{name}"),
        }
    }
}

impl Default for RenderMaterialLightingModel {
    fn default() -> Self {
        Self::Pbr
    }
}

impl Display for RenderMaterialLightingModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.as_token())
    }
}

impl FromStr for RenderMaterialLightingModel {
    type Err = RenderMaterialLightingModelParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let trimmed = value.trim();
        let normalized = trimmed.to_ascii_lowercase();
        match normalized.as_str() {
            "pbr" | "standard" | "standard_pbr" | "physically_based" | "physically-based" => {
                Ok(Self::Pbr)
            }
            "blinn_phong" | "blinn-phong" | "blinn phong" => Ok(Self::BlinnPhong),
            "unlit" | "unshaded" => Ok(Self::Unlit),
            _ => {
                if normalized.starts_with("custom:") {
                    let name = trimmed["custom:".len()..].trim();
                    if name.is_empty() {
                        Err(RenderMaterialLightingModelParseError)
                    } else {
                        Ok(Self::Custom {
                            name: name.to_string(),
                        })
                    }
                } else {
                    Err(RenderMaterialLightingModelParseError)
                }
            }
        }
    }
}

impl Serialize for RenderMaterialLightingModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_token())
    }
}

impl<'de> Deserialize<'de> for RenderMaterialLightingModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_str(&value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderMaterialLightingModelParseError;

impl Display for RenderMaterialLightingModelParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("unknown material lighting model")
    }
}

impl std::error::Error for RenderMaterialLightingModelParseError {}
