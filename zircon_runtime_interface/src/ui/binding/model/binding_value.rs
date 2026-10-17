//! 拥有型 UI 绑定值及其结构化编码入口。
//!
//! 模板、Editor 与 Runtime 在此交换值树；私有 wire 载荷先反序列化成拥有数据，再进行整树检查。
//! 手工构造与不同便利构造器的准入范围并不相同，发布复合值时仍应明确验证完整根值。
//! JSON 回调投影和原生文本分别由子模块提供，均不承担处理器注册、资源加载或提供方查询。

use std::collections::BTreeMap;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

#[path = "binding_value/projection.rs"]
mod projection;
#[path = "binding_value/types.rs"]
mod types;
#[path = "binding_value/validation.rs"]
mod validation;

pub use types::{
    UiBindingAssetReference, UiBindingCollectionView, UiBindingEntityReference, UiBindingEnumValue,
    UiBindingMap, UiBindingMapKey, UiBindingValueBudget, UiBindingValueIdentityKind,
    UiBindingValueValidationError, UI_BINDING_COLLECTION_VIEW_MAX_LENGTH,
    UI_BINDING_VALUE_IDENTITY_MAX_BYTES, UI_BINDING_VALUE_MAX_COLLECTION_ENTRIES,
    UI_BINDING_VALUE_MAX_DEPTH, UI_BINDING_VALUE_MAX_NODES, UI_BINDING_VALUE_MAX_STRING_BYTES,
};

/// 模板、Runtime 与 Editor 共用的绑定值协议，用于传递事件参数和模型数据。
/// 手工构造枚举或调用 `array` 不校验资源预算；将值写入资产或用作事件参数前应校验整棵值树。
///
/// # 所有权与编码
/// 所有字符串、容器和扩展载荷均由值树拥有；借用访问器的结果随该树存活。
/// Serde 编码保留枚举类别；回调 JSON 投影是另一种形状，不能当作此结构化协议的反序列化输入。
///
/// # 准入边界
/// 公开变体允许未经检查的手工组装；仅包裹、克隆或序列化不表示整树已通过预算与身份检查。
/// 参见 [UiBindingValue::validate] 和 [UiBindingValue::validate_with_budget]。
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum UiBindingValue {
    /// 拥有的 UTF-8 字符串；字节累计由整树校验决定。
    String(String),
    /// 完整无符号整数；便利构造方法只接收其中的较窄输入范围。
    Unsigned(u64),
    /// 有符号整数；原生文本未单独保存非负整数的有符号类别。
    Signed(i64),
    /// 浮点标量；手工构造允许非有限值，准入校验会拒绝。
    Float(f64),
    /// 布尔标量。
    Bool(bool),
    /// 显式空值，与结构化编码中的空可选载荷类别不同。
    Null,
    /// 有序拥有子值；构造此变体不自动执行预算检查。
    Array(Vec<UiBindingValue>),
    /// 按字符串字段排序的拥有记录；字段身份也纳入校验。
    Record(BTreeMap<String, UiBindingValue>),
    /// 保留异构键类别的拥有映射；其内部顺序由键排序决定。
    Map(UiBindingMap),
    /// 类型标识、变体与可选拥有载荷；不在此解析成反射类型。
    Enum(UiBindingEnumValue),
    /// 拥有定位字符串的资产引用；不持有已加载资源。
    Asset(UiBindingAssetReference),
    /// 实体编号与非零代际的引用值；不验证实体当前是否存在。
    Entity(UiBindingEntityReference),
    /// 可缺省的拥有子值；存在时子值占用下一层深度。
    Optional(Option<Box<UiBindingValue>>),
    /// 拥有提供方、模式键及受限窗口描述；不包含实际集合行。
    CollectionView(UiBindingCollectionView),
}

/// 反序列化的临时拥有载荷，形状与公共枚举一致。
/// 转换只移动字段；成功交付公共值之前，由其自定义反序列化入口检查完整根值。
#[derive(Deserialize)]
enum UiBindingValueWire {
    String(String),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Bool(bool),
    Null,
    Array(Vec<UiBindingValue>),
    Record(BTreeMap<String, UiBindingValue>),
    Map(UiBindingMap),
    Enum(UiBindingEnumValue),
    Asset(UiBindingAssetReference),
    Entity(UiBindingEntityReference),
    Optional(Option<Box<UiBindingValue>>),
    CollectionView(UiBindingCollectionView),
}

