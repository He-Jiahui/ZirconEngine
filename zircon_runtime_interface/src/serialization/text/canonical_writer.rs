use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;

use serde::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde::{Serialize, Serializer};

#[path = "canonical_writer/json_string.rs"]
mod json_string;
#[path = "canonical_writer/output.rs"]
mod output;

use self::json_string::{write_json_display, write_json_string, write_json_string_preaccounted};
use self::output::OutputBudget;
pub(super) use self::output::{CountingWriter, COPY_BUFFER_BYTES};
pub(in crate::serialization) use self::output::{
    MAX_CANONICAL_NESTING_DEPTH, MAX_CANONICAL_OBJECT_ENTRIES,
};
use super::super::write_error::CanonicalTextWriteError;
use super::super::SerializationBudget;
use super::canonical_map_key::{CanonicalMapKey, CanonicalMapKeySerializer};
use super::canonical_spool::TempSpool;
use super::wire::MAX_TEXT_DOCUMENT_BYTES;

pub(in crate::serialization) const SERDE_JSON_RAW_VALUE_TOKEN: &str =
    "$serde_json::private::RawValue";

/// Serializes canonical JSON directly to `sink` and appends exactly one newline.
///
/// Arrays are emitted directly. Object values are spooled while their keys are
/// collected, allowing lexical key ordering and duplicate-key replacement
/// without retaining a second complete document in memory.
pub(in crate::serialization) fn write_canonical_text<T, W>(
    value: &T,
    sink: &mut W,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    write_canonical_text_with_limit(value, sink, MAX_TEXT_DOCUMENT_BYTES)
}

/// Streams canonical JSON under a caller-owned archive budget.
pub(in crate::serialization) fn write_canonical_text_with_budget<T, W>(
    value: &T,
    sink: &mut W,
    budget: SerializationBudget,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    write_canonical_text_with_limit(value, sink, budget.max_output_bytes())
}

fn write_canonical_text_with_limit<T, W>(
    value: &T,
    sink: &mut W,
    max_bytes: usize,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    let mut budget = OutputBudget::new(max_bytes);
    let mut output = CountingWriter::new(sink, &mut budget);
    {
        let mut serializer = CanonicalTextSerializer {
            output: &mut output,
            depth: 0,
        };
        value.serialize(&mut serializer)?;
    }
    output.write_counted(b"\n")?;
    output.flush()?;
    let bytes = output.budget.bytes;
    drop(output);
    Ok(bytes)
}

#[cfg(test)]
pub(in crate::serialization) fn write_canonical_text_with_limit_for_test<T, W>(
    value: &T,
    sink: &mut W,
    max_bytes: usize,
) -> Result<usize, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    write_canonical_text_with_limit(value, sink, max_bytes)
}

struct CanonicalTextSerializer<'output, 'sink, 'budget, W: Write + ?Sized> {
    output: &'output mut CountingWriter<'sink, 'budget, W>,
    depth: usize,
}

impl<'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalTextSerializer<'output, 'sink, 'budget, W>
{
    fn write_indent(&mut self, depth: usize) -> Result<(), CanonicalTextWriteError> {
        for _ in 0..depth {
            self.output.write_counted(b"  ")?;
        }
        Ok(())
    }

    fn write_json_string(&mut self, value: &str) -> Result<(), CanonicalTextWriteError> {
        write_json_string(self.output, value)
    }

    fn write_scalar<T>(&mut self, value: &T) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        let encoded = serde_json::to_string(value).map_err(|source| {
            CanonicalTextWriteError::PayloadEncode {
                reason: source.to_string(),
            }
        })?;
        self.output.write_counted(encoded.as_bytes())
    }
}

struct CanonicalArray<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> {
    serializer: &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>,
    first: bool,
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalArray<'serializer, 'output, 'sink, 'budget, W>
{
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        let depth = self.serializer.depth + 1;
        self.serializer.output.budget.ensure_nesting_depth(depth)?;
        if self.first {
            self.serializer.output.write_counted(b"\n")?;
            self.first = false;
        } else {
            self.serializer.output.write_counted(b",\n")?;
        }
        self.serializer.write_indent(depth)?;
        let output = &mut *self.serializer.output;
        let mut nested = CanonicalTextSerializer { output, depth };
        value.serialize(&mut nested)
    }

