use serde::{Deserialize, Serialize};

/// 模块层的启动先决条件，也授权跨模块服务依赖边。
///
/// 冻结图会拒绝缺失、重复或违反初始化层级的边；卸载时按反向依赖保护提供方。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDependencySpec {
    pub module_name: String,
}

impl ModuleDependencySpec {
    pub fn named(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
        }
    }
}
