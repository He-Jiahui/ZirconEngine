use std::fmt;
use std::sync::Arc;

use zircon_runtime::core::framework::ai::{
    AiBlackboardEntry, AiBlackboardValue, AiBlackboardValueType,
};
use zircon_runtime::core::math::{Real, Vec3};

use super::{BlackboardLayout, BlackboardSlot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Result of a blackboard write.
pub struct BlackboardWriteOutcome {
    /// Slot that was addressed.
    pub slot: BlackboardSlot,
    /// Slot generation after the write.
    pub generation: u32,
    /// Whether the stored value changed.
    pub changed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Typed blackboard storage error.
pub enum BlackboardRuntimeError {
    /// The key is absent from the compiled layout.
    UnknownKey { key: String },
    /// The same key appears more than once in one synchronized snapshot.
    DuplicateKey { key: String },
    /// The value type does not match the compiled slot.
    TypeMismatch {
        key: String,
        expected: AiBlackboardValueType,
        actual: AiBlackboardValueType,
    },
    /// A scalar or vector contains a non-finite component.
    NonFiniteValue { key: String },
}

impl fmt::Display for BlackboardRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKey { key } => write!(formatter, "blackboard key `{key}` is unknown"),
            Self::DuplicateKey { key } => {
                write!(formatter, "blackboard key `{key}` is duplicated")
            }
            Self::TypeMismatch {
                key,
                expected,
                actual,
            } => write!(
                formatter,
                "blackboard key `{key}` expects {}, got {}",
                expected.as_str(),
                actual.as_str()
            ),
            Self::NonFiniteValue { key } => {
                write!(
                    formatter,
                    "blackboard key `{key}` contains a non-finite value"
                )
            }
        }
    }
}

impl std::error::Error for BlackboardRuntimeError {}

#[derive(Clone, Debug)]
/// Per-agent dense blackboard values, generations, and pending slot notifications.
pub struct BlackboardStore {
    layout: Arc<BlackboardLayout>,
    bools: Box<[Option<bool>]>,
    integers: Box<[Option<i64>]>,
    scalars: Box<[Option<Real>]>,
    strings: Box<[Option<String>]>,
    vectors: Box<[Option<Vec3>]>,
    entities: Box<[Option<u64>]>,
    generations: Box<[u32]>,
    entries_cache: Vec<AiBlackboardEntry>,
    pending_changes: Vec<BlackboardSlot>,
}

impl BlackboardStore {
    /// Creates an empty store for a compiled layout.
    pub fn new(layout: Arc<BlackboardLayout>) -> Self {
        Self {
            bools: empty_values(layout.count(AiBlackboardValueType::Bool)),
            integers: empty_values(layout.count(AiBlackboardValueType::Integer)),
            scalars: empty_values(layout.count(AiBlackboardValueType::Scalar)),
            strings: empty_values(layout.count(AiBlackboardValueType::String)),
            vectors: empty_values(layout.count(AiBlackboardValueType::Vec3)),
            entities: empty_values(layout.count(AiBlackboardValueType::Entity)),
            generations: vec![0; layout.key_count()].into_boxed_slice(),
            entries_cache: Vec::new(),
            pending_changes: Vec::new(),
            layout,
        }
    }

    /// Returns the immutable layout used by this store.
    pub fn layout(&self) -> &Arc<BlackboardLayout> {
        &self.layout
    }

