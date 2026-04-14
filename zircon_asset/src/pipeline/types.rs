//! Asset path sources, vertex layout, and request/payload types.

use serde::{Deserialize, Serialize};
use zircon_math::{Vec2, Vec3};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TextureSource {
    BuiltinChecker,
    BuiltinGrid,
    Path(String),
}

impl TextureSource {
    pub fn label(&self) -> String {
        match self {
            Self::BuiltinChecker => "builtin://checker".to_string(),
            Self::BuiltinGrid => "builtin://grid".to_string(),
            Self::Path(path) => path.clone(),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MeshSource {
    BuiltinCube,
    Path(String),
}

impl MeshSource {
    pub fn label(&self) -> String {
        match self {
            Self::BuiltinCube => "builtin://cube".to_string(),
            Self::Path(path) => path.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl MeshVertex {
    pub fn new(position: Vec3, normal: Vec3, uv: Vec2) -> Self {
        Self {
            position: position.to_array(),
            normal: normal.to_array(),
            uv: uv.to_array(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CpuTexturePayload {
    pub source: TextureSource,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct CpuMeshPayload {
    pub source: MeshSource,
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

#[derive(Clone, Debug)]
pub enum CpuAssetPayload {
    Texture(CpuTexturePayload),
    Mesh(CpuMeshPayload),
    Failure {
        request: AssetRequest,
        message: String,
    },
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum AssetRequest {
    Texture(TextureSource),
    Mesh(MeshSource),
}
