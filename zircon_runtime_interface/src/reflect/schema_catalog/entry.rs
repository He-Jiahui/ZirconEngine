//! 保存一个类型的中立注册元数据及其显式依赖，供目录接纳和快照传递。
//!
//! 构造、链式配置与 serde 解码只形成候选；身份唯一性、依赖闭包和预算在目录入口复核。

use serde::{Deserialize, Serialize};

use crate::reflect::ReflectTypeRegistration;

/// One neutral reflection registration and its explicit schema dependencies.
/// 依赖使用完整类型路径；整批建目录时依赖须包含在批次中且整体无环，增量插入时依赖须已先入目录。
///
/// # 使用时机与生命周期
/// 由注册端拥有后交给 [`super::ReflectSchemaCatalog::try_new`] 或 `try_insert`；目录持有规范化后的值。
/// 不包含对象适配器或实时对象引用，依赖不会从字段值类型自动推导。
///
/// # 参考
/// [`super::ReflectSchemaCatalog::validate_entry`] 只检查单项；`try_new` 另查跨类型身份与依赖图。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReflectSchemaCatalogEntry {
    /// 未接纳的类型描述；接纳保留字段声明顺序，并规范化别名集合。
    pub registration: ReflectTypeRegistration,
    /// 显式完整类型路径集合；接纳排序但拒绝重复、自依赖或缺失目标，不静默去重。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
}

impl ReflectSchemaCatalogEntry {
    /// 为注册描述建立无依赖的候选项。
    ///
    /// # 参数
    /// `registration` 的所有权移入候选；此处不验证字段身份或运行时值语义。
    ///
    /// # 返回值
    /// 返回可继续配置或交给目录接纳的项。
    ///
    /// # 参考
    /// [`Self::with_dependencies`]、[`super::ReflectSchemaCatalog::try_insert`]。
    pub fn new(registration: ReflectTypeRegistration) -> Self {
        Self {
            registration,
            dependencies: Vec::new(),
        }
    }

    /// 消费候选并替换其完整依赖列表，留待目录接纳时验证。
    ///
    /// # 参数
    /// `dependencies` 是完整类型路径列表；此处保留输入顺序与重复项。
    ///
    /// # 返回值
    /// 返回仍未接纳的候选，不修改任何已发布目录。
    ///
    /// # 参考
    /// [`super::ReflectSchemaCatalog::try_new`] 允许批内依赖乱序，`try_insert` 要求依赖已存在。
    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies = dependencies;
        self
    }
}
