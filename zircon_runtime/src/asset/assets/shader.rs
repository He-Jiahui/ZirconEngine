use serde::{Deserialize, Serialize};

use crate::asset::AssetUri;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShaderAsset {
    pub uri: AssetUri,
    pub source: String,
}
