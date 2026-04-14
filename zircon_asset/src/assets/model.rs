use serde::{Deserialize, Serialize};

use crate::{AssetUri, MeshVertex};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelPrimitiveAsset {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelAsset {
    pub uri: AssetUri,
    pub primitives: Vec<ModelPrimitiveAsset>,
}
