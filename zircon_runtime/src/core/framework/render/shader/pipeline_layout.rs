use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderShaderBindGroupLayoutDescriptor {
    pub group: u32,
    pub label: Option<String>,
    pub bindings: Vec<RenderShaderBindingDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderShaderBindingDescriptor {
    pub binding: u32,
    pub label: Option<String>,
    pub resource_type: RenderShaderBindingResourceType,
    pub visibility: Vec<super::RenderShaderStage>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderShaderBindingResourceType {
    UniformBuffer,
    StorageBuffer,
    Texture,
    StorageTexture,
    Sampler,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderShaderPipelineLayoutDescriptor {
    pub bind_groups: Vec<RenderShaderBindGroupLayoutDescriptor>,
    pub push_constant_ranges: Vec<String>,
}
