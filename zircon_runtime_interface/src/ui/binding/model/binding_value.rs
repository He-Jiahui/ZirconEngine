use std::collections::BTreeMap;

use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

mod projection;
mod types;
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
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum UiBindingValue {
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
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    pub fn unsigned(value: u32) -> Self {
        Self::Unsigned(value as u64)
    }

    /// 仅构造数组；与 `record`、`map` 不同，调用方须在发布前校验整个结果。
    pub fn array(values: impl Into<Vec<UiBindingValue>>) -> Self {
        Self::Array(values.into())
    }

    pub fn record(
        fields: BTreeMap<String, UiBindingValue>,
    ) -> Result<Self, UiBindingValueValidationError> {
        let value = Self::Record(fields);
        value.validate()?;
        Ok(value)
    }

    pub fn map(
        entries: impl IntoIterator<Item = (UiBindingMapKey, UiBindingValue)>,
    ) -> Result<Self, UiBindingValueValidationError> {
        let value = Self::Map(UiBindingMap::try_from_entries(entries)?);
        value.validate()?;
        Ok(value)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Self::Unsigned(value) => (*value).try_into().ok(),
            Self::Signed(value) if *value >= 0 => (*value as u64).try_into().ok(),
            _ => None,
        }
    }
}
