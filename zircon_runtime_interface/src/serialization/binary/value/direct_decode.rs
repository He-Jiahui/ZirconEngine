//! 对已解析的 binary 节点流先做完整结构/值域检查，再由 serde 直接构造目标类型。
//! 本模块区分位置解码、节点流无效和目标类型不匹配；schema 身份/版本由上层 loader 负责。

use std::collections::BTreeSet;

use serde::de::{
    self, DeserializeOwned, DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;

use super::super::wire::{
    MAX_BINARY_CONTAINER_ENTRIES, MAX_BINARY_DEPTH, MAX_BINARY_NODES, MAX_BINARY_STRING_BYTES,
};
use super::{BinaryNode, BinaryValue, BinaryValueError};

/// 当前 binary typed decode 边界的错误类别；上层 loader 将其映射为公开读取错误。
/// 此类型不负责校验封套中的 schema identity 或 schema version。
#[derive(Debug)]
pub(in crate::serialization) enum DirectBinaryDecodeError {
    /// positional body 无法解码；保留底层格式或输入完整性错误。
    Malformed(crate::serialization::PositionalDecodeError),
    /// 节点流可解码但违反 binary value 域约束，或存在多个根值。
    Invalid(BinaryValueError),
    /// 已验证节点流与目标 serde 类型不匹配；保存目标类型解码原因。
    Payload(String),
}

/// 校验完整节点流后，让 serde 直接从同一节点序列构造目标类型。
///
/// # 参数
/// - value：已由 positional decoder 解析的拥有型节点流；此处仍检查节点结构和值域限制。
///
/// # 返回值
/// 返回拥有型 T，且必须恰好消费一个根节点及其全部子树。
///
/// # 错误
/// 节点结构/值域错误为 DirectBinaryDecodeError::Invalid；目标类型不匹配为 Payload。
/// positional body 的格式错误由调用方在进入此处前包装。
pub(in crate::serialization) fn decode_binary_value_direct<T>(
    value: BinaryValue,
) -> Result<T, DirectBinaryDecodeError>
where
    T: DeserializeOwned,
{
    // 先验证整条节点流的边界、键唯一性和深度，再交给目标类型反序列化。
    validate_stream(&value.nodes).map_err(DirectBinaryDecodeError::Invalid)?;

    let mut decoder = BinaryValueDeserializer {
        nodes: &value.nodes,
        index: 0,
    };
    let decoded = T::deserialize(&mut decoder)
        .map_err(|error| DirectBinaryDecodeError::Payload(error.to_string()))?;
    if decoder.index != decoder.nodes.len() {
        return Err(DirectBinaryDecodeError::Invalid(
            BinaryValueError::MultipleRootValues,
        ));
    }
    Ok(decoded)
}

/// 检查扁平前序节点流恰好表示一个根值，并先应用节点数上限。
/// 游标结束时仍有节点表示多个根值；递归提前取不到节点表示容器不完整。
fn validate_stream(nodes: &[BinaryNode]) -> Result<(), BinaryValueError> {
    if nodes.is_empty() {
        return Err(BinaryValueError::EmptyValue);
    }
    if nodes.len() > MAX_BINARY_NODES {
        return Err(BinaryValueError::NodeLimitExceeded {
            max: MAX_BINARY_NODES,
            found: nodes.len(),
        });
    }

    let mut index = 0;
    validate_value(nodes, &mut index, 0)?;
    if index != nodes.len() {
        return Err(BinaryValueError::MultipleRootValues);
    }
    Ok(())
}

/// 从 index 开始深度优先消费一个值及其子树，并推进该游标。
/// root 调用的 depth 为零；每进入数组或对象子项，容器深度增加一层。
fn validate_value(
    nodes: &[BinaryNode],
    index: &mut usize,
    depth: usize,
) -> Result<(), BinaryValueError> {
    let Some(node) = nodes.get(*index) else {
        return Err(BinaryValueError::IncompleteContainer);
    };
    *index += 1;

    match node {
        BinaryNode::Null | BinaryNode::Bool(_) | BinaryNode::I64(_) | BinaryNode::U64(_) => {}
        BinaryNode::F64(value) if value.is_finite() => {}
        BinaryNode::F64(value) => {
            return Err(BinaryValueError::NonFiniteFloat { value: *value });
        }
        BinaryNode::String(value) => validate_string(value)?,
        BinaryNode::Array { len } => {
            let len = validate_container(*len)?;
            validate_depth(depth + 1)?;
            for _ in 0..len {
                validate_value(nodes, index, depth + 1)?;
            }
        }
        BinaryNode::Object { len } => {
            let len = validate_container(*len)?;
            validate_depth(depth + 1)?;
            let mut keys = BTreeSet::new();
            for _ in 0..len {
                let Some(BinaryNode::ObjectKey(key)) = nodes.get(*index) else {
                    return Err(BinaryValueError::MissingObjectKey);
                };
                *index += 1;
                validate_string(key)?;
                // 在目标 map visitor 接收键之前拒绝重复项，避免由 map 容器覆盖或折叠重复键。
                if !keys.insert(key.as_str()) {
                    return Err(BinaryValueError::DuplicateObjectKey { key: key.clone() });
                }
                validate_value(nodes, index, depth + 1)?;
            }
        }
        BinaryNode::ObjectKey(key) => {
            return Err(BinaryValueError::UnexpectedObjectKey { key: key.clone() });
        }
    }
    Ok(())
}

/// 将容器长度转为宿主 usize，并限制单个数组元素数或对象键值对数。
fn validate_container(found: u32) -> Result<usize, BinaryValueError> {
    let found = found as usize;
    if found <= MAX_BINARY_CONTAINER_ENTRIES {
        return Ok(found);
    }
    Err(BinaryValueError::ContainerLimitExceeded {
        max: MAX_BINARY_CONTAINER_ENTRIES,
        found,
    })
}

/// 限制根容器及其嵌套数组/对象的最大深度。
fn validate_depth(found: usize) -> Result<(), BinaryValueError> {
    if found <= MAX_BINARY_DEPTH {
        return Ok(());
    }
    Err(BinaryValueError::DepthLimitExceeded {
        max: MAX_BINARY_DEPTH,
        found,
    })
}

/// 按 UTF-8 字节长度限制字符串值和对象键。
fn validate_string(value: &str) -> Result<(), BinaryValueError> {
    if value.len() <= MAX_BINARY_STRING_BYTES {
        return Ok(());
    }
    Err(BinaryValueError::StringLimitExceeded {
        max: MAX_BINARY_STRING_BYTES,
        found: value.len(),
    })
}

type DecodeError = de::value::Error;

/// 以游标读取已验证节点流的 serde adapter；游标指向下一个待读节点。
/// 节点切片在调用期间只被借用；外层 DeserializeOwned 约束使返回值不借用该切片。
struct BinaryValueDeserializer<'nodes> {
    // 借用 BinaryValue 的节点缓冲；其生命周期覆盖本次目标类型解码。
    nodes: &'nodes [BinaryNode],
    // 下一项将由 next_node 消费的扁平节点位置。
    index: usize,
}

