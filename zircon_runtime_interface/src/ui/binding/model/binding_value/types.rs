//! 绑定值的预算、类型化容器和轻量引用载荷。
//!
//! 本模块拥有身份字符串、映射条目与窗口描述；构造器只落实各自明确的局部不变量。
//! 这些载荷不注册类型、不访问资产或实体，也不向提供方请求集合数据。
//! Serde 的借用序列化视图仅在调用期间有效；反序列化 wire 载荷会被移入校验后的拥有类型。
//! 完整根值预算由相邻验证模块负责，局部容器或身份构造成功不等于整树准入。

use std::fmt::Write as _;
use std::{
    collections::{btree_map::Entry, BTreeMap},
    fmt,
};

use serde::{de::Error as _, ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use thiserror::Error;

use super::{
    super::{UiModelProviderKey, UiModelSchemaKey},
    UiBindingValue,
};

/// 标准值树从根为第一层计算的最大深度。
pub const UI_BINDING_VALUE_MAX_DEPTH: usize = 64;
/// 标准值树累计节点上限；映射键也各计一个节点。
pub const UI_BINDING_VALUE_MAX_NODES: usize = 1_024;
/// 标准值树累计 UTF-8 字符串字节上限，包含相应身份与字符串键。
pub const UI_BINDING_VALUE_MAX_STRING_BYTES: usize = 16 * 1_024;
/// 标准校验中单个数组、记录或映射的条目上限。
pub const UI_BINDING_VALUE_MAX_COLLECTION_ENTRIES: usize = 256;
/// 独立于可调树预算的单个身份字符串字节上限。
pub const UI_BINDING_VALUE_IDENTITY_MAX_BYTES: usize = 256;
/// 独立于树预算的集合窗口最大行数。
pub const UI_BINDING_COLLECTION_VIEW_MAX_LENGTH: u32 = 256;

/// 约束整个 binding 值树的资源预算；验证会累计节点与字符串字节，并限制嵌套深度及各容器项数。
///
/// # 使用与约束
/// 按值复制到一次根值校验；所有上限直接采用调用方数值，零值也按同一比较规则生效。
/// 此类型只保存上限，不预留内存、不累计状态，也不保证构造它时已有值树满足预算。
/// 身份字符串和集合窗口另有固定领域上限。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingValueBudget {
    /// 从根深度一开始计算的最大层数。
    pub max_depth: usize,
    /// 本次完整值树累计节点数上限，包含映射键。
    pub max_nodes: usize,
    /// 本次完整值树累计字符串的 UTF-8 字节数上限。
    pub max_string_bytes: usize,
    /// 每个可变长容器允许的最大条目数，不是全树条目总数。
    pub max_collection_entries: usize,
}

impl UiBindingValueBudget {
    /// 标准准入上限；供默认构造与无显式预算的根值校验使用。
    pub const STANDARD: Self = Self::new(
        UI_BINDING_VALUE_MAX_DEPTH,
        UI_BINDING_VALUE_MAX_NODES,
        UI_BINDING_VALUE_MAX_STRING_BYTES,
        UI_BINDING_VALUE_MAX_COLLECTION_ENTRIES,
    );

    /// 按给定单位保存四项上限，返回可复制的预算。
    /// 不校验或截断参数；调用方可收紧或放宽可调上限，固定身份和窗口上限仍独立生效。
    pub const fn new(
        max_depth: usize,
        max_nodes: usize,
        max_string_bytes: usize,
        max_collection_entries: usize,
    ) -> Self {
        Self {
            max_depth,
            max_nodes,
            max_string_bytes,
            max_collection_entries,
        }
    }
}

/// 默认使用标准上限，不继承前一次校验的累计状态。
impl Default for UiBindingValueBudget {
    fn default() -> Self {
        Self::STANDARD
    }
}

/// 标明身份或版本约束错误所指的领域，供错误显示和调用方区分原因。
/// 它本身不验证字符串，也不查找对应资源。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiBindingValueIdentityKind {
    /// 记录中的字段名。
    RecordField,
    /// 枚举的类型身份字符串。
    EnumType,
    /// 枚举的变体身份字符串。
    EnumVariant,
    /// 资产定位字符串。
    AssetLocator,
    /// 实体引用的代际值。
    EntityGeneration,
    /// 集合窗口对应的修订值。
    CollectionRevision,
}

