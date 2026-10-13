//! 定义规范化目录的版本化内容编码，供模式响应及目录快照比较。
//!
//! 完整路径、别名和依赖按接纳后的顺序编码；字段及枚举选项保留原有语义顺序。
//! 编码不含对象适配器、地址或注册代际，所有标签和字节序均属于算法契约。

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::reflect::{
    ReflectEditorHint, ReflectFieldInfo, ReflectNumericRange, ReflectScriptVisibility,
    ReflectSerializationStrategy, ReflectTypeKind, ReflectTypeRegistration, ReflectTypeRole,
    ReflectedValue,
};

use super::ReflectSchemaCatalogEntry;

/// 内容指纹的编码版本；改变字段编码或顺序规则时必须同时提升版本。
pub const REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION: u32 = 1;
/// 将此指纹与其他 BLAKE3 用途分域；域文本及终止字节参与实际编码。
const FINGERPRINT_DOMAIN: &[u8] = b"zircon-reflect-schema-catalog-v1\0";

/// 规范化反射目录元数据的 32 字节 BLAKE3 指纹，在查询响应与快照中标识同一目录内容。
///
/// # 使用时机与生命周期
/// [`super::ReflectSchemaCatalog::fingerprint`] 返回自有值，查询响应和快照可独立携带。
/// 私有元组字段持有原始摘要字节；展示/serde 使用十六进制文本。
/// 内容相同不证明运行时对象或旧槽位仍属于当前注册代际。
///
/// # 参考
/// [`super::ReflectSchemaCatalog::try_from_snapshot`] 从候选重算后比对；[`REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ReflectSchemaFingerprint([u8; blake3::OUT_LEN]);

impl ReflectSchemaFingerprint {
    /// 借用这份指纹的原始 32 字节摘要，生命周期依附 `self`；不返回十六进制文本。
    pub fn as_bytes(&self) -> &[u8; blake3::OUT_LEN] {
        &self.0
    }
}

// 展示和 serde 字符串共用 BLAKE3 的十六进制表示，保持快照的可读编码一致。
impl Display for ReflectSchemaFingerprint {
    /// 将摘要写为十六进制文本；格式化失败沿用目标 formatter 的错误。
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(blake3::Hash::from_bytes(self.0).to_hex().as_str())
    }
}

impl Serialize for ReflectSchemaFingerprint {
    /// 以展示文本序列化摘要，不计算目录内容或重新接纳快照。
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ReflectSchemaFingerprint {
    /// 从十六进制字符串解析摘要，格式错误转为 serde 错误。
    ///
    /// 泛型 `D` 提供解码器；成功只得到期望摘要，目录内容是否相符仍由 `try_from_snapshot` 重算。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        blake3::Hash::from_hex(&encoded)
            .map(|hash| Self(*hash.as_bytes()))
            .map_err(serde::de::Error::custom)
    }
}

// 先按完整类型路径排序，再编码规范化的注册元数据和依赖，避免输入顺序改变指纹。
/// 为已接纳的完整目录累计版本化编码，返回自有摘要。
///
/// # 参数与约束
/// `entries` 以完整路径为键且已规范化；别名/依赖顺序稳定，字段次序保留。
///
/// # 返回值与参考
/// 返回自有指纹；[`super::ReflectSchemaCatalog::fingerprint`] 缓存它，目录变更后失效。
/// `WorldReflection::list_reflect_types` 的过滤响应仍使用完整目录的指纹。
pub(super) fn fingerprint(
    entries: &BTreeMap<String, ReflectSchemaCatalogEntry>,
) -> ReflectSchemaFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FINGERPRINT_DOMAIN);
    hash_u32(&mut hasher, REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION);
    hash_len(&mut hasher, entries.len());
    for entry in entries.values() {
        hash_registration(&mut hasher, &entry.registration);
        hash_len(&mut hasher, entry.dependencies.len());
        for dependency in &entry.dependencies {
            hash_str(&mut hasher, dependency);
        }
    }
    ReflectSchemaFingerprint(*hasher.finalize().as_bytes())
}