impl<'nodes> BinaryValueDeserializer<'nodes> {
    /// 取出并推进一个节点；节点流提前结束时返回 serde 目标类型错误。
    fn next_node(&mut self) -> Result<&BinaryNode, DecodeError> {
        let node = self.nodes.get(self.index).ok_or_else(|| {
            de::Error::custom("binary value ended before a typed payload completed")
        })?;
        self.index += 1;
        Ok(node)
    }

    /// 借看下一个节点但不移动游标，供 Option 等前瞻分支使用。
    fn peek_node(&self) -> Result<&BinaryNode, DecodeError> {
        self.nodes
            .get(self.index)
            .ok_or_else(|| de::Error::custom("binary value ended before a typed payload completed"))
    }

    // Serde 忽略未知字段时会调用此路径；按节点数跳过完整子树，保持后续兄弟值的游标对齐。
    fn skip_value(&mut self) -> Result<(), DecodeError> {
        match self.next_node()? {
            BinaryNode::Array { len } => {
                for _ in 0..*len {
                    self.skip_value()?;
                }
            }
            BinaryNode::Object { len } => {
                for _ in 0..*len {
                    if !matches!(self.next_node()?, BinaryNode::ObjectKey(_)) {
                        return Err(de::Error::custom("binary object entry is missing its key"));
                    }
                    self.skip_value()?;
                }
            }
            BinaryNode::ObjectKey(_) => {
                return Err(de::Error::custom(
                    "binary object key appeared where a value was required",
                ));
            }
            _ => {}
        }
        Ok(())
    }
}

