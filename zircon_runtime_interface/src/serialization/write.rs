use std::fmt;
use std::io::{self, Write};

use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};

use super::binary::encode_binary_payload;
use super::text::{
    canonical_writer::{
        write_canonical_text, write_canonical_text_with_budget, SERDE_JSON_RAW_VALUE_TOKEN,
    },
    document::TextDocument,
    envelope::TextEnvelope,
};
use super::{
    CanonicalTextWriteError, Format, PayloadHeader, SerializationBudget, VersionedSchema,
    WriteError,
};

/// Encodes a payload with the current schema header and canonical text rules.
/// # 参数
/// - `value`：实现 `VersionedSchema + Serialize` 的当前版本载荷。
/// - `format`：选择版本化文本或二进制线格式。
/// # 返回值
/// 成功时返回完整封套字节；失败时保留 schema/version 上下文。文本分支通过规范写入器写入 `Vec`，二进制分支先验证再构造载荷值。
pub fn write_versioned<T>(value: &T, format: Format) -> Result<Vec<u8>, WriteError>
where
    T: VersionedSchema + Serialize,
{
    match format {
        Format::Text => {
            let mut encoded = Vec::new();
            write_versioned_text_to(value, &mut encoded)?;
            Ok(encoded)
        }
        Format::Binary => {
            let payload = encode_binary_payload_value(value)?;
            encode_binary_payload(current_header::<T>(), payload)
        }
    }
}

/// Encodes canonical, pretty JSON with a single trailing newline.
/// 参数 `value` 是当前版本载荷；成功时返回含版本封套、规范 JSON 和单个末尾换行的 UTF-8 字符串。
/// 载荷编码或 UTF-8 转换失败时返回带 schema/version 上下文的 `WriteError`。
pub fn write_versioned_text<T>(value: &T) -> Result<String, WriteError>
where
    T: VersionedSchema + Serialize,
{
    let mut encoded = Vec::new();
    write_versioned_text_to(value, &mut encoded)?;
    String::from_utf8(encoded).map_err(|error| WriteError::TextEncode {
        schema_id: T::SCHEMA.as_str().to_string(),
        schema_version: T::VERSION,
        source: serde_json::Error::io(io::Error::new(io::ErrorKind::InvalidData, error)),
    })
}

/// Streams canonical, pretty JSON with a single trailing newline to a sink.
///
/// The caller owns any buffering policy. This function never constructs a
/// complete text document solely to write it to the supplied sink.
/// 写入或刷新失败时 sink 可能已有文档前缀；持久化调用方负责临时文件与原子替换。
/// # 参数
/// - `value`：同步借用并序列化的当前版本载荷。
/// - `sink`：由调用方持有的字节写入器，其缓冲和持久化策略由调用方决定。
/// # 返回值
/// 成功时返回写入的 UTF-8 字节数；失败时返回带 schema/version 上下文的 `WriteError`，此前已写入的前缀不会回滚。
pub fn write_versioned_text_to<T, W>(value: &T, sink: &mut W) -> Result<usize, WriteError>
where
    T: VersionedSchema + Serialize,
    W: Write + ?Sized,
{
    let document = TextDocument {
        envelope: TextEnvelope {
            header: current_header::<T>(),
            payload: value,
        },
    };
    write_canonical_text(&document, sink).map_err(|error| match error {
        CanonicalTextWriteError::NonFinite { value } => WriteError::NonFiniteFloat {
            schema_id: T::SCHEMA.as_str().to_string(),
            schema_version: T::VERSION,
            value,
        },
        CanonicalTextWriteError::PayloadValidation { reason } => WriteError::PayloadValidation {
            schema_id: T::SCHEMA.as_str().to_string(),
            schema_version: T::VERSION,
            reason,
        },
        CanonicalTextWriteError::PayloadEncode { reason } => WriteError::PayloadEncode {
            schema_id: T::SCHEMA.as_str().to_string(),
            schema_version: T::VERSION,
            source: <serde_json::Error as serde::ser::Error>::custom(reason),
        },
        CanonicalTextWriteError::OutputTooLarge { max, found } => {
            WriteError::TextDocumentTooLarge {
                schema_id: T::SCHEMA.as_str().to_string(),
                schema_version: T::VERSION,
                max,
                found,
            }
        }
        CanonicalTextWriteError::ResourceLimitExceeded {
            resource,
            max,
            found,
        } => WriteError::TextResourceLimitExceeded {
            schema_id: T::SCHEMA.as_str().to_string(),
            schema_version: T::VERSION,
            resource,
            max,
            found,
        },
        CanonicalTextWriteError::Io { operation, source } => WriteError::TextWrite {
            schema_id: T::SCHEMA.as_str().to_string(),
            schema_version: T::VERSION,
            operation,
            source,
        },
    })
}

