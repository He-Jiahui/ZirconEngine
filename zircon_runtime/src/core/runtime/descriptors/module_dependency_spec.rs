use serde::{Deserialize, Serialize};

/// 模块层的启动先决条件，也授权跨模块服务依赖边。
///
/// 冻结图会拒绝缺失、重复或违反初始化层级的边；卸载时按反向依赖保护提供方。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDependencySpec {
    /// 先决模块的原始名称；构造/反序列化不查询图，冻结时按该名称寻找目标。
    pub module_name: String,
}

impl ModuleDependencySpec {
    /// 取得 `module_name` 的所有权并返回声明；重复、缺失和层级错误留待图冻结报告。
    pub fn named(module_name: impl Into<String>) -> Self {
        Self {
            module_name: module_name.into(),
        }
    }
}