// 已预校验的节点种类决定标量、容器和枚举表示；访问器共享同一游标。
// 未显式实现的标量入口由 deserialize_any 转发，具体类型仍由 serde Visitor 判定。
impl<'de, 'nodes> Deserializer<'de> for &mut BinaryValueDeserializer<'nodes> {
    type Error = DecodeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.next_node()? {
            BinaryNode::Null => visitor.visit_unit(),
            BinaryNode::Bool(value) => visitor.visit_bool(*value),
            BinaryNode::I64(value) => visitor.visit_i64(*value),
            BinaryNode::U64(value) => visitor.visit_u64(*value),
            BinaryNode::F64(value) => visitor.visit_f64(*value),
            BinaryNode::String(value) => visitor.visit_string(value.clone()),
            BinaryNode::Array { len } => {
                let remaining = *len as usize;
                visitor.visit_seq(BinarySequenceAccess {
                    decoder: self,
                    remaining,
                })
            }
            BinaryNode::Object { len } => {
                let remaining = *len as usize;
                visitor.visit_map(BinaryMapAccess {
                    decoder: self,
                    remaining,
                    awaiting_value: false,
                })
            }
            BinaryNode::ObjectKey(_) => Err(de::Error::custom(
                "binary object key appeared where a value was required",
            )),
        }
    }

    // null 消费为 None；其他节点留给 Some 分支递归反序列化。
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if matches!(self.peek_node()?, BinaryNode::Null) {
            self.index += 1;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.next_node()? {
            BinaryNode::Bool(value) => visitor.visit_bool(*value),
            _ => Err(de::Error::custom("expected a binary bool")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.next_node()? {
            BinaryNode::Null => visitor.visit_unit(),
            _ => Err(de::Error::custom("expected binary null for a unit value")),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.next_node()? {
            BinaryNode::Array { len } => {
                let remaining = *len as usize;
                visitor.visit_seq(BinarySequenceAccess {
                    decoder: self,
                    remaining,
                })
            }
            _ => Err(de::Error::custom("expected a binary array")),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.next_node()? {
            BinaryNode::Object { len } => {
                let remaining = *len as usize;
                visitor.visit_map(BinaryMapAccess {
                    decoder: self,
                    remaining,
                    awaiting_value: false,
                })
            }
            _ => Err(de::Error::custom("expected a binary object")),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    // 无数据 unit variant 可用字符串；对象形式恰含一个 variant key，payload 再由访问器解码。
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.next_node()? {
            BinaryNode::String(variant) => visitor.visit_enum(UnitEnumAccess {
                variant: variant.clone(),
            }),
            BinaryNode::Object { len: 1 } => {
                let variant = match self.next_node()? {
                    BinaryNode::ObjectKey(variant) => variant.clone(),
                    _ => {
                        return Err(de::Error::custom(
                            "binary enum object is missing its variant key",
                        ));
                    }
                };
                visitor.visit_enum(ValueEnumAccess {
                    decoder: self,
                    variant,
                })
            }
            BinaryNode::Object { .. } => Err(de::Error::custom(
                "binary enum object must contain exactly one variant",
            )),
            _ => Err(de::Error::custom("expected a binary enum string or object")),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }

    // 忽略字段时仍消费其整棵子树，避免后续字段游标错位。
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.skip_value()?;
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
    }
}

/// serde 序列访问状态；decoder 借用节点游标，remaining 表示尚未请求的直接元素数。
struct BinarySequenceAccess<'decoder, 'nodes> {
    // 与父 deserializer 共用游标，递归读取当前元素。
    decoder: &'decoder mut BinaryValueDeserializer<'nodes>,
    // 当前数组中尚未交付给 serde visitor 的直接元素数。
    remaining: usize,
}

// 每次请求最多解码一个直接元素；remaining 为零时按 serde 协议返回 None。
impl<'de, 'decoder, 'nodes> SeqAccess<'de> for BinarySequenceAccess<'decoder, 'nodes> {
    type Error = DecodeError;

    // remaining 为零返回 None；否则先递减直接元素计数，再递归解码一个元素。
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(&mut *self.decoder).map(Some)
    }
}

/// serde map/struct 访问状态；每个对象项必须先交付 key，再解码一个 value。
struct BinaryMapAccess<'decoder, 'nodes> {
    // 与父 deserializer 共用对象节点游标。
    decoder: &'decoder mut BinaryValueDeserializer<'nodes>,
    // 尚未完成 value 解码的对象项数。
    remaining: usize,
    // 已交付 key、尚未交付其 value 时为 true。
    awaiting_value: bool,
}

// awaiting_value 阻止重复请求 key 或在没有 key 时取 value；remaining 按完成项递减。
impl<'de, 'decoder, 'nodes> MapAccess<'de> for BinaryMapAccess<'decoder, 'nodes> {
    type Error = DecodeError;

    // 每项只交付一个对象键；上次 key 尚无 value 时拒绝继续取键。
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }
        if self.awaiting_value {
            return Err(de::Error::custom(
                "binary map key was not followed by a value",
            ));
        }
        let BinaryNode::ObjectKey(key) = self.decoder.next_node()? else {
            return Err(de::Error::custom("binary object entry is missing its key"));
        };
        self.awaiting_value = true;
        seed.deserialize(BinaryObjectKeyDeserializer { key: key.clone() })
            .map(Some)
    }

    // 仅在已有待配对 key 时消费一个 value，并完成该对象项。
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if !self.awaiting_value {
            return Err(de::Error::custom(
                "binary map value was requested before a key",
            ));
        }
        self.awaiting_value = false;
        self.remaining -= 1;
        seed.deserialize(&mut *self.decoder)
    }
}

