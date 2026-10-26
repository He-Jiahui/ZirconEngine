use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::asset::{
    AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset, MeshSdfAsset,
    MeshSkinAsset, VirtualGeometryAsset,
};

// ArtifactStore 的 bincode 边界使用此表示保存网格属性与索引；
// 读回后交还普通 MeshAsset，避免编辑侧 serde 形状成为库工件协议。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in crate::asset::artifact) struct ArtifactCacheMeshAsset {
    uri: AssetUri,
    topology: crate::core::framework::render::RenderMeshTopology,
    attributes: BTreeMap<String, ArtifactCacheMeshAttributeValues>,
    indices: Option<ArtifactCacheMeshIndices>,
    asset_usage: crate::asset::MeshAssetUsage,
    morph_targets: Vec<ArtifactCacheMeshMorphTargetAsset>,
    skin: Option<MeshSkinAsset>,
    #[serde(default)]
    mesh_sdf: Option<MeshSdfAsset>,
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
            mesh_sdf: asset.mesh_sdf.clone(),
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
            mesh_sdf: self.mesh_sdf,
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

#[derive(Serialize)]
pub(super) struct ArtifactCacheMeshAssetRef<'a> {
    uri: &'a AssetUri,
    topology: crate::core::framework::render::RenderMeshTopology,
    attributes: MeshAttributesRef<'a>,
    indices: Option<MeshIndicesRef<'a>>,
    asset_usage: crate::asset::MeshAssetUsage,
    morph_targets: MorphTargetsRef<'a>,
    skin: &'a Option<MeshSkinAsset>,
    mesh_sdf: &'a Option<MeshSdfAsset>,
    virtual_geometry: &'a Option<VirtualGeometryAsset>,
}
impl<'a> ArtifactCacheMeshAssetRef<'a> {
    pub(super) fn new(asset: &'a MeshAsset) -> Self {
        Self {
            uri: &asset.uri,
            topology: asset.topology,
            attributes: MeshAttributesRef(&asset.attributes),
            indices: asset.indices.as_ref().map(|indices| match indices {
                MeshIndices::U16(v) => MeshIndicesRef::U16(v),
                MeshIndices::U32(v) => MeshIndicesRef::U32(v),
            }),
            asset_usage: asset.asset_usage,
            morph_targets: MorphTargetsRef(&asset.morph_targets),
            skin: &asset.skin,
            mesh_sdf: &asset.mesh_sdf,
            virtual_geometry: &asset.virtual_geometry,
        }
    }
}
#[derive(Serialize)]
enum MeshAttributeRef<'a> {
    Float32x2(&'a [[f32; 2]]),
    Float32x3(&'a [[f32; 3]]),
    Float32x4(&'a [[f32; 4]]),
    Uint16x4(&'a [[u16; 4]]),
    Uint32x4(&'a [[u32; 4]]),
}
impl<'a> From<&'a MeshAttributeValues> for MeshAttributeRef<'a> {
    fn from(value: &'a MeshAttributeValues) -> Self {
        match value {
            MeshAttributeValues::Float32x2(v) => Self::Float32x2(v),
            MeshAttributeValues::Float32x3(v) => Self::Float32x3(v),
            MeshAttributeValues::Float32x4(v) => Self::Float32x4(v),
            MeshAttributeValues::Uint16x4(v) => Self::Uint16x4(v),
            MeshAttributeValues::Uint32x4(v) => Self::Uint32x4(v),
        }
    }
}
struct MeshAttributesRef<'a>(&'a BTreeMap<String, MeshAttributeValues>);
impl Serialize for MeshAttributesRef<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0 {
            map.serialize_entry(key, &MeshAttributeRef::from(value))?;
        }
        map.end()
    }
}
#[derive(Serialize)]
enum MeshIndicesRef<'a> {
    U16(&'a [u16]),
    U32(&'a [u32]),
}
#[derive(Serialize)]
struct MorphTargetRef<'a> {
    name: &'a Option<String>,
    attributes: MeshAttributesRef<'a>,
}
struct MorphTargetsRef<'a>(&'a [MeshMorphTargetAsset]);
impl Serialize for MorphTargetsRef<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for target in self.0 {
            seq.serialize_element(&MorphTargetRef {
                name: &target.name,
                attributes: MeshAttributesRef(&target.attributes),
            })?;
        }
        seq.end()
    }
}
