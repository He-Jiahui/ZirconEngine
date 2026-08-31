use std::collections::{btree_map::Entry, BTreeMap};

use crate::ProtocolError;

const TAG_NULL: u8 = 0;
const TAG_FALSE: u8 = 1;
const TAG_TRUE: u8 = 2;
const TAG_NUMBER: u8 = 3;
const TAG_STRING: u8 = 4;
const TAG_ARRAY: u8 = 5;
const TAG_OBJECT: u8 = 6;
const LENGTH_BYTES: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub enum CommandValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<CommandValue>),
    Object(BTreeMap<String, CommandValue>),
}

impl CommandValue {
    pub fn object(
        entries: impl IntoIterator<Item = (String, CommandValue)>,
    ) -> Result<Self, ProtocolError> {
        let mut values = BTreeMap::new();
        for (key, value) in entries {
            match values.entry(key) {
                Entry::Vacant(vacant) => {
                    vacant.insert(value);
                }
                Entry::Occupied(occupied) => {
                    return Err(ProtocolError::DuplicateCommandObjectKey {
                        key: occupied.key().clone(),
                    });
                }
            }
        }
        Ok(Self::Object(values))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandValueLimits {
    pub max_total_bytes: usize,
    pub max_value_depth: usize,
    pub max_collection_entries: usize,
    pub max_string_bytes: usize,
}

impl Default for CommandValueLimits {
    fn default() -> Self {
        Self {
            max_total_bytes: 65_536,
            max_value_depth: 32,
            max_collection_entries: 4_096,
            max_string_bytes: 16_384,
        }
    }
}

/// Encodes a canonical, bounded command value for a payload contract that has an explicit schema.
pub fn encode_command_value(
    value: &CommandValue,
    limits: CommandValueLimits,
) -> Result<Vec<u8>, ProtocolError> {
    let mut writer = Writer::new(limits);
    writer.write_value(value, 0)?;
    Ok(writer.finish())
}

/// Decodes one canonical command value; callers still validate its command-specific schema.
pub fn decode_command_value(
    bytes: &[u8],
    limits: CommandValueLimits,
) -> Result<CommandValue, ProtocolError> {
    if bytes.len() > limits.max_total_bytes {
        return Err(limit_error(
            "command value bytes",
            bytes.len(),
            limits.max_total_bytes,
        ));
    }
    let mut reader = Reader::new(bytes, limits);
    let value = reader.read_value(0)?;
    reader.finish()?;
    Ok(value)
}

struct Writer {
    bytes: Vec<u8>,
    limits: CommandValueLimits,
}

impl Writer {
    fn new(limits: CommandValueLimits) -> Self {
        Self {
            bytes: Vec::new(),
            limits,
        }
    }

    fn write_value(&mut self, value: &CommandValue, depth: usize) -> Result<(), ProtocolError> {
        ensure_depth(depth, self.limits)?;
        match value {
            CommandValue::Null => self.push(&[TAG_NULL]),
            CommandValue::Bool(false) => self.push(&[TAG_FALSE]),
            CommandValue::Bool(true) => self.push(&[TAG_TRUE]),
            CommandValue::Number(value) => {
                if !value.is_finite() {
                    return Err(ProtocolError::NonFinite {
                        field: "command value number",
                        value: *value,
                    });
                }
                let value = if *value == 0.0 { 0.0 } else { *value };
                self.push(&[TAG_NUMBER])?;
                self.push(&value.to_le_bytes())
            }
            CommandValue::String(value) => {
                ensure_string(value.len(), self.limits)?;
                self.ensure_capacity(
                    1_usize
                        .checked_add(LENGTH_BYTES)
                        .and_then(|length| length.checked_add(value.len()))
                        .unwrap_or(usize::MAX),
                )?;
                self.push(&[TAG_STRING])?;
                self.write_string(value)
            }
            CommandValue::Array(values) => {
                ensure_collection("command value array", values.len(), self.limits)?;
                self.push(&[TAG_ARRAY])?;
                self.write_length("command value array", values.len())?;
                for value in values {
                    self.write_value(value, depth + 1)?;
                }
                Ok(())
            }
            CommandValue::Object(values) => {
                ensure_collection("command value object", values.len(), self.limits)?;
                self.push(&[TAG_OBJECT])?;
                self.write_length("command value object", values.len())?;
                for (key, value) in values {
                    self.write_string(key)?;
                    self.write_value(value, depth + 1)?;
                }
                Ok(())
            }
        }
    }

    fn write_string(&mut self, value: &str) -> Result<(), ProtocolError> {
        ensure_string(value.len(), self.limits)?;
        self.write_length("command value string", value.len())?;
        self.push(value.as_bytes())
    }

    fn write_length(&mut self, context: &'static str, length: usize) -> Result<(), ProtocolError> {
        let length =
            u32::try_from(length).map_err(|_| limit_error(context, length, u32::MAX as usize))?;
        self.push(&length.to_le_bytes())
    }

    fn push(&mut self, bytes: &[u8]) -> Result<(), ProtocolError> {
        self.ensure_capacity(bytes.len())?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn ensure_capacity(&self, additional: usize) -> Result<(), ProtocolError> {
        let actual = self
            .bytes
            .len()
            .checked_add(additional)
            .unwrap_or(usize::MAX);
        if actual > self.limits.max_total_bytes {
            Err(limit_error(
                "command value bytes",
                actual,
                self.limits.max_total_bytes,
            ))
        } else {
            Ok(())
        }
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
    limits: CommandValueLimits,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8], limits: CommandValueLimits) -> Self {
        Self {
            bytes,
            offset: 0,
            limits,
        }
    }

    fn read_value(&mut self, depth: usize) -> Result<CommandValue, ProtocolError> {
        ensure_depth(depth, self.limits)?;
        match self.read_byte("command value tag")? {
            TAG_NULL => Ok(CommandValue::Null),
            TAG_FALSE => Ok(CommandValue::Bool(false)),
            TAG_TRUE => Ok(CommandValue::Bool(true)),
            TAG_NUMBER => {
                let value = f64::from_le_bytes(
                    self.take("command value number", 8)?
                        .try_into()
                        .expect("fixed command value number slice"),
                );
                if !value.is_finite() {
                    return Err(ProtocolError::NonFinite {
                        field: "command value number",
                        value,
                    });
                }
                Ok(CommandValue::Number(if value == 0.0 { 0.0 } else { value }))
            }
            TAG_STRING => Ok(CommandValue::String(self.read_string()?)),
            TAG_ARRAY => {
                let length = self.read_collection_length("command value array")?;
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(self.read_value(depth + 1)?);
                }
                Ok(CommandValue::Array(values))
            }
            TAG_OBJECT => {
                let length = self.read_collection_length("command value object")?;
                let mut values = BTreeMap::new();
                for _ in 0..length {
                    let key = self.read_string()?;
                    let value = self.read_value(depth + 1)?;
                    match values.entry(key) {
                        Entry::Vacant(vacant) => {
                            vacant.insert(value);
                        }
                        Entry::Occupied(occupied) => {
                            return Err(ProtocolError::DuplicateCommandObjectKey {
                                key: occupied.key().clone(),
                            });
                        }
                    }
                }
                Ok(CommandValue::Object(values))
            }
            tag => Err(ProtocolError::UnknownCommandValueTag(tag)),
        }
    }

    fn read_string(&mut self) -> Result<String, ProtocolError> {
        let length = self.read_length("command value string")?;
        ensure_string(length, self.limits)?;
        let bytes = self.take("command value string", length)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ProtocolError::InvalidUtf8 {
            context: "command value string",
        })
    }