/// 按固定顺序编码类型身份、显示/文档、字段和序列化/可见性元数据。
///
/// `hasher` 累计本次目录编码，`registration` 只在调用期间借用。
/// 字段顺序参与摘要以匹配当前槽位；此处不安装适配器或验证值语义。
fn hash_registration(hasher: &mut blake3::Hasher, registration: &ReflectTypeRegistration) {
    let type_path = &registration.type_path;
    hash_str(hasher, type_path.type_path());
    hash_str(hasher, type_path.short_type_path());
    hash_optional_str(hasher, type_path.module_path());
    hash_optional_str(hasher, type_path.plugin_id());
    hash_str(hasher, &registration.display_name);
    hash_optional_str(hasher, registration.documentation.as_deref());
    hash_u8(hasher, type_kind_tag(&registration.type_info.kind));
    hash_len(hasher, registration.type_info.fields.len());
    for field in &registration.type_info.fields {
        hash_field(hasher, field);
    }
    hash_u8(hasher, serialization_tag(&registration.serialization));
    hash_u8(hasher, role_tag(registration.role));
    hash_bool(hasher, registration.serializable);
    hash_bool(hasher, registration.editor_visible);
    hash_bool(hasher, registration.remote_visible);
    hash_u8(
        hasher,
        script_visibility_tag(registration.script_visibility),
    );
}

/// 编码一个字段的 UUID 身份、名称/别名、默认值、范围、选项和编辑描述。
///
/// 别名已排序，字段及选项的原有次序仍有意义；长度与可选标记区分各段边界。
/// 参考 [`hash_registration`]；改变参与字段或顺序时须提升算法版本。
fn hash_field(hasher: &mut blake3::Hasher, field: &ReflectFieldInfo) {
    hasher.update(field.id.as_uuid().as_bytes());
    hash_str(hasher, &field.name);
    hash_str(hasher, &field.display_name);
    hash_len(hasher, field.aliases.len());
    for alias in &field.aliases {
        hash_str(hasher, alias);
    }
    hash_str(hasher, &field.value_type_path);
    hash_bool(hasher, field.editable);
    hash_bool(hasher, field.serializable);
    hash_bool(hasher, field.editor_visible);
    hash_optional_reflected_value(hasher, field.default_value.as_ref());
    hash_optional_numeric_range(hasher, field.numeric_range.as_ref());
    hash_len(hasher, field.enum_options.len());
    for option in &field.enum_options {
        hash_str(hasher, &option.value);
        hash_str(hasher, &option.display_name);
        hash_optional_str(hasher, option.documentation.as_deref());
    }
    hash_u8(hasher, editor_hint_tag(&field.editor_hint));
    hash_optional_str(hasher, field.documentation.as_deref());
}

/// 先编码可选值是否存在，再编码节点，使缺省与显式 Null 有不同输入。
fn hash_optional_reflected_value(hasher: &mut blake3::Hasher, value: Option<&ReflectedValue>) {
    match value {
        Some(value) => {
            hash_u8(hasher, 1);
            hash_reflected_value(hasher, value);
        }
        None => hash_u8(hasher, 0),
    }
}

