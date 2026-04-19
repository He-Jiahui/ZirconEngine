use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureAsset {
    pub uri: AssetUri,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