/// Streams canonical JSON without applying a versioned-schema envelope.
///
/// This is reserved for runtime-owned archive formats that already define
/// their own compatibility contract.
/// 此入口使用调用方预算，不套用版本化文本的固定线格式上限。
/// # 参数
/// - `value`：同步序列化的借用值，不添加版本封套。
/// - `sink`：接收规范 JSON 字节的调用方写入器。
/// - `budget`：含末尾换行的输出字节上限；深度、条目数等资源另有内部限额。
/// # 返回值
/// 成功时返回本次写出的字节数（含末尾换行）；预算、载荷或 I/O 失败返回对应的 `CanonicalTextWriteError`。
pub fn write_canonical_text_to<T, W>(
    value: &T,
    sink: &mut W,
    budget: SerializationBudget,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    write_canonical_text_with_budget(value, sink, budget)
}

/// 先以只校验的 serde 遍历检查浮点和受支持的载荷形状，再将载荷物化为供二进制编码器使用的 `serde_json::Value`。
fn encode_binary_payload_value<T>(value: &T) -> Result<serde_json::Value, WriteError>
where
    T: VersionedSchema + Serialize,
{
    if let Err(error) = value.serialize(FiniteFloatGuard) {
        return Err(match error {
            FloatValidationError::NonFinite { value } => WriteError::NonFiniteFloat {
                schema_id: T::SCHEMA.as_str().to_string(),
                schema_version: T::VERSION,
                value,
            },
            FloatValidationError::Custom { reason } => WriteError::PayloadValidation {
                schema_id: T::SCHEMA.as_str().to_string(),
                schema_version: T::VERSION,
                reason,
            },
        });
    }
    serde_json::to_value(value).map_err(|source| WriteError::PayloadEncode {
        schema_id: T::SCHEMA.as_str().to_string(),
        schema_version: T::VERSION,
        source,
    })
}

#[derive(Debug)]
/// 二进制载荷预检期间由 serde 回调返回的内部错误；不承载编码后的数据。
enum FloatValidationError {
    /// 发现不能进入载荷的非有限浮点值。
    NonFinite { value: f64 },
    /// serde 自定义验证失败，包括拒绝不在支持域内的 RawValue。
    Custom { reason: String },
}

impl fmt::Display for FloatValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite { value } => write!(formatter, "non-finite float {value}"),
            Self::Custom { reason } => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for FloatValidationError {}

impl serde::ser::Error for FloatValidationError {
    fn custom<T>(message: T) -> Self
    where
        T: fmt::Display,
    {
        Self::Custom {
            reason: message.to_string(),
        }
    }
}

#[derive(Clone, Copy)]
/// 只遍历载荷并验证，不保留序列化结果的校验用 `Serializer`。
struct FiniteFloatGuard;

/// 复合值访问器；每个元素、键和值都重新交给 `FiniteFloatGuard` 检查。
struct FiniteFloatCompound;

// 整数序列化入口不产生数据，只接受各整数类型；本轮预检只额外限制浮点。
macro_rules! finite_integer_methods {
    ($($method:ident($type:ty)),* $(,)?) => {
        $(fn $method(self, _value: $type) -> Result<Self::Ok, Self::Error> { Ok(()) })*
    };
}

