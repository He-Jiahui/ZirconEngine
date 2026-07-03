use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderRenderStateDescriptor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cull_mode: Option<ShaderCullMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth_compare: Option<ShaderDepthCompare>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth_write: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blend: Option<ShaderBlendMode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderCullMode {
    None,
    Front,
    Back,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderDepthCompare {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderBlendMode {
    Opaque,
    AlphaBlend,
    Additive,
    PremultipliedAlpha,
}
