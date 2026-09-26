use serde::{Deserialize, Serialize};

use super::RegistryName;

/// 服务对另一已注册服务的显式依赖声明。
///
/// 冻结模块图时先验证名称、类别和跨模块边，再由解析器按依赖顺序实例化。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencySpec {
    pub name: RegistryName,
}

impl DependencySpec {
    pub fn named(name: RegistryName) -> Self {
        Self { name }
    }
}