/// 处理 binary string 形式的无数据 enum variant。
struct UnitEnumAccess {
    // 从字符串节点取出的变体名，交给目标 enum 的 serde visitor。
    variant: String,
}

// 把字符串节点中的变体名交给 DeserializeSeed；后续 unit 访问无需再消费 payload 节点。
impl<'de> EnumAccess<'de> for UnitEnumAccess {
    type Error = DecodeError;
    type Variant = UnitVariantAccess;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(de::value::StringDeserializer::<DecodeError>::new(
            self.variant,
        ))?;
        Ok((value, UnitVariantAccess))
    }
}

/// string 形式 unit variant 的访问标记；其他携带数据的 variant 形状不由它解码。
struct UnitVariantAccess;

// 字符串形式的 unit enum 不携带 payload；newtype/tuple/struct 请求会显式失败。
impl<'de> VariantAccess<'de> for UnitVariantAccess {
    type Error = DecodeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        Err(de::Error::custom(
            "binary unit enum cannot decode a newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::custom(
            "binary unit enum cannot decode a tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::custom(
            "binary unit enum cannot decode a struct variant",
        ))
    }
}

/// 处理单键对象形式的 enum 变体，并把 payload 接回同一节点游标。
struct ValueEnumAccess<'decoder, 'nodes> {
    // 变体 payload 所在的同一节点流游标。
    decoder: &'decoder mut BinaryValueDeserializer<'nodes>,
    // 对象中唯一的 variant key。
    variant: String,
}

// 将对象中的 variant key 解析成目标枚举项，并保留 decoder 供其 payload 访问。
impl<'de, 'decoder, 'nodes> EnumAccess<'de> for ValueEnumAccess<'decoder, 'nodes> {
    type Error = DecodeError;
    type Variant = ValueVariantAccess<'decoder, 'nodes>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(de::value::StringDeserializer::<DecodeError>::new(
            self.variant,
        ))?;
        Ok((
            variant,
            ValueVariantAccess {
                decoder: self.decoder,
            },
        ))
    }
}

