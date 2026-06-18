use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{
    AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset, MeshSkinAsset,
    VirtualGeometryAsset,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheMeshAsset {
    uri: AssetUri,
    topology: crate::core::framework::render::RenderMeshTopology,
    attributes: BTreeMap<String, ArtifactCacheMeshAttributeValues>,
    indices: Option<ArtifactCacheMeshIndices>,
    asset_usage: crate::asset::MeshAssetUsage,
    morph_targets: Vec<ArtifactCacheMeshMorphTargetAsset>,
    skin: Option<MeshSkinAsset>,
    virtual_geometry: Option<VirtualGeometryAsset>,
}

impl From<&MeshAsset> for ArtifactCacheMeshAsset {
    fn from(asset: &MeshAsset) -> Self {
        Self {
            uri: asset.uri.clone(),
            topology: asset.topology,
            attributes: mesh_attribute_table_to_cache(&asset.attributes),
            indices: asset.indices.as_ref().map(ArtifactCacheMeshIndices::from),
            asset_usage: asset.asset_usage,
            morph_targets: asset
                .morph_targets
                .iter()
                .map(ArtifactCacheMeshMorphTargetAsset::from)
                .collect(),
            skin: asset.skin.clone(),
            virtual_geometry: asset.virtual_geometry.clone(),
        }
    }
}

impl ArtifactCacheMeshAsset {
    pub(super) fn into_asset(self) -> MeshAsset {
        MeshAsset {
            uri: self.uri,
            topology: self.topology,
            attributes: cache_table_to_mesh_attributes(self.attributes),
            indices: self.indices.map(ArtifactCacheMeshIndices::into),
            asset_usage: self.asset_usage,
            morph_targets: self
                .morph_targets
                .into_iter()
                .map(ArtifactCacheMeshMorphTargetAsset::into)
                .collect(),
            skin: self.skin,
            virtual_geometry: self.virtual_geometry,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheMeshMorphTargetAsset {
    name: Option<String>,
    attributes: BTreeMap<String, ArtifactCacheMeshAttributeValues>,
}

impl From<&MeshMorphTargetAsset> for ArtifactCacheMeshMorphTargetAsset {
    fn from(asset: &MeshMorphTargetAsset) -> Self {
        Self {
            name: asset.name.clone(),
            attributes: mesh_attribute_table_to_cache(&asset.attributes),
        }
    }
}

impl From<ArtifactCacheMeshMorphTargetAsset> for MeshMorphTargetAsset {
    fn from(asset: ArtifactCacheMeshMorphTargetAsset) -> Self {
        Self {
            name: asset.name,
            attributes: cache_table_to_mesh_attributes(asset.attributes),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum ArtifactCacheMeshAttributeValues {
    Float32x2(Vec<[f32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Float32x4(Vec<[f32; 4]>),
    Uint16x4(Vec<[u16; 4]>),
    Uint32x4(Vec<[u32; 4]>),
}

impl From<&MeshAttributeValues> for ArtifactCacheMeshAttributeValues {
    fn from(values: &MeshAttributeValues) -> Self {
        match values {
            MeshAttributeValues::Float32x2(values) => Self::Float32x2(values.clone()),
            MeshAttributeValues::Float32x3(values) => Self::Float32x3(values.clone()),
            MeshAttributeValues::Float32x4(values) => Self::Float32x4(values.clone()),
            MeshAttributeValues::Uint16x4(values) => Self::Uint16x4(values.clone()),
            MeshAttributeValues::Uint32x4(values) => Self::Uint32x4(values.clone()),
        }
    }
}

impl From<ArtifactCacheMeshAttributeValues> for MeshAttributeValues {
    fn from(values: ArtifactCacheMeshAttributeValues) -> Self {
        match values {
            ArtifactCacheMeshAttributeValues::Float32x2(values) => Self::Float32x2(values),
            ArtifactCacheMeshAttributeValues::Float32x3(values) => Self::Float32x3(values),
            ArtifactCacheMeshAttributeValues::Float32x4(values) => Self::Float32x4(values),
            ArtifactCacheMeshAttributeValues::Uint16x4(values) => Self::Uint16x4(values),
            ArtifactCacheMeshAttributeValues::Uint32x4(values) => Self::Uint32x4(values),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum ArtifactCacheMeshIndices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

impl From<&MeshIndices> for ArtifactCacheMeshIndices {
    fn from(indices: &MeshIndices) -> Self {
        match indices {
            MeshIndices::U16(indices) => Self::U16(indices.clone()),
            MeshIndices::U32(indices) => Self::U32(indices.clone()),
        }
    }
}

impl From<ArtifactCacheMeshIndices> for MeshIndices {
    fn from(indices: ArtifactCacheMeshIndices) -> Self {
        match indices {
            ArtifactCacheMeshIndices::U16(indices) => Self::U16(indices),
            ArtifactCacheMeshIndices::U32(indices) => Self::U32(indices),
        }
    }
}

fn mesh_attribute_table_to_cache(
    table: &BTreeMap<String, MeshAttributeValues>,
) -> BTreeMap<String, ArtifactCacheMeshAttributeValues> {
    table
        .iter()
        .map(|(key, value)| (key.clone(), ArtifactCacheMeshAttributeValues::from(value)))
        .collect()
}

fn cache_table_to_mesh_attributes(
    table: BTreeMap<String, ArtifactCacheMeshAttributeValues>,
) -> BTreeMap<String, MeshAttributeValues> {
    table
        .into_iter()
        .map(|(key, value)| (key, value.into()))
        .collect()
}