/// 把领域种类写为错误消息中的固定名称，不查询或分配领域对象。
impl fmt::Display for UiBindingValueIdentityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::RecordField => "record field",
            Self::EnumType => "enum type",
            Self::EnumVariant => "enum variant",
            Self::AssetLocator => "asset locator",
            Self::EntityGeneration => "entity generation",
            Self::CollectionRevision => "collection revision",
        })
    }
}

/// 拥有型校验错误，区分局部身份不变量与整树资源预算。
///
/// # 载荷
/// 计数字段记录发现首错时的实际值和上限；字节以 UTF-8 长度计，深度从根一开始。
/// 错误不借用输入树，不含资源加载状态，也不表示同时报告全部违例。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum UiBindingValueValidationError {
    /// 指定领域身份字符串为空。
    #[error("{kind} cannot be empty")]
    EmptyIdentity { kind: UiBindingValueIdentityKind },
    /// 单个身份的字节长度超过固定上限。
    #[error("{kind} uses {actual_bytes} bytes, exceeding the {maximum_bytes}-byte limit")]
    IdentityTooLong {
        kind: UiBindingValueIdentityKind,
        actual_bytes: usize,
        maximum_bytes: usize,
    },
    /// 要求非零的实体代际或集合修订为零。
    #[error("{kind} must be non-zero")]
    ZeroGeneration { kind: UiBindingValueIdentityKind },
    /// 树内出现非有限浮点值。
    #[error("binding value contains a non-finite float")]
    NonFiniteFloat,
    /// 当前递归层级超过本次深度上限。
    #[error("binding value depth {actual} exceeds the configured limit {maximum}")]
    DepthExceeded { actual: usize, maximum: usize },
    /// 累计节点数超过本次节点上限。
    #[error("binding value node count {actual} exceeds the configured limit {maximum}")]
    NodeBudgetExceeded { actual: usize, maximum: usize },
    /// 累计字符串字节数超过本次字节上限。
    #[error("binding value string bytes {actual} exceed the configured limit {maximum}")]
    StringBudgetExceeded { actual: usize, maximum: usize },
    /// 一个容器的条目数超过本次单容器上限。
    #[error(
        "binding value collection has {actual} entries, exceeding the configured limit {maximum}"
    )]
    CollectionEntriesExceeded { actual: usize, maximum: usize },
    /// 存在类别与内容完全相同的映射键。
    #[error("binding map contains a duplicate key")]
    DuplicateMapKey,
    /// 集合窗口行数超过固定最大值。
    #[error("collection view window length {actual} exceeds the supported limit {maximum}")]
    CollectionViewWindowExceeded { actual: u32, maximum: u32 },
    /// 窗口终点溢出或超过总行数，载荷保留输入标量。
    #[error(
        "collection view window offset {offset} and length {length} exceed total length {total_length}"
    )]
    InvalidCollectionViewWindow {
        offset: u64,
        length: u32,
        total_length: u64,
    },
}

/// 异构映射键；相等与排序同时保留类别及其内容。
/// 字符串、无符号、有符号和布尔键可以共存，重复键只指同一变体的相同内容。
/// 原生文本整数没有额外类别标签，结构化编码才保留此处的类别区分。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiBindingMapKey {
    String(String),
    Unsigned(u64),
    Signed(i64),
    Bool(bool),
}

impl UiBindingMapKey {
    #[cfg(test)]
    /// 测试用分配型投影入口；返回新文本，不改变键。
    pub(super) fn native_repr(&self) -> String {
        let mut output = String::new();
        self.native_repr_into(&mut output);
        output
    }

    /// 向调用方已有缓冲区追加键的原生字面量，不清空前缀或借用输出到返回之后。
    pub(super) fn native_repr_into(&self, output: &mut String) {
        match self {
            Self::String(value) => quoted_into(value, output),
            Self::Unsigned(value) => {
                let _ = write!(output, "{value}");
            }
            Self::Signed(value) => {
                let _ = write!(output, "{value}");
            }
            Self::Bool(value) => {
                let _ = write!(output, "{value}");
            }
        }
    }

    /// 为扩展映射的回调标签对象生成一个拥有 JSON 键值，不把它当作 JSON 对象字段名。
    pub(super) fn to_json_value(&self) -> Value {
        match self {
            Self::String(value) => Value::String(value.clone()),
            Self::Unsigned(value) => Value::Number((*value).into()),
            Self::Signed(value) => Value::Number((*value).into()),
            Self::Bool(value) => Value::Bool(*value),
        }
    }
}