/// 显式工作栈借用待编码节点，生命周期 `'a` 限定在一次默认值遍历中。
enum ValueWork<'a> {
    /// 反射值节点，先写反射类别标签。
    Reflected(&'a ReflectedValue),
    /// 嵌入 JSON 节点，使用独立的 JSON 类别标签。
    Json(&'a serde_json::Value),
    /// 映射键文本，在对应值前消费。
    Text(&'a str),
}

/// 以显式工作栈编码反射默认值与 JSON 子树，避免哈希遍历随值深度递归。
///
/// # 参数与生命周期
/// `hasher` 累计当前字段编码；`root` 及栈内节点都为借用，不改写元数据。
///
/// # 约束
/// 此处编码而不校验值预算或相容性，注册端 `validate_registration` 另行检查。
/// 反射浮点数保留原始位模式，列表保留顺序，映射按键序，各类别有独立标签。
///
/// # 参考
/// [`hash_optional_reflected_value`]、[`hash_field`]。
fn hash_reflected_value(hasher: &mut blake3::Hasher, root: &ReflectedValue) {
    // 后进先出；逆序压入子节点，使实际编码次序与正向遍历一致。
    let mut work = vec![ValueWork::Reflected(root)];
    while let Some(item) = work.pop() {
        match item {
            ValueWork::Text(value) => hash_str(hasher, value),
            ValueWork::Reflected(value) => match value {
                ReflectedValue::Null => hash_u8(hasher, 0),
                ReflectedValue::Bool(value) => {
                    hash_u8(hasher, 1);
                    hash_bool(hasher, *value);
                }
                ReflectedValue::Integer(value) => {
                    hash_u8(hasher, 2);
                    hasher.update(&value.to_le_bytes());
                }
                ReflectedValue::Unsigned(value) => {
                    hash_u8(hasher, 3);
                    hasher.update(&value.to_le_bytes());
                }
                ReflectedValue::Scalar(value) => {
                    hash_u8(hasher, 4);
                    // 保留浮点位模式，包含正负零；不按数值等价关系归一化。
                    hasher.update(&value.to_bits().to_le_bytes());
                }
                ReflectedValue::String(value) => {
                    hash_u8(hasher, 5);
                    hash_str(hasher, value);
                }
                ReflectedValue::Enum(value) => {
                    hash_u8(hasher, 6);
                    hash_str(hasher, value);
                }
                ReflectedValue::Vec2(values) => {
                    hash_u8(hasher, 7);
                    hash_floats(hasher, values);
                }
                ReflectedValue::Vec3(values) => {
                    hash_u8(hasher, 8);
                    hash_floats(hasher, values);
                }
                ReflectedValue::Vec4(values) => {
                    hash_u8(hasher, 9);
                    hash_floats(hasher, values);
                }
                ReflectedValue::Quaternion(values) => {
                    hash_u8(hasher, 10);
                    hash_floats(hasher, values);
                }
                ReflectedValue::Entity(value) => {
                    hash_u8(hasher, 11);
                    match value {
                        Some(value) => {
                            hash_u8(hasher, 1);
                            hasher.update(&value.to_le_bytes());
                        }
                        None => hash_u8(hasher, 0),
                    }
                }
                ReflectedValue::Resource(value) => {
                    hash_u8(hasher, 12);
                    hash_str(hasher, value);
                }
                ReflectedValue::List(values) => {
                    hash_u8(hasher, 13);
                    hash_len(hasher, values.len());
                    work.extend(values.iter().rev().map(ValueWork::Reflected));
                }
                ReflectedValue::Map(values) => {
                    hash_u8(hasher, 14);
                    hash_len(hasher, values.len());
                    // BTreeMap 已按键排序；先压值再压键，使弹栈时先写键后写值。
                    for (key, value) in values.iter().rev() {
                        work.push(ValueWork::Reflected(value));
                        work.push(ValueWork::Text(key));
                    }
                }
                ReflectedValue::Json(value) => {
                    // 外层反射 JSON 标签先写入，避免与直接反射节点混淆。
                    hash_u8(hasher, 15);
                    work.push(ValueWork::Json(value));
                }
            },
            ValueWork::Json(value) => match value {
                serde_json::Value::Null => hash_u8(hasher, 0),
                serde_json::Value::Bool(value) => {
                    hash_u8(hasher, 1);
                    hash_bool(hasher, *value);
                }
                serde_json::Value::Number(value) => {
                    hash_u8(hasher, 2);
                    hash_str(hasher, &value.to_string());
                }
                serde_json::Value::String(value) => {
                    hash_u8(hasher, 3);
                    hash_str(hasher, value);
                }
                serde_json::Value::Array(values) => {
                    hash_u8(hasher, 4);
                    hash_len(hasher, values.len());
                    work.extend(values.iter().rev().map(ValueWork::Json));
                }
                serde_json::Value::Object(values) => {
                    hash_u8(hasher, 5);
                    hash_len(hasher, values.len());
                    // 显式排序 JSON 键，不依赖 serde_json 当前采用的 map 存储顺序。
                    let mut entries = values.iter().collect::<Vec<_>>();
                    entries.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
                    for (key, value) in entries.into_iter().rev() {
                        work.push(ValueWork::Json(value));
                        work.push(ValueWork::Text(key));
                    }
                }
            },
        }
    }
}

/// 编码范围存在性及 min/max/step/precision，各可选项独立带存在标记。
fn hash_optional_numeric_range(hasher: &mut blake3::Hasher, range: Option<&ReflectNumericRange>) {
    let Some(range) = range else {
        hash_u8(hasher, 0);
        return;
    };
    hash_u8(hasher, 1);
    hash_optional_f32(hasher, range.min());
    hash_optional_f32(hasher, range.max());
    hash_optional_f32(hasher, range.step());
    match range.precision() {
        Some(value) => {
            hash_u8(hasher, 1);
            hash_u8(hasher, value);
        }
        None => hash_u8(hasher, 0),
    }
}

/// 编码可选浮点数的存在性及原始位模式，不用显示文本代替位表示。
fn hash_optional_f32(hasher: &mut blake3::Hasher, value: Option<f32>) {
    match value {
        Some(value) => {
            hash_u8(hasher, 1);
            hasher.update(&value.to_bits().to_le_bytes());
        }
        None => hash_u8(hasher, 0),
    }
}

/// 按分量顺序写固定 `N` 长度浮点数组；调用方节点标签已区分数组维度。
fn hash_floats<const N: usize>(hasher: &mut blake3::Hasher, values: &[f32; N]) {
    for value in values {
        hasher.update(&value.to_bits().to_le_bytes());
    }
}

/// 编码可选字符串存在性，区分空字符串与缺省值。
fn hash_optional_str(hasher: &mut blake3::Hasher, value: Option<&str>) {
    match value {
        Some(value) => {
            hash_u8(hasher, 1);
            hash_str(hasher, value);
        }
        None => hash_u8(hasher, 0),
    }
}

/// 先写 UTF-8 字节长度再写原始文本，明确相邻字符串的编码边界。
fn hash_str(hasher: &mut blake3::Hasher, value: &str) {
    hash_len(hasher, value.len());
    hasher.update(value.as_bytes());
}

/// 将计数统一写为 64 位小端长度，目录、列表和文本段共用此格式。
fn hash_len(hasher: &mut blake3::Hasher, value: usize) {
    hasher.update(&(value as u64).to_le_bytes());
}

/// 将布尔值编码为单字节 0/1，不依赖 Rust 内存布局。
fn hash_bool(hasher: &mut blake3::Hasher, value: bool) {
    hash_u8(hasher, u8::from(value));
}

/// 以固定小端字节序追加 32 位数值，供算法版本等固定宽度项使用。
fn hash_u32(hasher: &mut blake3::Hasher, value: u32) {
    hasher.update(&value.to_le_bytes());
}

/// 追加单字节标签，标签值由各枚举映射函数显式维护。
fn hash_u8(hasher: &mut blake3::Hasher, value: u8) {
    hasher.update(&[value]);
}

/// 将全部类型形状映射为固定标签，不依赖 Rust 判别值或 serde 文本。
fn type_kind_tag(kind: &ReflectTypeKind) -> u8 {
    match kind {
        ReflectTypeKind::Struct => 0,
        ReflectTypeKind::TupleStruct => 1,
        ReflectTypeKind::Tuple => 2,
        ReflectTypeKind::Enum => 3,
        ReflectTypeKind::List => 4,
        ReflectTypeKind::Map => 5,
        ReflectTypeKind::Scalar => 6,
        ReflectTypeKind::Opaque => 7,
        ReflectTypeKind::Json => 8,
    }
}

/// 将全部序列化策略映射为固定标签；调整编码时须与算法版本协调。
fn serialization_tag(strategy: &ReflectSerializationStrategy) -> u8 {
    match strategy {
        ReflectSerializationStrategy::None => 0,
        ReflectSerializationStrategy::Value => 1,
        ReflectSerializationStrategy::Json => 2,
        ReflectSerializationStrategy::ResourceHandle => 3,
        ReflectSerializationStrategy::EntityReference => 4,
    }
}

/// 编码 Value/Component/Resource 角色，它们属于目录内容。
fn role_tag(role: ReflectTypeRole) -> u8 {
    match role {
        ReflectTypeRole::Value => 0,
        ReflectTypeRole::Component => 1,
        ReflectTypeRole::Resource => 2,
    }
}

/// 编码私有/公开脚本可见性，保留该元数据对摘要的影响。
fn script_visibility_tag(visibility: ReflectScriptVisibility) -> u8 {
    match visibility {
        ReflectScriptVisibility::Private => 0,
        ReflectScriptVisibility::Public => 1,
    }
}

/// 编码全部编辑提示变体，枚举定义顺序改变不会自动改变这里的标签。
fn editor_hint_tag(hint: &ReflectEditorHint) -> u8 {
    match hint {
        ReflectEditorHint::None => 0,
        ReflectEditorHint::String => 1,
        ReflectEditorHint::MultilineString => 2,
        ReflectEditorHint::Bool => 3,
        ReflectEditorHint::Integer => 4,
        ReflectEditorHint::Unsigned => 5,
        ReflectEditorHint::Scalar => 6,
        ReflectEditorHint::Vec2 => 7,
        ReflectEditorHint::Vec3 => 8,
        ReflectEditorHint::Vec4 => 9,
        ReflectEditorHint::Enum => 10,
        ReflectEditorHint::Entity => 11,
        ReflectEditorHint::Resource => 12,
        ReflectEditorHint::Color => 13,
        ReflectEditorHint::Json => 14,
    }
}
