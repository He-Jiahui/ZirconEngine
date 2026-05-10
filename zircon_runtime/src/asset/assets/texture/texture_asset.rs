use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;
use crate::core::framework::render::RenderImageDescriptor;

use super::{metadata, payload::default_texture_payload, TexturePayload};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureAsset {
    pub uri: AssetUri,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    #[serde(default = "default_texture_payload")]
    pub payload: TexturePayload,
}

impl TextureAsset {
    pub fn render_image_descriptor(&self) -> RenderImageDescriptor {
        metadata::render_image_descriptor(self)
    }
}
