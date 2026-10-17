use serde::{Deserialize, Serialize};

use crate::asset::{
    AssetReference, AssetUri, MeshSdfAsset, MeshVertex, ModelAsset, ModelPrimitiveAsset,
    VirtualGeometryAsset,
};

// 模型子网格与引用按库工件顺序格式保存；项目加载路径会把它恢复为 ModelAsset。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(in crate::asset::artifact) struct ArtifactCacheModelAsset {
    uri: AssetUri,
    primitives: Vec<ArtifactCacheModelPrimitiveAsset>,
}

#[cfg(test)]
impl From<&ModelAsset> for ArtifactCacheModelAsset {
    fn from(asset: &ModelAsset) -> Self {
        Self {
            uri: asset.uri.clone(),
            primitives: asset
                .primitives
                .iter()
                .map(ArtifactCacheModelPrimitiveAsset::from)
                .collect(),
        }
    }
}

impl ArtifactCacheModelAsset {
    pub(super) fn into_asset(self) -> ModelAsset {
        ModelAsset {
            uri: self.uri,
            primitives: self
                .primitives
                .into_iter()
                .map(ArtifactCacheModelPrimitiveAsset::into_asset)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArtifactCacheModelPrimitiveAsset {
    vertices: Vec<MeshVertex>,
    indices: Vec<u32>,
    mesh: Option<AssetReference>,
    mesh_sdf: Option<MeshSdfAsset>,
    virtual_geometry: Option<VirtualGeometryAsset>,
}

#[cfg(test)]
impl From<&ModelPrimitiveAsset> for ArtifactCacheModelPrimitiveAsset {
    fn from(asset: &ModelPrimitiveAsset) -> Self {
        Self {
            vertices: asset.vertices.clone(),
            indices: asset.indices.clone(),
            mesh: asset.mesh.clone(),
            mesh_sdf: asset.mesh_sdf.clone(),
            virtual_geometry: asset.virtual_geometry.clone(),
        }
    }
}

impl ArtifactCacheModelPrimitiveAsset {
    fn into_asset(self) -> ModelPrimitiveAsset {
        ModelPrimitiveAsset {
            vertices: self.vertices,
            indices: self.indices,
            mesh: self.mesh,
            mesh_sdf: self.mesh_sdf,
            virtual_geometry: self.virtual_geometry,
        }
    }
}

// The field order deliberately matches the owned cache DTO above. Slices and
// options of references use the same positional representation without cloning payloads.
#[derive(Serialize)]
pub(super) struct ArtifactCacheModelAssetRef<'a> {
    uri: &'a AssetUri,
    primitives: ArtifactCacheModelPrimitivesRef<'a>,
}

impl<'a> ArtifactCacheModelAssetRef<'a> {
    pub(super) fn new(asset: &'a ModelAsset) -> Self {
        Self {
            uri: &asset.uri,
            primitives: ArtifactCacheModelPrimitivesRef(&asset.primitives),
        }
    }
}

struct ArtifactCacheModelPrimitivesRef<'a>(&'a [ModelPrimitiveAsset]);

impl Serialize for ArtifactCacheModelPrimitivesRef<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for primitive in self.0 {
            sequence.serialize_element(&ArtifactCacheModelPrimitiveRef {
                vertices: &primitive.vertices,
                indices: &primitive.indices,
                mesh: primitive.mesh.as_ref(),
                mesh_sdf: primitive.mesh_sdf.as_ref(),
                virtual_geometry: primitive.virtual_geometry.as_ref(),
            })?;
        }
        sequence.end()
    }
}

#[derive(Serialize)]
struct ArtifactCacheModelPrimitiveRef<'a> {
    vertices: &'a [MeshVertex],
    indices: &'a [u32],
    mesh: Option<&'a AssetReference>,
    mesh_sdf: Option<&'a MeshSdfAsset>,
    virtual_geometry: Option<&'a VirtualGeometryAsset>,
}

#[cfg(test)]
#[path = "model/tests.rs"]
mod tests;

#[cfg(all(test, feature = "memory-validation"))]
#[path = "model/memory_profile.rs"]
mod memory_profile;
