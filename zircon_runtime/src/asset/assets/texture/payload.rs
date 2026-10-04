//! 纹理 payload 区分原始像素与容器编码；上传规划根据表示和设备能力选择路径，不能仅凭扩展名推断可上传性。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TexturePayload {
    Rgba8,
    Container {
        format: String,
        bytes: Vec<u8>,
        mip_count: u32,
        array_layers: u32,
    },
}

pub fn default_texture_payload() -> TexturePayload {
    TexturePayload::Rgba8
}