    fn finish(self) -> Result<(), CanonicalTextWriteError> {
        if !self.first {
            self.serializer.output.write_counted(b"\n")?;
            self.serializer.write_indent(self.serializer.depth)?;
        }
        self.serializer.output.write_counted(b"]")
    }
}

struct CanonicalObject<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> {
    serializer: &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>,
    entries: BTreeMap<String, CanonicalObjectEntry>,
    pending_key: Option<CanonicalMapKey>,
}

struct CanonicalObjectEntry {
    value: TempSpool,
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalObject<'serializer, 'output, 'sink, 'budget, W>
{
    fn serialize_value_for_key<T>(
        &mut self,
        key: CanonicalMapKey,
        value: &T,
        depth: usize,
    ) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        if let Some(previous) = self.entries.remove(key.value()) {
            self.serializer
                .output
                .budget
                .release(previous.value.accounted_bytes);
        }
        let (key, _) = key.into_parts();
        let spool = serialize_to_spool(self.serializer.output, depth, value)?;
        self.entries
            .insert(key, CanonicalObjectEntry { value: spool });
        Ok(())
    }

    fn prepare_key(
        &mut self,
        key: CanonicalMapKey,
    ) -> Result<CanonicalMapKey, CanonicalTextWriteError> {
        if !self.entries.contains_key(key.value()) {
            self.serializer
                .output
                .budget
                .ensure_object_entries(self.entries.len().saturating_add(1))?;
            self.serializer.output.budget.reserve(key.encoded_bytes())?;
        }
        Ok(key)
    }

    fn finish(self) -> Result<(), CanonicalTextWriteError> {
        if self.pending_key.is_some() {
            return Err(CanonicalTextWriteError::PayloadEncode {
                reason: "map ended without a value".to_string(),
            });
        }
        write_object(self.serializer.output, self.serializer.depth, self.entries)
    }
}

struct CanonicalTupleVariant<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> {
    serializer: &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>,
    variant: &'static str,
    spool: TempSpool,
    first: bool,
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalTupleVariant<'serializer, 'output, 'sink, 'budget, W>
{
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        let depth = self.serializer.depth + 2;
        self.serializer.output.budget.ensure_nesting_depth(depth)?;
        {
            let mut output =
                CountingWriter::new_spool(&mut self.spool, &mut *self.serializer.output.budget);
            if self.first {
                output.write_counted(b"\n")?;
                self.first = false;
            } else {
                output.write_counted(b",\n")?;
            }
            write_indent(&mut output, depth)?;
            let mut nested = CanonicalTextSerializer {
                output: &mut output,
                depth,
            };
            value.serialize(&mut nested)?;
            output.flush()?;
        }
        Ok(())
    }

    fn finish(mut self) -> Result<(), CanonicalTextWriteError> {
        {
            let mut output =
                CountingWriter::new_spool(&mut self.spool, &mut *self.serializer.output.budget);
            if !self.first {
                output.write_counted(b"\n")?;
                write_indent(&mut output, self.serializer.depth + 1)?;
            }
            output.write_counted(b"]")?;
            output.flush()?;
        }
        write_single_object(
            self.serializer.output,
            self.serializer.depth,
            self.variant.to_string(),
            self.spool,
        )
    }
}

struct CanonicalStructVariant<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> {
    serializer: &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>,
    variant: &'static str,
    entries: BTreeMap<String, CanonicalObjectEntry>,
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized>
    CanonicalStructVariant<'serializer, 'output, 'sink, 'budget, W>
{
    fn serialize_field<T>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), CanonicalTextWriteError>
    where
        T: ?Sized + Serialize,
    {
        let key = CanonicalMapKey::from_str(key, self.serializer.output.budget.max_bytes())?;
        if !self.entries.contains_key(key.value()) {
            self.serializer
                .output
                .budget
                .ensure_object_entries(self.entries.len().saturating_add(1))?;
            self.serializer.output.budget.reserve(key.encoded_bytes())?;
        }
        if let Some(previous) = self.entries.remove(key.value()) {
            self.serializer
                .output
                .budget
                .release(previous.value.accounted_bytes);
        }
        let (key, _) = key.into_parts();
        let spool = serialize_to_spool(self.serializer.output, self.serializer.depth + 2, value)?;
        self.entries
            .insert(key, CanonicalObjectEntry { value: spool });
        Ok(())
    }

