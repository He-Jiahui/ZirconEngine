use serde::{Deserialize, Serialize};

use crate::core::resource::{AssetReference, ResourceKind};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderShaderDependency {
    pub kind: ResourceKind,
    pub reference: AssetReference,
}

impl RenderShaderDependency {
    pub fn new(kind: ResourceKind, reference: AssetReference) -> Self {
        Self { kind, reference }
    }
}