/// 将临时载荷逐变体移入公共值，不复制树也不在此执行准入检查。
impl From<UiBindingValueWire> for UiBindingValue {
    fn from(value: UiBindingValueWire) -> Self {
        match value {
            UiBindingValueWire::String(value) => Self::String(value),
            UiBindingValueWire::Unsigned(value) => Self::Unsigned(value),
            UiBindingValueWire::Signed(value) => Self::Signed(value),
            UiBindingValueWire::Float(value) => Self::Float(value),
            UiBindingValueWire::Bool(value) => Self::Bool(value),
            UiBindingValueWire::Null => Self::Null,
            UiBindingValueWire::Array(value) => Self::Array(value),
            UiBindingValueWire::Record(value) => Self::Record(value),
            UiBindingValueWire::Map(value) => Self::Map(value),
            UiBindingValueWire::Enum(value) => Self::Enum(value),
            UiBindingValueWire::Asset(value) => Self::Asset(value),
            UiBindingValueWire::Entity(value) => Self::Entity(value),
            UiBindingValueWire::Optional(value) => Self::Optional(value),
            UiBindingValueWire::CollectionView(value) => Self::CollectionView(value),
        }
    }
}

// TODO: [CR-UITREE-0004] serde 与原生绑定解析目前先构造复合值再校验预算；
// 若入口接收不可信的大输入，需在解析期间限制深度、元素数与字节数，避免拒绝前大量分配。
/// 反序列化并检查完整结果；先完成拥有载荷分配，再把校验失败转换为格式错误。
/// 嵌套公共子值也有各自的反序列化入口，但仍需根值检查累计预算；既有分配时机问题见上方标记。
impl<'de> Deserialize<'de> for UiBindingValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Self::from(UiBindingValueWire::deserialize(deserializer)?);
        value.validate().map_err(D::Error::custom)?;
        Ok(value)
    }
}

impl UiBindingValue {
    /// 移入或转换一个字符串并返回值，不执行字节预算校验。
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// 将较窄的无符号输入无损提升到本协议的完整无符号整数载荷。
    pub fn unsigned(value: u32) -> Self {
        Self::Unsigned(value as u64)
    }

    /// 仅构造数组；与 `record`、`map` 不同，调用方须在发布前校验整个结果。
    ///
    /// # 参数与生命周期
    /// `values` 转为拥有向量后移入结果；子值和结果不借用调用方容器。
    pub fn array(values: impl Into<Vec<UiBindingValue>>) -> Self {
        Self::Array(values.into())
    }

    /// 移入记录字段，并按标准预算校验作为根值的整个结果。
    ///
    /// # 返回与错误
    /// 成功返回拥有记录；字段身份、子值、深度或累计预算失败时返回首个校验错误。
    /// 校验失败后输入字段也随局部值释放，不返回部分记录。
    pub fn record(
        fields: BTreeMap<String, UiBindingValue>,
    ) -> Result<Self, UiBindingValueValidationError> {
        let value = Self::Record(fields);
        value.validate()?;
        Ok(value)
    }

    /// 消费键值条目并构造标准预算内的映射根值。
    ///
    /// # 参数与返回
    /// 先按完整键类别与内容拒绝重复键，再验证包含所有键和值的完整结果。
    /// 成功返回拥有映射；失败返回首个重复键或值树校验错误，已消费条目不会返还。
    pub fn map(
        entries: impl IntoIterator<Item = (UiBindingMapKey, UiBindingValue)>,
    ) -> Result<Self, UiBindingValueValidationError> {
        let value = Self::Map(UiBindingMap::try_from_entries(entries)?);
        value.validate()?;
        Ok(value)
    }

    /// 仅在字符串变体中返回借用切片；其他类别返回缺省值。
    /// 返回借用不能超过当前值的生命周期，不分配或转换其他类别。
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// 读取能无损表示为较窄无符号整数的整型值。
    /// 无符号过大、有符号为负或过大、以及其他变体均返回缺省值；不把浮点数强制转换成整数。
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Self::Unsigned(value) => (*value).try_into().ok(),
            Self::Signed(value) if *value >= 0 => (*value as u64).try_into().ok(),
            _ => None,
        }
    }
}
