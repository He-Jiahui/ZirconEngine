use serde::{Deserialize, Serialize};

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