// 以 serde 的实际序列化回调递归遍历载荷；在这里集中拒绝 RawValue 并检查有限浮点。
impl Serializer for FiniteFloatGuard {
    type Ok = ();
    type Error = FloatValidationError;
    type SerializeSeq = FiniteFloatCompound;
    type SerializeTuple = FiniteFloatCompound;
    type SerializeTupleStruct = FiniteFloatCompound;
    type SerializeTupleVariant = FiniteFloatCompound;
    type SerializeMap = FiniteFloatCompound;
    type SerializeStruct = FiniteFloatCompound;
    type SerializeStructVariant = FiniteFloatCompound;

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    finite_integer_methods!(
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
    );

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        validate_float(value as f64)
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        validate_float(value)
    }

    fn serialize_char(self, _value: char) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_str(self, _value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    // RawValue 会绕过规范载荷域的结构化验证，因此在两种 serde 表达入口都拒绝。
    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if name == SERDE_JSON_RAW_VALUE_TOKEN {
            Err(FloatValidationError::Custom {
                reason: "serde_json RawValue is outside the canonical versioned payload domain"
                    .to_string(),
            })
        } else {
            value.serialize(self)
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(FiniteFloatCompound)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(FiniteFloatCompound)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(FiniteFloatCompound)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(FiniteFloatCompound)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(FiniteFloatCompound)
    }

    // RawValue 也可能通过结构体回调到达，必须与 newtype 回调保持同一准入规则。
    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if name == SERDE_JSON_RAW_VALUE_TOKEN {
            Err(FloatValidationError::Custom {
                reason: "serde_json RawValue is outside the canonical versioned payload domain"
                    .to_string(),
            })
        } else {
            Ok(FiniteFloatCompound)
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(FiniteFloatCompound)
    }

    fn collect_str<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        Ok(())
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

// 多种 serde 复合值共享同一递归遍历：只检查每个字段，不累计复合内容。
macro_rules! finite_compound_impl {
    ($trait:ident, $method:ident) => {
        impl $trait for FiniteFloatCompound {
            type Ok = ();
            type Error = FloatValidationError;

            fn $method<T>(&mut self, value: &T) -> Result<(), Self::Error>
            where
                T: ?Sized + Serialize,
            {
                value.serialize(FiniteFloatGuard)
            }

            fn end(self) -> Result<Self::Ok, Self::Error> {
                Ok(())
            }
        }
    };
}

finite_compound_impl!(SerializeSeq, serialize_element);
finite_compound_impl!(SerializeTuple, serialize_element);
finite_compound_impl!(SerializeTupleStruct, serialize_field);
finite_compound_impl!(SerializeTupleVariant, serialize_field);

// Map 的键和值都可能包含嵌套浮点，分别完整遍历后才结束该复合值。
impl SerializeMap for FiniteFloatCompound {
    type Ok = ();
    type Error = FloatValidationError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(FiniteFloatGuard)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(FiniteFloatGuard)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// 结构体字段按 serde 回调递归检查；字段名不参与数值校验。
impl SerializeStruct for FiniteFloatCompound {
    type Ok = ();
    type Error = FloatValidationError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(FiniteFloatGuard)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// 结构体变体的字段值沿用相同校验器，不在预检阶段保存变体数据。
impl SerializeStructVariant for FiniteFloatCompound {
    type Ok = ();
    type Error = FloatValidationError;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(FiniteFloatGuard)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// 仅接受有限浮点；非有限值作为内部校验错误返回给编码入口映射。
fn validate_float(value: f64) -> Result<(), FloatValidationError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(FloatValidationError::NonFinite { value })
    }
}

/// 从载荷类型的静态 schema 标识和版本构造当前写入封套头。
fn current_header<T>() -> PayloadHeader
where
    T: VersionedSchema,
{
    PayloadHeader {
        schema_id: T::SCHEMA.clone(),
        schema_version: T::VERSION,
    }
}
