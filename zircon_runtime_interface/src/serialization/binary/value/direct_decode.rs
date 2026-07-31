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

#[derive(Debug)]
pub(in crate::serialization) enum DirectBinaryDecodeError {
    Malformed(bincode::Error),
    Invalid(BinaryValueError),
    Payload(String),
}

pub(in crate::serialization) fn decode_binary_value_direct<T>(
    value: BinaryValue,
) -> Result<T, DirectBinaryDecodeError>
where
    T: DeserializeOwned,
{
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

fn validate_depth(found: usize) -> Result<(), BinaryValueError> {
    if found <= MAX_BINARY_DEPTH {
        return Ok(());
    }
    Err(BinaryValueError::DepthLimitExceeded {
        max: MAX_BINARY_DEPTH,
        found,
    })
}

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

struct BinaryValueDeserializer<'nodes> {
    nodes: &'nodes [BinaryNode],
    index: usize,
}

impl<'nodes> BinaryValueDeserializer<'nodes> {
    fn next_node(&mut self) -> Result<&BinaryNode, DecodeError> {
        let node = self.nodes.get(self.index).ok_or_else(|| {
            de::Error::custom("binary value ended before a typed payload completed")
        })?;
        self.index += 1;
        Ok(node)
    }

    fn peek_node(&self) -> Result<&BinaryNode, DecodeError> {
        self.nodes
            .get(self.index)
            .ok_or_else(|| de::Error::custom("binary value ended before a typed payload completed"))
    }

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

struct BinarySequenceAccess<'decoder, 'nodes> {
    decoder: &'decoder mut BinaryValueDeserializer<'nodes>,
    remaining: usize,
}

impl<'de, 'decoder, 'nodes> SeqAccess<'de> for BinarySequenceAccess<'decoder, 'nodes> {
    type Error = DecodeError;

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

struct BinaryMapAccess<'decoder, 'nodes> {
    decoder: &'decoder mut BinaryValueDeserializer<'nodes>,
    remaining: usize,
    awaiting_value: bool,
}

impl<'de, 'decoder, 'nodes> MapAccess<'de> for BinaryMapAccess<'decoder, 'nodes> {
    type Error = DecodeError;

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

struct UnitEnumAccess {
    variant: String,
}

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

struct UnitVariantAccess;

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

struct ValueEnumAccess<'decoder, 'nodes> {
    decoder: &'decoder mut BinaryValueDeserializer<'nodes>,
    variant: String,
}

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

struct ValueVariantAccess<'decoder, 'nodes> {
    decoder: &'decoder mut BinaryValueDeserializer<'nodes>,
}

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

struct BinaryObjectKeyDeserializer {
    key: String,
}

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
