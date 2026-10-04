use serde::{Deserialize, Serialize};

use crate::core::resource::{AssetReference, ResourceKind};

/// shader 资产对其他资源的有类型引用；导入器据此构建依赖图并在资源变动时重新评估就绪状态。
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