/// 以异构类型键表达的 binding map；构造时拒绝重复键，内部排序使迭代和序列化顺序稳定。
///
/// # 所有权与准入
/// 拥有所有键值条目；借用迭代器和检索结果受映射自身生命周期限制。
/// 本类型的构造与反序列化只保证键唯一；值树累计预算由外层公共值根的校验落实。
#[derive(Clone, Debug, PartialEq)]
pub struct UiBindingMap(BTreeMap<UiBindingMapKey, UiBindingValue>);

impl UiBindingMap {
    /// 消费条目并建立有序映射，只检查重复键。
    ///
    /// # 返回与错误
    /// 成功拥有所有条目；遇到首个重复键即返回错误，已消费条目释放且不会部分返回。
    /// 不检查条目数预算、非有限值或嵌套树；需要完整准入时应包裹为公共值再校验。
    pub fn try_from_entries(
        entries: impl IntoIterator<Item = (UiBindingMapKey, UiBindingValue)>,
    ) -> Result<Self, UiBindingValueValidationError> {
        let mut values = BTreeMap::new();
        for (key, value) in entries {
            match values.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(value);
                }
                Entry::Occupied(_) => return Err(UiBindingValueValidationError::DuplicateMapKey),
            }
        }
        Ok(Self(values))
    }

    /// 按完整键类别和内容查找，返回随当前映射存活的只读值借用；不存在时返回缺省值。
    pub fn get(&self, key: &UiBindingMapKey) -> Option<&UiBindingValue> {
        self.0.get(key)
    }

    /// 按内部键序借用遍历全部条目；剩余长度可精确获知，迭代器不能超过映射生命周期。
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&UiBindingMapKey, &UiBindingValue)> {
        self.0.iter()
    }

    /// 返回当前拥有条目数，不遍历或验证子值。
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 报告当前是否没有条目，不表示其他预算或身份条件已校验。
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// 单次序列化期间的借用条目视图；键和值始终由原映射拥有。
#[derive(Serialize)]
struct UiBindingMapEntryRef<'a> {
    key: &'a UiBindingMapKey,
    value: &'a UiBindingValue,
}

/// 反序列化时的拥有条目；随后消费进映射，以再次检查键唯一性。
#[derive(Deserialize)]
struct UiBindingMapEntry {
    key: UiBindingMapKey,
    value: UiBindingValue,
}

// 用 entry 序列而非 JSON 对象编码，以保留字符串、整数和布尔键的类型；反序列化会再次检查重复键。
impl Serialize for UiBindingMap {
    /// 按原有键序直接向序列输出借用条目，不建立中间条目向量。
    /// 序列长度提示来自当前映射；序列化器错误立即透传，源映射不变。
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for (key, value) in &self.0 {
            sequence.serialize_element(&UiBindingMapEntryRef { key, value })?;
        }
        sequence.end()
    }
}

impl<'de> Deserialize<'de> for UiBindingMap {
    /// 先读取拥有条目序列，再消费条目建立去重映射。
    /// 重复键转为格式错误；嵌套公共值按各自入口校验，但此入口不额外累计完整映射预算。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries = Vec::<UiBindingMapEntry>::deserialize(deserializer)?;
        Self::try_from_entries(entries.into_iter().map(|entry| (entry.key, entry.value)))
            .map_err(D::Error::custom)
    }
}

/// 拥有枚举类型标识、变体名与可选载荷的协议值。
/// 构造器只检查两个身份字符串；不按反射模式解析变体，也不独立校验整个可选载荷树。
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UiBindingEnumValue {
    type_id: String,
    variant: String,
    payload: Option<Box<UiBindingValue>>,
}

impl UiBindingEnumValue {
    /// 移入身份和可选载荷，检查身份非空与固定字节上限。
    ///
    /// # 返回与生命周期
    /// 成功结果拥有两个字符串和装箱后的子值，不借用输入；身份失败返回首个校验错误。
    /// 可选载荷的预算与数值约束须由外层公共值的整树校验确认。
    pub fn try_new(
        type_id: impl Into<String>,
        variant: impl Into<String>,
        payload: Option<UiBindingValue>,
    ) -> Result<Self, UiBindingValueValidationError> {
        let type_id = type_id.into();
        let variant = variant.into();
        validate_identity(UiBindingValueIdentityKind::EnumType, &type_id)?;
        validate_identity(UiBindingValueIdentityKind::EnumVariant, &variant)?;
        Ok(Self {
            type_id,
            variant,
            payload: payload.map(Box::new),
        })
    }

    /// 借用拥有的类型身份；结果不能超过当前枚举载荷的生命周期。
    pub fn type_id(&self) -> &str {
        &self.type_id
    }

