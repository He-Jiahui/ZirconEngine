use std::collections::BTreeMap;
use std::fmt;

use zircon_runtime::core::framework::ai::{AiBlackboardSchemaDescriptor, AiBlackboardValueType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// A stable key location inside one schema-compiled blackboard layout.
pub struct BlackboardSlot {
    value_type: AiBlackboardValueType,
    offset: u32,
    generation_index: u32,
}

impl BlackboardSlot {
    /// Returns the value type owned by this slot.
    pub const fn value_type(self) -> AiBlackboardValueType {
        self.value_type
    }

    /// Returns the offset within the slot's type partition.
    pub const fn offset(self) -> u32 {
        self.offset
    }

    /// Returns the index used by generation and observer arrays.
    pub const fn generation_index(self) -> u32 {
        self.generation_index
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Immutable key-to-slot mapping compiled from a validated schema descriptor.
pub struct BlackboardLayout {
    schema_id: String,
    slots: BTreeMap<String, BlackboardSlot>,
    keys: Box<[String]>,
    counts: [u32; 6],
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Error returned when a schema cannot be compiled into a dense layout.
pub enum BlackboardLayoutError {
    /// The schema declares the same key more than once.
    DuplicateKey { key: String },
    /// The schema declares a value type unsupported by the runtime store.
    UnknownValueType { key: String, value_type: String },
}

impl fmt::Display for BlackboardLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateKey { key } => write!(formatter, "blackboard key `{key}` is duplicated"),
            Self::UnknownValueType { key, value_type } => write!(
                formatter,
                "blackboard key `{key}` uses unknown value type `{value_type}`"
            ),
        }
    }
}

impl std::error::Error for BlackboardLayoutError {}

impl BlackboardLayout {
    /// Compiles a schema into stable, per-type dense partitions.
    pub fn from_schema(
        descriptor: &AiBlackboardSchemaDescriptor,
    ) -> Result<Self, BlackboardLayoutError> {
        let mut slots = BTreeMap::new();
        let mut keys = Vec::with_capacity(descriptor.keys.len());
        let mut counts = [0_u32; 6];
        for key in &descriptor.keys {
            if slots.contains_key(key.key.as_str()) {
                return Err(BlackboardLayoutError::DuplicateKey {
                    key: key.key.clone(),
                });
            }
            let value_type = key.expected_value_type().ok_or_else(|| {
                BlackboardLayoutError::UnknownValueType {
                    key: key.key.clone(),
                    value_type: key.value_type.clone(),
                }
            })?;
            let type_index = value_type_index(value_type);
            let slot = BlackboardSlot {
                value_type,
                offset: counts[type_index],
                generation_index: keys.len() as u32,
            };
            counts[type_index] += 1;
            slots.insert(key.key.clone(), slot);
            keys.push(key.key.clone());
        }
        Ok(Self {
            schema_id: descriptor.id.clone(),
            slots,
            keys: keys.into_boxed_slice(),
            counts,
        })
    }

    /// Returns the source schema id.
    pub fn schema_id(&self) -> &str {
        &self.schema_id
    }

    /// Resolves a schema key once into its dense runtime slot.
    pub fn resolve(&self, key: &str) -> Option<BlackboardSlot> {
        self.slots.get(key).copied()
    }

    /// Returns the schema key associated with a slot.
    pub fn key_for_slot(&self, slot: BlackboardSlot) -> Option<&str> {
        self.keys
            .get(slot.generation_index as usize)
            .map(String::as_str)
    }

    /// Returns the number of compiled keys.
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    pub(crate) fn slots(&self) -> impl Iterator<Item = (&str, BlackboardSlot)> {
        self.slots.iter().map(|(key, slot)| (key.as_str(), *slot))
    }

    pub(crate) const fn count(&self, value_type: AiBlackboardValueType) -> usize {
        self.counts[value_type_index(value_type)] as usize
    }
}

const fn value_type_index(value_type: AiBlackboardValueType) -> usize {
    match value_type {
        AiBlackboardValueType::Bool => 0,
        AiBlackboardValueType::Integer => 1,
        AiBlackboardValueType::Scalar => 2,
        AiBlackboardValueType::String => 3,
        AiBlackboardValueType::Vec3 => 4,
        AiBlackboardValueType::Entity => 5,
    }
}
