//! 将 Serde 映射键规整为规范 JSON 字符串，并在预聚合时核算其带引号和转义后的字节数。
//! 仅接纳 JSON 对象键可表示的标量形式；null、RawValue 与复合形式不属于此键域。

use std::fmt;

use serde::ser::{Impossible, Serializer};
use serde::Serialize;

use super::super::write_error::CanonicalTextWriteError;
use super::canonical_writer::SERDE_JSON_RAW_VALUE_TOKEN;

/// A normalized JSON object key and the bytes its quoted canonical spelling owns.
///
/// `value` 保持不带引号的文本供对象排序与查找；`encoded_bytes` 包含引号和转义扩展，供 writer 提前预留输出预算。
pub(super) struct CanonicalMapKey {
    value: String,
    encoded_bytes: usize,
}

impl CanonicalMapKey {
    /// 复制借用文本，并按带引号、转义后的 JSON 键长度校验 `max_bytes`。
    pub(super) fn from_str(value: &str, max_bytes: usize) -> Result<Self, CanonicalTextWriteError> {
        let encoded_bytes = json_string_encoded_len(value, max_bytes)?;
        Ok(Self {
            value: value.to_string(),
            encoded_bytes,
        })
    }

    /// 接管已格式化的文本，并执行与借用入口相同的 JSON 键长度校验。
    pub(super) fn from_string(
        value: String,
        max_bytes: usize,
    ) -> Result<Self, CanonicalTextWriteError> {
        let encoded_bytes = json_string_encoded_len(&value, max_bytes)?;
        Ok(Self {
            value,
            encoded_bytes,
        })
    }

    pub(super) fn value(&self) -> &str {
        &self.value
    }

    pub(super) fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    pub(super) fn into_parts(self) -> (String, usize) {
        (self.value, self.encoded_bytes)
    }
}

/// Converts serde map keys into the scalar JSON spelling used by canonical output.
///
/// 允许布尔、数字、字符、字符串、单元枚举变体与 `Display` 文本；`Some` 递归使用同一标量规则。
/// 拒绝 null、RawValue、字节串及复合值。
/// `max_bytes` 限制最终带引号且转义后的拼写长度，而不只是原始字符串长度。
pub(super) struct CanonicalMapKeySerializer {
    max_bytes: usize,
}

impl CanonicalMapKeySerializer {
    pub(super) fn new(max_bytes: usize) -> Self {
        Self { max_bytes }
    }
}

// 整数先转成十进制键文本，再统一校验 JSON 引号和转义所占的字节数。
macro_rules! key_integer_methods {
    ($($method:ident($type:ty)),* $(,)?) => {
        $(fn $method(self, value: $type) -> Result<Self::Ok, Self::Error> {
            CanonicalMapKey::from_string(value.to_string(), self.max_bytes)
        })*
    };
}

impl Serializer for CanonicalMapKeySerializer {
    type Ok = CanonicalMapKey;
    type Error = CanonicalTextWriteError;
    type SerializeSeq = Impossible<CanonicalMapKey, CanonicalTextWriteError>;
    type SerializeTuple = Impossible<CanonicalMapKey, CanonicalTextWriteError>;
    type SerializeTupleStruct = Impossible<CanonicalMapKey, CanonicalTextWriteError>;
    type SerializeTupleVariant = Impossible<CanonicalMapKey, CanonicalTextWriteError>;
    type SerializeMap = Impossible<CanonicalMapKey, CanonicalTextWriteError>;
    type SerializeStruct = Impossible<CanonicalMapKey, CanonicalTextWriteError>;
    type SerializeStructVariant = Impossible<CanonicalMapKey, CanonicalTextWriteError>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        CanonicalMapKey::from_string(value.to_string(), self.max_bytes)
    }

    key_integer_methods!(
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128)
    );

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        finite_key(value as f64, self.max_bytes)
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        finite_key(value, self.max_bytes)
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        CanonicalMapKey::from_string(value.to_string(), self.max_bytes)
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        CanonicalMapKey::from_str(value, self.max_bytes)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        invalid_key()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(CanonicalTextWriteError::PayloadEncode {
            reason: "JSON object keys cannot be null".to_string(),
        })
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        CanonicalMapKey::from_str(variant, self.max_bytes)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        reject_raw_value(name)?;
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        invalid_key()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        invalid_key()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        invalid_key()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        invalid_key()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        invalid_key()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        invalid_key()
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        reject_raw_value(name)?;
        invalid_key()
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        invalid_key()
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        let mut writer = CanonicalMapKeyDisplayWriter::new(self.max_bytes)?;
        let display_result = fmt::write(&mut writer, format_args!("{value}"));
        if let Some(error) = writer.error.take() {
            return Err(error);
        }
        if display_result.is_err() {
            return Err(CanonicalTextWriteError::PayloadValidation {
                reason: "map key Display implementation rejected canonical formatting".to_string(),
            });
        }
        Ok(writer.finish())
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

