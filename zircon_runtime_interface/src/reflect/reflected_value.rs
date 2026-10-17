use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
/// 反射读写传输使用的带标签值；此枚举描述形状，不单独实施运行时深度、大小或字段类型预算。
///
/// # 使用时机与生命周期
/// 在反射字段读写、持久化和 VM 状态迁移之间传递拥有型值树；文本和容器由值持有。
/// `Entity` 携带可选实体 ID，不持有实体对象或保证它仍存在；`Resource` 的文本解析由使用方负责。
///
/// # 约束
/// 构造和派生 serde 只描述值形状，不代替运行时预算、字段类型或访问策略检查。
/// 值的准入方负责选择预算并调用校验，再按字段合同处理引用。
/// 派生 serde 使用 `kind`/`value` 标签；运行时持久化文档的版本由文档层管理。
///
/// # 参考
/// [`Self::validate_with_budget`]；[`super::ZrReflectValue`] 的字段转换合同。
pub enum ReflectedValue {
    /// 无载荷的空值。
    Null,
    /// 布尔值。
    Bool(bool),
    /// 有符号 64 位整数。
    Integer(i64),
    /// 无符号 64 位整数。
    Unsigned(u64),
    /// 单精度标量；有限性由运行时值校验路径检查。
    Scalar(f32),
    /// UTF-8 文本。
    String(String),
    /// 枚举成员名称。
    Enum(String),
    /// 二维向量分量。
    Vec2([f32; 2]),
    /// 三维向量分量。
    Vec3([f32; 3]),
    /// 四维向量分量。
    Vec4([f32; 4]),
    /// 四元数分量。
    Quaternion([f32; 4]),
    /// 可选实体标识。
    Entity(Option<u64>),
    /// 资源引用路径或标识文本。
    Resource(String),
    /// 有序的递归值列表。
    List(Vec<ReflectedValue>),
    /// 按键排序的递归值映射。
    Map(BTreeMap<String, ReflectedValue>),
    /// JSON 扩展值；运行时仍会对嵌套内容应用预算。
    Json(serde_json::Value),
}

impl ReflectedValue {
    /// 返回与当前变体对应的稳定类型标签，供诊断和类型错误上下文使用。
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "Null",
            Self::Bool(_) => "Bool",
            Self::Integer(_) => "Integer",
            Self::Unsigned(_) => "Unsigned",
            Self::Scalar(_) => "Scalar",
            Self::String(_) => "String",
            Self::Enum(_) => "Enum",
            Self::Vec2(_) => "Vec2",
            Self::Vec3(_) => "Vec3",
            Self::Vec4(_) => "Vec4",
            Self::Quaternion(_) => "Quaternion",
            Self::Entity(_) => "Entity",
            Self::Resource(_) => "Resource",
            Self::List(_) => "List",
            Self::Map(_) => "Map",
            Self::Json(_) => "Json",
        }
    }
}

impl crate::retained_memory::RetainedOwnerV1 for ReflectedValue {
    fn retained_bound_v1(
        &self,
        m: &mut crate::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), crate::retained_memory::RetentionContractError> {
        use crate::retained_memory::RetainedOwnerV1;
        m.nest(|m| match self {
            Self::String(v) | Self::Enum(v) | Self::Resource(v) => v.retained_bound_v1(m),
            Self::List(v) => v.retained_bound_v1(m),
            Self::Map(v) => v.retained_bound_v1(m),
            Self::Json(v) => v.retained_bound_v1(m),
            Self::Null
            | Self::Bool(_)
            | Self::Integer(_)
            | Self::Unsigned(_)
            | Self::Scalar(_)
            | Self::Vec2(_)
            | Self::Vec3(_)
            | Self::Vec4(_)
            | Self::Quaternion(_)
            | Self::Entity(_) => Ok(()),
        })
    }
}
unsafe impl crate::retained_memory::ImmutableRetainedOwnerV1 for ReflectedValue {}
