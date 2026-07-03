use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderResourceDescriptor {
    pub name: String,
    pub kind: ShaderResourceKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access: Option<ShaderResourceAccess>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderResourceKind {
    UniformBuffer,
    StorageBuffer,
    Texture,
    StorageTexture,
    Sampler,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderResourceAccess {
    Read,
    ReadWrite,
    Write,
}