/// 增量收集 Display 文本并计算最终 JSON 键长度；`fmt::Write` 只能返回通用错误，因此另存具体预算错误。
struct CanonicalMapKeyDisplayWriter {
    value: String,
    encoded_bytes: usize,
    max_bytes: usize,
    error: Option<CanonicalTextWriteError>,
}

impl CanonicalMapKeyDisplayWriter {
    fn new(max_bytes: usize) -> Result<Self, CanonicalTextWriteError> {
        ensure_within_limit(2, max_bytes)?;
        Ok(Self {
            value: String::new(),
            encoded_bytes: 2,
            max_bytes,
            error: None,
        })
    }

    fn finish(self) -> CanonicalMapKey {
        CanonicalMapKey {
            value: self.value,
            encoded_bytes: self.encoded_bytes,
        }
    }
}

impl fmt::Write for CanonicalMapKeyDisplayWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let additional = match json_string_content_len(value, self.max_bytes) {
            Ok(additional) => additional,
            Err(error) => {
                self.error = Some(error);
                return Err(fmt::Error);
            }
        };
        let found = match self.encoded_bytes.checked_add(additional) {
            Some(found) => found,
            None => {
                self.error = Some(output_too_large(self.max_bytes, usize::MAX));
                return Err(fmt::Error);
            }
        };
        if let Err(error) = ensure_within_limit(found, self.max_bytes) {
            self.error = Some(error);
            return Err(fmt::Error);
        }
        self.value.push_str(value);
        self.encoded_bytes = found;
        Ok(())
    }
}

// JSON 对象键仍是字符串；先拒绝非有限值，再将 JSON 数字文本按带引号键长度校验。
fn finite_key(value: f64, max_bytes: usize) -> Result<CanonicalMapKey, CanonicalTextWriteError> {
    if !value.is_finite() {
        return Err(CanonicalTextWriteError::NonFinite { value });
    }
    let value =
        serde_json::to_string(&value).map_err(|source| CanonicalTextWriteError::PayloadEncode {
            reason: source.to_string(),
        })?;
    CanonicalMapKey::from_string(value, max_bytes)
}

fn reject_raw_value(name: &str) -> Result<(), CanonicalTextWriteError> {
    if name == SERDE_JSON_RAW_VALUE_TOKEN {
        return Err(CanonicalTextWriteError::PayloadValidation {
            reason: "serde_json RawValue is outside the canonical versioned payload domain"
                .to_string(),
        });
    }
    Ok(())
}

// 预算按转义后的 JSON 键长度计费，防止短原串在实际输出时越过上限。
fn json_string_encoded_len(
    value: &str,
    max_bytes: usize,
) -> Result<usize, CanonicalTextWriteError> {
    let content_bytes = json_string_content_len(value, max_bytes)?;
    let found = content_bytes
        .checked_add(2)
        .ok_or_else(|| output_too_large(max_bytes, usize::MAX))?;
    ensure_within_limit(found, max_bytes)?;
    Ok(found)
}

// 此处的转义长度必须与 json_string.rs 的实际输出规则一致，预算才不会低估最终键长度。
fn json_string_content_len(
    value: &str,
    max_bytes: usize,
) -> Result<usize, CanonicalTextWriteError> {
    let mut bytes = 0_usize;
    for character in value.chars() {
        let additional = match character {
            '"' | '\\' | '\u{08}' | '\u{0C}' | '\n' | '\r' | '\t' => 2,
            control if control <= '\u{1F}' => 6,
            character => character.len_utf8(),
        };
        bytes = bytes
            .checked_add(additional)
            .ok_or_else(|| output_too_large(max_bytes, usize::MAX))?;
    }
    Ok(bytes)
}

fn ensure_within_limit(found: usize, max_bytes: usize) -> Result<(), CanonicalTextWriteError> {
    if found > max_bytes {
        return Err(CanonicalTextWriteError::OutputTooLarge {
            max: max_bytes,
            found,
        });
    }
    Ok(())
}

fn output_too_large(max_bytes: usize, found: usize) -> CanonicalTextWriteError {
    CanonicalTextWriteError::OutputTooLarge {
        max: max_bytes,
        found,
    }
}

fn invalid_key<T>() -> Result<T, CanonicalTextWriteError> {
    Err(CanonicalTextWriteError::PayloadEncode {
        reason: "JSON object keys must be scalar".to_string(),
    })
}
