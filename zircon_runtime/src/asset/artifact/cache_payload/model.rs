use serde::{Deserialize, Serialize};

use crate::asset::{
    AssetReference, AssetUri, MeshSdfAsset, MeshVertex, ModelAsset, ModelPrimitiveAsset,
    VirtualGeometryAsset,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(super) struct ArtifactCacheModelAsset {
    uri: AssetUri,
    primitives: Vec<ArtifactCacheModelPrimitiveAsset>,
}

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
