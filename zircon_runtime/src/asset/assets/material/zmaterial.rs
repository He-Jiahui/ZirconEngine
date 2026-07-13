use std::collections::BTreeMap;

use serde::de::Error as _;
use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;

use super::MaterialTextureSlotValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZMaterialDocument {
    #[serde(default = "default_zmaterial_version")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub shader: AssetReference,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<AssetReference>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub options: BTreeMap<String, toml::Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(alias = "property_overrides")]
    pub overrides: BTreeMap<String, toml::Value>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub textures: BTreeMap<String, MaterialTextureSlotValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue: Option<ZMaterialQueueOverride>,
    #[serde(default, skip_serializing_if = "toml::Table::is_empty")]
    pub editor: toml::Table,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_diagnostics: Vec<String>,
}

impl ZMaterialDocument {
    pub fn to_project_toml_string(
        &self,
        resolver: impl FnMut(
            &AssetReference,
        ) -> Result<
            zircon_runtime_interface::project::PersistedAssetReference,
            crate::asset::ReferenceResolutionError,
        >,
    ) -> Result<String, crate::asset::assets::ProjectDocumentError> {
        let document = crate::asset::assets::project_document::serialize_material(self, resolver)?;
        crate::asset::assets::project_document::validate_material(&document)?;
        Ok(document)
    }

    pub fn from_project_toml_str(
        document: &str,
        resolver: impl FnMut(
            &zircon_runtime_interface::project::PersistedAssetReference,
        ) -> Result<AssetReference, crate::asset::ReferenceResolutionError>,
    ) -> Result<Self, crate::asset::assets::ProjectDocumentError> {
        crate::asset::assets::project_document::validate_material(document)?;
        let parsed =
            crate::asset::assets::project_document::deserialize_material(document, resolver)?;
        validate_version(parsed).map_err(Into::into)
    }

    #[cfg(test)]
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        let parsed: Self = toml::from_str(document)?;
        validate_version(parsed)
    }

    #[cfg(test)]
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

fn validate_version(document: ZMaterialDocument) -> Result<ZMaterialDocument, toml::de::Error> {
    if document.version == 2 {
        Ok(document)
    } else {
        Err(toml::de::Error::custom(format!(
            "zmaterial v2 document version `{}` is unsupported; migrate material files to version = 2",
            document.version
        )))
    }
}

fn default_zmaterial_version() -> u32 {
    2
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZMaterialQueueOverride {
    #[serde(default, skip_serializing_if = "is_zero_i16")]
    pub offset: i16,
}

fn is_zero_i16(value: &i16) -> bool {
    *value == 0
}