    /// Returns the current generation for a slot.
    pub fn generation(&self, slot: BlackboardSlot) -> u32 {
        self.generations
            .get(slot.generation_index() as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Writes one key and records a notification only when its value changes.
    pub fn write(
        &mut self,
        key: &str,
        value: AiBlackboardValue,
    ) -> Result<BlackboardWriteOutcome, BlackboardRuntimeError> {
        let outcome = self.write_untracked(key, value)?;
        if outcome.changed {
            self.refresh_entries();
            self.record_changes(std::slice::from_ref(&outcome.slot));
        }
        Ok(outcome)
    }

    fn write_untracked(
        &mut self,
        key: &str,
        value: AiBlackboardValue,
    ) -> Result<BlackboardWriteOutcome, BlackboardRuntimeError> {
        let slot = self.validate_write(key, &value)?;
        let changed = match value {
            AiBlackboardValue::Bool(value) => replace(&mut self.bools, slot, value),
            AiBlackboardValue::Integer(value) => replace(&mut self.integers, slot, value),
            AiBlackboardValue::Scalar(value) => replace(&mut self.scalars, slot, value),
            AiBlackboardValue::String(value) => replace(&mut self.strings, slot, value),
            AiBlackboardValue::Vec3(value) => replace(&mut self.vectors, slot, value),
            AiBlackboardValue::Entity(value) => replace(&mut self.entities, slot, value),
        };
        if changed {
            let generation = &mut self.generations[slot.generation_index() as usize];
            *generation = generation.wrapping_add(1);
        }
        Ok(BlackboardWriteOutcome {
            slot,
            generation: self.generation(slot),
            changed,
        })
    }

    /// Atomically synchronizes a complete DTO snapshot into the dense store.
    pub fn synchronize(
        &mut self,
        entries: &[AiBlackboardEntry],
    ) -> Result<Vec<BlackboardSlot>, BlackboardRuntimeError> {
        let mut seen = std::collections::HashSet::with_capacity(entries.len());
        let mut present = vec![false; self.layout.key_count()];
        for entry in entries {
            if !seen.insert(entry.key.as_str()) {
                return Err(BlackboardRuntimeError::DuplicateKey {
                    key: entry.key.clone(),
                });
            }
            let slot = self.validate_write(&entry.key, &entry.value)?;
            present[slot.generation_index() as usize] = true;
        }
        let mut changed = Vec::new();
        for entry in entries {
            let outcome = self.write_untracked(&entry.key, entry.value.clone())?;
            if outcome.changed {
                changed.push(outcome.slot);
            }
        }
        let slots = self
            .layout
            .slots()
            .map(|(_, slot)| slot)
            .collect::<Vec<_>>();
        for slot in slots {
            if !present[slot.generation_index() as usize] && self.clear(slot) {
                let generation = &mut self.generations[slot.generation_index() as usize];
                *generation = generation.wrapping_add(1);
                changed.push(slot);
            }
        }
        if !changed.is_empty() {
            self.refresh_entries();
            self.record_changes(&changed);
        }
        Ok(changed)
    }

    pub(crate) fn drain_changed_slots(&mut self) -> Vec<BlackboardSlot> {
        std::mem::take(&mut self.pending_changes)
    }

    /// Returns a boundary DTO snapshot in deterministic key order.
    pub fn entries(&self) -> Vec<AiBlackboardEntry> {
        self.entries_cache.clone()
    }

    pub(crate) fn entries_ref(&self) -> &[AiBlackboardEntry] {
        &self.entries_cache
    }

    pub(crate) fn read(&self, slot: BlackboardSlot) -> Option<AiBlackboardValue> {
        match slot.value_type() {
            AiBlackboardValueType::Bool => value(&self.bools, slot).map(AiBlackboardValue::Bool),
            AiBlackboardValueType::Integer => {
                value(&self.integers, slot).map(AiBlackboardValue::Integer)
            }
            AiBlackboardValueType::Scalar => {
                value(&self.scalars, slot).map(AiBlackboardValue::Scalar)
            }
            AiBlackboardValueType::String => {
                value(&self.strings, slot).map(AiBlackboardValue::String)
            }
            AiBlackboardValueType::Vec3 => value(&self.vectors, slot).map(AiBlackboardValue::Vec3),
            AiBlackboardValueType::Entity => {
                value(&self.entities, slot).map(AiBlackboardValue::Entity)
            }
        }
    }

    fn refresh_entries(&mut self) {
        let mut entries = Vec::with_capacity(self.layout.key_count());
        for (key, slot) in self.layout.slots() {
            if let Some(value) = self.read(slot) {
                entries.push(AiBlackboardEntry::new(key, value));
            }
        }
        self.entries_cache = entries;
    }

    fn record_changes(&mut self, changed: &[BlackboardSlot]) {
        for slot in changed {
            if !self.pending_changes.contains(slot) {
                self.pending_changes.push(*slot);
            }
        }
    }

    fn clear(&mut self, slot: BlackboardSlot) -> bool {
        match slot.value_type() {
            AiBlackboardValueType::Bool => take(&mut self.bools, slot),
            AiBlackboardValueType::Integer => take(&mut self.integers, slot),
            AiBlackboardValueType::Scalar => take(&mut self.scalars, slot),
            AiBlackboardValueType::String => take(&mut self.strings, slot),
            AiBlackboardValueType::Vec3 => take(&mut self.vectors, slot),
            AiBlackboardValueType::Entity => take(&mut self.entities, slot),
        }
    }

    fn validate_write(
        &self,
        key: &str,
        value: &AiBlackboardValue,
    ) -> Result<BlackboardSlot, BlackboardRuntimeError> {
        let slot = self
            .layout
            .resolve(key)
            .ok_or_else(|| BlackboardRuntimeError::UnknownKey {
                key: key.to_string(),
            })?;
        let actual = value.value_type();
        if slot.value_type() != actual {
            return Err(BlackboardRuntimeError::TypeMismatch {
                key: key.to_string(),
                expected: slot.value_type(),
                actual,
            });
        }
        if !value.is_finite() {
            return Err(BlackboardRuntimeError::NonFiniteValue {
                key: key.to_string(),
            });
        }
        Ok(slot)
    }
}

fn empty_values<T>(count: usize) -> Box<[Option<T>]> {
    std::iter::repeat_with(|| None)
        .take(count)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn replace<T: PartialEq>(values: &mut [Option<T>], slot: BlackboardSlot, value: T) -> bool {
    let target = &mut values[slot.offset() as usize];
    if target.as_ref() == Some(&value) {
        false
    } else {
        *target = Some(value);
        true
    }
}

fn value<T: Clone>(values: &[Option<T>], slot: BlackboardSlot) -> Option<T> {
    values.get(slot.offset() as usize).cloned().flatten()
}

fn take<T>(values: &mut [Option<T>], slot: BlackboardSlot) -> bool {
    values[slot.offset() as usize].take().is_some()
}
