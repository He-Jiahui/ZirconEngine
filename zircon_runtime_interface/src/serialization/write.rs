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
        SERDE_JSON_RAW_VALUE_TOKEN, write_canonical_text, write_canonical_text_unbounded,
    },
    document::TextDocument,
    envelope::TextEnvelope,
};
use super::{CanonicalTextWriteError, Format, PayloadHeader, VersionedSchema, WriteError};

/// Encodes a payload with the current schema header and canonical text rules.
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
pub fn write_canonical_text_to<T, W>(
    value: &T,
    sink: &mut W,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    write_canonical_text_unbounded(value, sink)
}

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
enum FloatValidationError {
    NonFinite { value: f64 },
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
struct FiniteFloatGuard;

struct FiniteFloatCompound;

macro_rules! finite_integer_methods {
    ($($method:ident($type:ty)),* $(,)?) => {
        $(fn $method(self, _value: $type) -> Result<Self::Ok, Self::Error> { Ok(()) })*
    };
}

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

fn validate_float(value: f64) -> Result<(), FloatValidationError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(FloatValidationError::NonFinite { value })
    }
}

fn current_header<T>() -> PayloadHeader
where
    T: VersionedSchema,
{
    PayloadHeader {
        schema_id: T::SCHEMA.clone(),
        schema_version: T::VERSION,
    }
}
