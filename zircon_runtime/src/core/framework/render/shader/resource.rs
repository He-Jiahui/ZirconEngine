use serde::{Deserialize, Serialize};

/// 导入资产声明的逻辑资源需求；调用前仍需与 pass 绑定容量和访问权限契约核对。
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