    /// 借用拥有的变体身份，不执行模式查找。
    pub fn variant(&self) -> &str {
        &self.variant
    }

    /// 借用可选子值；缺省时无子值，不为调用方克隆载荷。
    pub fn payload(&self) -> Option<&UiBindingValue> {
        self.payload.as_deref()
    }
}

/// 拥有型 wire 形状；转换时移动标识和子值到公开载荷。
#[derive(Deserialize)]
struct UiBindingEnumValueWire {
    type_id: String,
    variant: String,
    payload: Option<Box<UiBindingValue>>,
}

impl<'de> Deserialize<'de> for UiBindingEnumValue {
    /// 读取 wire 后复用局部身份构造约束，将违例转换成格式错误。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = UiBindingEnumValueWire::deserialize(deserializer)?;
        Self::try_new(wire.type_id, wire.variant, wire.payload.map(|value| *value))
            .map_err(D::Error::custom)
    }
}

/// 只拥有定位字符串的资产引用，结构化编码透明表示该字符串。
/// 不持有资源句柄；准入只要求非空和固定字节上限，不验证 URI 语法或实际存在性。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct UiBindingAssetReference(String);

impl UiBindingAssetReference {
    /// 移入定位字符串并验证局部身份约束；成功后由引用对象拥有，失败返回身份错误。
    pub fn try_new(locator: impl Into<String>) -> Result<Self, UiBindingValueValidationError> {
        let locator = locator.into();
        validate_identity(UiBindingValueIdentityKind::AssetLocator, &locator)?;
        Ok(Self(locator))
    }

    /// 借用定位字符串，不触发加载；借用受引用对象生命周期限制。
    pub fn locator(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for UiBindingAssetReference {
    /// 读取字符串并复用局部身份构造器，拒绝空或过长的定位文本。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::try_new(String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

/// 可复制的实体编号与代际载荷。
/// 只保证代际非零；不检查编号对应实体是否仍存在，也不延长实体生命周期。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct UiBindingEntityReference {
    entity_id: u64,
    generation: u64,
}

impl UiBindingEntityReference {
    /// 接受任意编号与非零代际并返回引用值；零代际返回领域错误，不访问实体注册表。
    pub fn try_new(entity_id: u64, generation: u64) -> Result<Self, UiBindingValueValidationError> {
        if generation == 0 {
            return Err(UiBindingValueValidationError::ZeroGeneration {
                kind: UiBindingValueIdentityKind::EntityGeneration,
            });
        }
        Ok(Self {
            entity_id,
            generation,
        })
    }

    /// 按值读取编号，不查找实体。
    pub const fn entity_id(self) -> u64 {
        self.entity_id
    }

    /// 按值读取已经过非零检查的代际，不代表当前实体状态。
    pub const fn generation(self) -> u64 {
        self.generation
    }
}

/// 实体引用的临时 wire 载荷；非零代际约束在转换时落实。
#[derive(Deserialize)]
struct UiBindingEntityReferenceWire {
    entity_id: u64,
    generation: u64,
}

impl<'de> Deserialize<'de> for UiBindingEntityReference {
    /// 读取 wire 后复用实体引用构造器，零代际转换为格式错误。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = UiBindingEntityReferenceWire::deserialize(deserializer)?;
        Self::try_new(wire.entity_id, wire.generation).map_err(D::Error::custom)
    }
}

/// 集合的拥有型窗口描述，不包含或借用集合行。
///
/// # 状态与生命周期
/// 提供方键、条目模式键和修订描述调用方选定的视图，引用本身不发起查询或维持外部会话。
/// 构造与反序列化保证非零修订、固定窗口行数上限和不越总长的终点；不确认提供方当前可用。
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UiBindingCollectionView {
    provider: UiModelProviderKey,
    item_schema: UiModelSchemaKey,
    revision: u64,
    offset: u64,
    length: u32,
    total_length: u64,
}

impl UiBindingCollectionView {
    /// 移入提供方与模式键，并校验指定窗口。
    ///
    /// # 参数与返回
    /// `revision` 必须非零；`offset`、`length`、`total_length` 均以行计，终点使用受检加法。
    /// 成功返回拥有描述，允许空窗口但其起点仍不得越过总长；失败返回首个窗口错误。
    /// 不分配实际行、不读取提供方内容，也不校验外部修订是否仍为最新。
    pub fn try_new(
        provider: UiModelProviderKey,
        item_schema: UiModelSchemaKey,
        revision: u64,
        offset: u64,
        length: u32,
        total_length: u64,
    ) -> Result<Self, UiBindingValueValidationError> {
        let value = Self {
            provider,
            item_schema,
            revision,
            offset,
            length,
            total_length,
        };
        value.validate_window()?;
        Ok(value)
    }

