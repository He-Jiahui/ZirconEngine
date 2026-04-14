use serde::{Deserialize, Serialize};

use crate::AssetUri;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShaderAsset {
    pub uri: AssetUri,
    pub source: String,
}
