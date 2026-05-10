use serde::{Deserialize, Serialize};

use super::RenderShaderStage;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderShaderEntryPointDescriptor {
    pub name: String,
    pub stage: RenderShaderStage,
}