    /// 借用拥有的提供方键；不解析提供方或延长外部会话。
    pub fn provider(&self) -> &UiModelProviderKey {
        &self.provider
    }

    /// 借用拥有的条目模式键；不执行模式注册表查询。
    pub fn item_schema(&self) -> &UiModelSchemaKey {
        &self.item_schema
    }

    /// 读取此描述携带的非零修订，不查询当前提供方修订。
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// 读取窗口起始行偏移。
    pub const fn offset(&self) -> u64 {
        self.offset
    }

    /// 读取窗口行数，可能为零。
    pub const fn length(&self) -> u32 {
        self.length
    }

    /// 读取描述记录的集合总行数，不请求真实集合计数。
    pub const fn total_length(&self) -> u64 {
        self.total_length
    }

    /// 按固定次序检查修订、窗口长度和受检终点；只读描述，首错即返回。
    pub(super) fn validate_window(&self) -> Result<(), UiBindingValueValidationError> {
        if self.revision == 0 {
            return Err(UiBindingValueValidationError::ZeroGeneration {
                kind: UiBindingValueIdentityKind::CollectionRevision,
            });
        }
        if self.length > UI_BINDING_COLLECTION_VIEW_MAX_LENGTH {
            return Err(
                UiBindingValueValidationError::CollectionViewWindowExceeded {
                    actual: self.length,
                    maximum: UI_BINDING_COLLECTION_VIEW_MAX_LENGTH,
                },
            );
        }
        if self
            .offset
            .checked_add(u64::from(self.length))
            .is_none_or(|end| end > self.total_length)
        {
            return Err(UiBindingValueValidationError::InvalidCollectionViewWindow {
                offset: self.offset,
                length: self.length,
                total_length: self.total_length,
            });
        }
        Ok(())
    }
}

/// 集合窗口的拥有 wire 载荷；反序列化之后仍须复用窗口准入。
#[derive(Deserialize)]
struct UiBindingCollectionViewWire {
    provider: UiModelProviderKey,
    item_schema: UiModelSchemaKey,
    revision: u64,
    offset: u64,
    length: u32,
    total_length: u64,
}

impl<'de> Deserialize<'de> for UiBindingCollectionView {
    /// 移入 wire 字段并复用窗口构造器；格式解码成功不跳过领域窗口检查。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = UiBindingCollectionViewWire::deserialize(deserializer)?;
        Self::try_new(
            wire.provider,
            wire.item_schema,
            wire.revision,
            wire.offset,
            wire.length,
            wire.total_length,
        )
        .map_err(D::Error::custom)
    }
}

/// 借用身份文本，先拒绝空值，再比较固定 UTF-8 字节上限。
/// 只报告对应领域身份错误，不修剪、不规范化，也不检查标识语法或资源存在性。
pub(super) fn validate_identity(
    kind: UiBindingValueIdentityKind,
    value: &str,
) -> Result<(), UiBindingValueValidationError> {
    if value.is_empty() {
        return Err(UiBindingValueValidationError::EmptyIdentity { kind });
    }
    if value.len() > UI_BINDING_VALUE_IDENTITY_MAX_BYTES {
        return Err(UiBindingValueValidationError::IdentityTooLong {
            kind,
            actual_bytes: value.len(),
            maximum_bytes: UI_BINDING_VALUE_IDENTITY_MAX_BYTES,
        });
    }
    Ok(())
}

#[cfg(test)]
/// 测试用字符串转义分配入口；结果独立拥有，转义集合与追加实现一致。
pub(super) fn quoted(value: &str) -> String {
    let mut output = String::new();
    quoted_into(value, &mut output);
    output
}

// 该转义集合必须与 parser.rs 的 parse_string 保持互逆；新增转义时两端需一起更新。
/// 把借用字符串按原生文本约定转义并追加到调用方缓冲区。
/// 仅处理反斜杠、双引号和三种控制字符；其他字符按标量写入，输出前缀保持。
/// 不使用 JSON 通用转义，也不把输入借用保留到返回之后。
pub(super) fn quoted_into(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character => output.push(character),
        }
    }
    output.push('"');
}

#[cfg(test)]
#[path = "../types/tests/map_serialize_perf.rs"]
mod map_serialize_performance_tests;
