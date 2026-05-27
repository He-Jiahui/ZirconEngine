use serde::{Deserialize, Serialize};

use super::RenderShaderDefinitionValue;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderShaderVariantKey {
    pub entry_point: Option<String>,
    pub stage: Option<String>,
    pub defines: Vec<RenderShaderDefinitionValue>,
}