    fn read_collection_length(&mut self, context: &'static str) -> Result<usize, ProtocolError> {
        let length = self.read_length(context)?;
        ensure_collection(context, length, self.limits)?;
        Ok(length)
    }

    fn read_length(&mut self, context: &'static str) -> Result<usize, ProtocolError> {
        Ok(u32::from_le_bytes(
            self.take(context, LENGTH_BYTES)?
                .try_into()
                .expect("fixed command value length slice"),
        ) as usize)
    }

    fn read_byte(&mut self, context: &'static str) -> Result<u8, ProtocolError> {
        Ok(self.take(context, 1)?[0])
    }

    fn take(&mut self, context: &'static str, length: usize) -> Result<&'a [u8], ProtocolError> {
        let remaining = self.bytes.len().saturating_sub(self.offset);
        if remaining < length {
            return Err(ProtocolError::TruncatedPayload {
                context,
                needed: length,
                remaining,
            });
        }
        let start = self.offset;
        self.offset += length;
        Ok(&self.bytes[start..self.offset])
    }

    fn finish(self) -> Result<(), ProtocolError> {
        let remaining = self.bytes.len() - self.offset;
        if remaining == 0 {
            Ok(())
        } else {
            Err(ProtocolError::TrailingPayload { remaining })
        }
    }
}

fn ensure_depth(depth: usize, limits: CommandValueLimits) -> Result<(), ProtocolError> {
    if depth > limits.max_value_depth {
        Err(limit_error(
            "command value depth",
            depth,
            limits.max_value_depth,
        ))
    } else {
        Ok(())
    }
}

fn ensure_collection(
    context: &'static str,
    actual: usize,
    limits: CommandValueLimits,
) -> Result<(), ProtocolError> {
    if actual > limits.max_collection_entries {
        Err(limit_error(context, actual, limits.max_collection_entries))
    } else {
        Ok(())
    }
}

fn ensure_string(actual: usize, limits: CommandValueLimits) -> Result<(), ProtocolError> {
    if actual > limits.max_string_bytes {
        Err(limit_error(
            "command value string",
            actual,
            limits.max_string_bytes,
        ))
    } else {
        Ok(())
    }
}

fn limit_error(context: &'static str, actual: usize, maximum: usize) -> ProtocolError {
    ProtocolError::CollectionTooLarge {
        context,
        actual,
        maximum,
    }
}
