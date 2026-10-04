//! 源语言标记随 shader 文档进入导入与运行时源选择；默认语言属于持久化兼容契约，不应由文件扩展名隐式覆盖。

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderSourceLanguage {
    Wgsl,
    Glsl,
    SpirV,
    Hlsl,
    Cg,
}

impl ShaderSourceLanguage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Wgsl => "wgsl",
            Self::Glsl => "glsl",
            Self::SpirV => "spir_v",
            Self::Hlsl => "hlsl",
            Self::Cg => "cg",
        }
    }
}

pub fn default_shader_language() -> ShaderSourceLanguage {
    ShaderSourceLanguage::Wgsl
}
