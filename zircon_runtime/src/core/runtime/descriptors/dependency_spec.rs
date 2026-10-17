use serde::{Deserialize, Serialize};

use super::RegistryName;

/// 服务对另一已注册服务的显式依赖声明。
///
/// 冻结模块图时先验证名称、类别和跨模块边，再由解析器按依赖顺序实例化。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencySpec {
    /// 依赖提供方的规范注册名；名称合法不表示提供方已注册或当前可调用。
    pub name: RegistryName,
}

impl DependencySpec {
    /// 保存 `name` 并返回依赖声明；存在性、类别及跨模块授权由后续注册/冻结流程核对。
    pub fn named(name: RegistryName) -> Self {
        Self { name }
    }
}
