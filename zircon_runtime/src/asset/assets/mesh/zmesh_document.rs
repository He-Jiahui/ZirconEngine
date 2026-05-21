use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::core::framework::render::RenderMeshTopology;

use super::super::model::VirtualGeometryAsset;
use super::{MeshAsset, MeshAssetUsage, MeshAttributeValues, MeshIndices, MeshValidationError};

pub const ZMESH_DOCUMENT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZMeshDocument {
    #[serde(default = "default_zmesh_document_version")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub topology: RenderMeshTopology,
    #[serde(default)]
    pub attributes: BTreeMap<String, MeshAttributeValues>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indices: Option<MeshIndices>,
    #[serde(default)]
    pub asset_usage: MeshAssetUsage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_geometry: Option<VirtualGeometryAsset>,
}

impl ZMeshDocument {
    pub fn from_toml_str(document: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(document)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn into_mesh_asset(self, uri: AssetUri) -> Result<MeshAsset, MeshValidationError> {
        let asset = MeshAsset {
            uri,
            topology: self.topology,
            attributes: self.attributes,
            indices: self.indices,
            asset_usage: self.asset_usage,
            virtual_geometry: self.virtual_geometry,
        };
        asset.validate()?;
        Ok(asset)
    }
}

impl From<&MeshAsset> for ZMeshDocument {
    fn from(asset: &MeshAsset) -> Self {
        Self {
            version: ZMESH_DOCUMENT_VERSION,
            name: None,
            topology: asset.topology,
            attributes: asset.attributes.clone(),
            indices: asset.indices.clone(),
            asset_usage: asset.asset_usage,
            virtual_geometry: asset.virtual_geometry.clone(),
        }
    }
}

const fn default_zmesh_document_version() -> u32 {
    ZMESH_DOCUMENT_VERSION
}
