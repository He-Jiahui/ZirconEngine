use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetReference, AssetUri};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ZShaderDocument {
    #[serde(default = "default_zshader_version")]
    pub version: u32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub wgsl_files: Vec<String>,
    #[serde(default)]
    pub entry_points: Vec<ZShaderEntryPointDocument>,
    #[serde(default)]
    pub imports: Vec<ZShaderImportDocument>,
    #[serde(default)]
    pub properties: Vec<ShaderMaterialPropertyAsset>,
    #[serde(default)]
    pub texture_slots: Vec<ZShaderTextureSlotDocument>,
    #[serde(default)]
    pub editor: toml::Table,
}

impl ZShaderDocument {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZShaderEntryPointDocument {
    pub name: String,
    pub stage: String,
    #[serde(default)]
    pub file: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZShaderImportDocument {
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect: Option<AssetReference>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderSourceFileAsset {
    pub path: String,
    pub url: AssetUri,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderImportRedirectAsset {
    pub source: String,
    pub redirect: AssetReference,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShaderMaterialPropertyAsset {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<toml::Value>,
    #[serde(default)]
    pub editor: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZShaderTextureSlotDocument {
    pub name: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampler: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub editor: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderTextureSlotAsset {
    pub name: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampler: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default)]
    pub editor: BTreeMap<String, String>,
}

impl From<&ZShaderTextureSlotDocument> for ShaderTextureSlotAsset {
    fn from(slot: &ZShaderTextureSlotDocument) -> Self {
        Self {
            name: slot.name.clone(),
            kind: slot.kind.clone(),
            default: slot.default.clone(),
            sampler: slot.sampler.clone(),
            group: slot.group.clone(),
            label: slot.label.clone(),
            editor: slot.editor.clone(),
        }
    }
}

impl ShaderMaterialPropertyAsset {
    pub fn accepts_value(&self, value: &toml::Value) -> bool {
        match self.kind.trim().to_ascii_lowercase().as_str() {
            "bool" | "boolean" => value.as_bool().is_some(),
            "float" | "f32" | "number" => {
                value.as_float().is_some() || value.as_integer().is_some()
            }
            "int" | "i32" | "u32" | "integer" => value.as_integer().is_some(),
            "string" => value.as_str().is_some(),
            "color" | "color4" | "vec4" => numeric_array_len(value, 4),
            "vec3" => numeric_array_len(value, 3),
            "vec2" => numeric_array_len(value, 2),
            "texture" | "asset_ref" | "reference" => {
                value.as_str().is_some() || value.as_table().is_some()
            }
            _ => true,
        }
    }
}

fn numeric_array_len(value: &toml::Value, len: usize) -> bool {
    value.as_array().is_some_and(|items| {
        items.len() == len
            && items
                .iter()
                .all(|item| item.as_float().is_some() || item.as_integer().is_some())
    })
}

fn default_zshader_version() -> u32 {
    1
}
