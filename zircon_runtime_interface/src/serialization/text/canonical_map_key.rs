use std::fmt;

use serde::ser::{Impossible, Serializer};
use serde::Serialize;

use super::super::write_error::CanonicalTextWriteError;
use super::canonical_writer::SERDE_JSON_RAW_VALUE_TOKEN;

/// A normalized JSON object key and the bytes its quoted canonical spelling owns.
pub(super) struct CanonicalMapKey {
    value: String,
    encoded_bytes: usize,
}

impl CanonicalMapKey {
    pub(super) fn from_str(value: &str, max_bytes: usize) -> Result<Self, CanonicalTextWriteError> {
        let encoded_bytes = json_string_encoded_len(value, max_bytes)?;
        Ok(Self {
            value: value.to_string(),
            encoded_bytes,
        })
    }

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
pub(super) struct CanonicalMapKeySerializer {
    max_bytes: usize,
}

impl CanonicalMapKeySerializer {
    pub(super) fn new(max_bytes: usize) -> Self {
        Self { max_bytes }
    }
}

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
