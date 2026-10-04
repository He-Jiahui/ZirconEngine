use serde::{Deserialize, Serialize};

use crate::reflect::ReflectTypeRegistration;

/// One neutral reflection registration and its explicit schema dependencies.
/// 依赖使用完整类型路径；整批建目录时依赖须包含在批次中且整体无环，增量插入时依赖须已先入目录。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectSchemaCatalogEntry {
    pub registration: ReflectTypeRegistration,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

impl ReflectSchemaCatalogEntry {
    pub fn new(registration: ReflectTypeRegistration) -> Self {
        Self {
            registration,
            dependencies: Vec::new(),
        }
    }

    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }
}
