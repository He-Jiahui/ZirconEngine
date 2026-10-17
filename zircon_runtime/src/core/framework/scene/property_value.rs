use serde::{Deserialize, Serialize};

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::Real;

use super::{ComponentPropertyPath, EntityId};

/// 世界属性读取、动画采样和编辑器检查共用的值域，不代表组件存储本身。
///
/// 拥有属性值的共享交换格式，serde 使用外部标记和蛇形小写变体名。
/// 枚举构造及回读不验证数值有限性、四元数可归一化性或实体/资源可达性，写入消费方另作准入。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenePropertyValue {
    /// 布尔值。
    Bool(bool),
    /// 有符号 64 位整数，目标字段的窄化转换由写入器检查。
    Integer(i64),
    /// 无符号 64 位整数，目标字段的范围由写入器检查。
    Unsigned(u64),
    /// 单个 `Real` 标量。
    Scalar(Real),
    /// 两个分量，分量语义由目标字段定义。
    Vec2([Real; 2]),
    /// 三个分量，分量语义由目标字段定义。
    Vec3([Real; 3]),
    /// 四个分量的向量。
    Vec4([Real; 4]),
    /// 四元数分量数组，构造本变体不会归一化或拒绝零长度值。
    Quaternion([Real; 4]),
    /// 拥有型普通字符串。
    String(String),
    /// 拥有型枚举文本，合法取值由目标字段判断。
    Enum(String),
    /// 可选实体键，`None` 表示未指定引用。
    Entity(Option<EntityId>),
    /// 拥有型资源标识文本，不延长资源寿命。
    Resource(String),
    /// 动画参数值域，保留其自身的类型与数值宽度。
    AnimationParameter(AnimationParameterValue),
}

/// World 枚举属性时形成的拥有型条目，不借用组件存储。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ScenePropertyEntry {
    /// 条目的组件及属性定位路径。
    pub property_path: ComponentPropertyPath,
    /// 枚举时取得的属性值副本。
    pub value: ScenePropertyValue,
    /// 消费方声明的可动画标志，实际绑定支持仍由编译器判断。
    pub animatable: bool,
}

impl ScenePropertyEntry {
    /// 接管路径和值并保存可动画标志，返回独立条目，不执行字段查找或写入。
    pub(crate) fn new(
        property_path: ComponentPropertyPath,
        value: ScenePropertyValue,
        animatable: bool,
    ) -> Self {
        Self {
            property_path,
            value,
            animatable,
        }
    }
}
