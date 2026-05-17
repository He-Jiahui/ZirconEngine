use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialTextureSlotValue {
    #[serde(default, flatten, skip_serializing_if = "Option::is_none")]
    pub reference: Option<AssetReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
}

impl MaterialTextureSlotValue {
    pub fn new(reference: AssetReference) -> Self {
        Self {
            reference: Some(reference),
            fallback: None,
        }
    }
}