    fn finish(self) -> Result<(), CanonicalTextWriteError> {
        let mut inner = TempSpool::new(self.serializer.output.budget.spool_attempt());
        {
            let mut output =
                CountingWriter::new_spool(&mut inner, &mut *self.serializer.output.budget);
            write_object(&mut output, self.serializer.depth + 1, self.entries)?;
            output.flush()?;
        }
        write_single_object(
            self.serializer.output,
            self.serializer.depth,
            self.variant.to_string(),
            inner,
        )
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> Serializer
    for &'serializer mut CanonicalTextSerializer<'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    type SerializeSeq = CanonicalArray<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeTuple = CanonicalArray<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeTupleStruct = CanonicalArray<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeTupleVariant = CanonicalTupleVariant<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeMap = CanonicalObject<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeStruct = CanonicalObject<'serializer, 'output, 'sink, 'budget, W>;
    type SerializeStructVariant = CanonicalStructVariant<'serializer, 'output, 'sink, 'budget, W>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        self.output
            .write_counted(if value { b"true" } else { b"false" })
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        i64::try_from(value)
            .map_err(|_| CanonicalTextWriteError::PayloadEncode {
                reason: "number out of range".to_string(),
            })
            .and_then(|value| self.write_scalar(&value))
    }
    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        self.write_scalar(&value)
    }
    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
        u64::try_from(value)
            .map_err(|_| CanonicalTextWriteError::PayloadEncode {
                reason: "number out of range".to_string(),
            })
            .and_then(|value| self.write_scalar(&value))
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        if value.is_finite() {
            self.write_scalar(&value)
        } else {
            Err(CanonicalTextWriteError::NonFinite {
                value: value as f64,
            })
        }
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        if value.is_finite() {
            self.write_scalar(&value)
        } else {
            Err(CanonicalTextWriteError::NonFinite { value })
        }
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.write_json_string(&value.to_string())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        self.write_json_string(value)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut array = self.serialize_seq(Some(value.len()))?;
        for byte in value {
            array.serialize_element(byte)?;
        }
        SerializeSeq::end(array)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.output.write_counted(b"null")
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
        self.write_json_string(variant)
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
            return Err(CanonicalTextWriteError::PayloadValidation {
                reason: "serde_json RawValue is outside the canonical versioned payload domain"
                    .to_string(),
            });
        }
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let spool = serialize_to_spool(self.output, self.depth + 1, value)?;
        write_single_object(self.output, self.depth, variant.to_string(), spool)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.output.write_counted(b"[")?;
        Ok(CanonicalArray {
            serializer: self,
            first: true,
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(None)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let mut spool = TempSpool::new(self.output.budget.spool_attempt());
        {
            let mut output = CountingWriter::new_spool(&mut spool, &mut *self.output.budget);
            output.write_counted(b"[")?;
            output.flush()?;
        }
        Ok(CanonicalTupleVariant {
            serializer: self,
            variant,
            spool,
            first: true,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(CanonicalObject {
            serializer: self,
            entries: BTreeMap::new(),
            pending_key: None,
        })
    }

    fn serialize_struct(
        self,
        name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if name == SERDE_JSON_RAW_VALUE_TOKEN {
            return Err(CanonicalTextWriteError::PayloadValidation {
                reason: "serde_json RawValue is outside the canonical versioned payload domain"
                    .to_string(),
            });
        }
        Ok(CanonicalObject {
            serializer: self,
            entries: BTreeMap::new(),
            pending_key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(CanonicalStructVariant {
            serializer: self,
            variant,
            entries: BTreeMap::new(),
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + fmt::Display,
    {
        write_json_display(self.output, value)
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeSeq
    for CanonicalArray<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeTuple
    for CanonicalArray<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeTupleStruct
    for CanonicalArray<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_element(value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeTupleVariant
    for CanonicalTupleVariant<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_field(value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeMap
    for CanonicalObject<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.pending_key.is_some() {
            return Err(CanonicalTextWriteError::PayloadEncode {
                reason: "map key was not followed by a value".to_string(),
            });
        }
        let key = key.serialize(CanonicalMapKeySerializer::new(
            self.serializer.output.budget.max_bytes(),
        ))?;
        self.pending_key = Some(self.prepare_key(key)?);
        Ok(())
    }
    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key =
            self.pending_key
                .take()
                .ok_or_else(|| CanonicalTextWriteError::PayloadEncode {
                    reason: "map value was emitted without a key".to_string(),
                })?;
        self.serialize_value_for_key(key, value, self.serializer.depth + 1)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeStruct
    for CanonicalObject<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = CanonicalMapKey::from_str(key, self.serializer.output.budget.max_bytes())?;
        let key = self.prepare_key(key)?;
        self.serialize_value_for_key(key, value, self.serializer.depth + 1)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

impl<'serializer, 'output, 'sink, 'budget, W: Write + ?Sized> SerializeStructVariant
    for CanonicalStructVariant<'serializer, 'output, 'sink, 'budget, W>
{
    type Ok = ();
    type Error = CanonicalTextWriteError;
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_field(key, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.finish()
    }
}

fn serialize_to_spool<T, W>(
    parent: &mut CountingWriter<'_, '_, W>,
    depth: usize,
    value: &T,
) -> Result<TempSpool, CanonicalTextWriteError>
where
    T: ?Sized + Serialize,
    W: Write + ?Sized,
{
    let before = parent.budget.bytes;
    parent.budget.ensure_nesting_depth(depth)?;
    let mut spool = TempSpool::new(parent.budget.spool_attempt());
    {
        let mut output = CountingWriter::new_spool(&mut spool, &mut *parent.budget);
        let mut serializer = CanonicalTextSerializer {
            output: &mut output,
            depth,
        };
        value.serialize(&mut serializer)?;
        output.flush()?;
    }
    spool.finish_write()?;
    spool.accounted_bytes = parent.budget.bytes.saturating_sub(before);
    Ok(spool)
}

fn write_single_object<W>(
    output: &mut CountingWriter<'_, '_, W>,
    depth: usize,
    key: String,
    value: TempSpool,
) -> Result<(), CanonicalTextWriteError>
where
    W: Write + ?Sized,
{
    output
        .budget
        .ensure_nesting_depth(depth.saturating_add(1))?;
    let key = CanonicalMapKey::from_string(key, output.budget.max_bytes())?;
    output.budget.reserve(key.encoded_bytes())?;
    let (key, _) = key.into_parts();
    let mut entries = BTreeMap::new();
    entries.insert(key, CanonicalObjectEntry { value });
    write_object(output, depth, entries)
}

fn write_object<W>(
    output: &mut CountingWriter<'_, '_, W>,
    depth: usize,
    mut entries: BTreeMap<String, CanonicalObjectEntry>,
) -> Result<(), CanonicalTextWriteError>
where
    W: Write + ?Sized,
{
    if entries.is_empty() {
        return output.write_counted(b"{}");
    }
    output.write_counted(b"{\n")?;
    let entry_count = entries.len();
    for (index, (key, entry)) in entries.iter_mut().enumerate() {
        write_indent(output, depth + 1)?;
        write_json_string_preaccounted(output, key)?;
        output.write_counted(b": ")?;
        entry.value.copy_to(output)?;
        if index + 1 != entry_count {
            output.write_counted(b",")?;
        }
        output.write_counted(b"\n")?;
    }
    write_indent(output, depth)?;
    output.write_counted(b"}")
}

fn write_indent<W>(
    output: &mut CountingWriter<'_, '_, W>,
    depth: usize,
) -> Result<(), CanonicalTextWriteError>
where
    W: Write + ?Sized,
{
    for _ in 0..depth {
        output.write_counted(b"  ")?;
    }
    Ok(())
}
