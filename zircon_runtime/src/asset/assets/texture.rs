use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureAsset {
    pub uri: AssetUri,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    #[serde(default = "default_texture_payload")]
    pub payload: TexturePayload,
}

fn default_texture_payload() -> TexturePayload {
    TexturePayload::Rgba8
}
