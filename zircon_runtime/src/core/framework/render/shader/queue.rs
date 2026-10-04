use serde::{Deserialize, Serialize};

/// shader 提供给材质默认渲染队列的建议值；材质解析时再映射为具体渲染阶段。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderQueueDescriptor {
    pub segment: ShaderQueueSegment,
    #[serde(default)]
    pub offset: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderQueueSegment {
    Background,
    Opaque,
    AlphaTest,
    Transparent,
    Overlay,
}