/// 将 variant payload 的 newtype、tuple、struct 或 null unit 解码委派给节点 adapter。
struct ValueVariantAccess<'decoder, 'nodes> {
    // 供 variant payload 解码继续消费的共享游标。
    decoder: &'decoder mut BinaryValueDeserializer<'nodes>,
}

// object variant 的 unit payload 必须为 Null；其他形态按 newtype/tuple/struct 委派解码。
impl<'de, 'decoder, 'nodes> VariantAccess<'de> for ValueVariantAccess<'decoder, 'nodes> {
    type Error = DecodeError;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.decoder.next_node()? {
            BinaryNode::Null => Ok(()),
            _ => Err(de::Error::custom(
                "binary enum unit variant must contain null",
            )),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.decoder)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        (&mut *self.decoder).deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        (&mut *self.decoder).deserialize_struct("binary enum", fields, visitor)
    }
}

/// 将对象键字符串按 serde 请求的 map key 类型解析为拥有型标量。
struct BinaryObjectKeyDeserializer {
    // 对象键节点的 owned 文本；转换为目标 map key 时消耗此字符串。
    key: String,
}

/// 为各宽度整数 key 生成统一的字符串解析和 visitor 调用。
macro_rules! parse_object_key_integer {
    ($($method:ident($type:ty, $visitor:ident)),* $(,)?) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                let value = self.key.parse::<$type>().map_err(|_| {
                    de::Error::custom(format!("binary object key {:?} is not a valid {}", self.key, stringify!($type)))
                })?;
                visitor.$visitor(value)
            }
        )*
    };
}

// BinaryValue 的对象键在线格式中是字符串；此处只按目标 key 类型解析该字符串。
// 不匹配的数字/布尔/字符及非有限浮点返回 serde 错误，不推进节点流游标。
impl<'de> Deserializer<'de> for BinaryObjectKeyDeserializer {
    type Error = DecodeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.key)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.key.parse::<bool>().map_err(|_| {
            de::Error::custom(format!("binary object key {:?} is not a bool", self.key))
        })?;
        visitor.visit_bool(value)
    }

    parse_object_key_integer!(
        deserialize_i8(i8, visit_i8),
        deserialize_i16(i16, visit_i16),
        deserialize_i32(i32, visit_i32),
        deserialize_i64(i64, visit_i64),
        deserialize_i128(i128, visit_i128),
        deserialize_u8(u8, visit_u8),
        deserialize_u16(u16, visit_u16),
        deserialize_u32(u32, visit_u32),
        deserialize_u64(u64, visit_u64),
        deserialize_u128(u128, visit_u128),
    );

    // f32/f64 key 均解析字符串，并拒绝 NaN 和无穷值以保持值域约束。
    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.key.parse::<f32>().map_err(|_| {
            de::Error::custom(format!("binary object key {:?} is not an f32", self.key))
        })?;
        if !value.is_finite() {
            return Err(de::Error::custom("binary object key must be finite"));
        }
        visitor.visit_f32(value)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.key.parse::<f64>().map_err(|_| {
            de::Error::custom(format!("binary object key {:?} is not an f64", self.key))
        })?;
        if !value.is_finite() {
            return Err(de::Error::custom("binary object key must be finite"));
        }
        visitor.visit_f64(value)
    }

    // char key 必须恰好包含一个 Unicode scalar value。
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut characters = self.key.chars();
        let Some(value) = characters.next() else {
            return Err(de::Error::custom("binary object key is not a character"));
        };
        if characters.next().is_some() {
            return Err(de::Error::custom("binary object key is not a character"));
        }
        visitor.visit_char(value)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.key)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.key)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.key.into_bytes())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.key.into_bytes())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::custom("binary object key cannot decode as unit"))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(de::value::StringDeserializer::<DecodeError>::new(self.key))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.key)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        seq tuple tuple_struct map struct
    }
}
