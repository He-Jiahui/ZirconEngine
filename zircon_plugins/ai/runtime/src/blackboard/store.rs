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
    entry_positions: Box<[Option<u32>]>,
    pending_changes: Vec<BlackboardSlot>,
    pending_change_flags: Box<[bool]>,
    synchronize_epoch: u32,
    synchronize_marks: Box<[u32]>,
    synchronize_slots: Vec<BlackboardSlot>,
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
            entries_cache: Vec::with_capacity(layout.key_count()),
            entry_positions: vec![None; layout.key_count()].into_boxed_slice(),
            pending_changes: Vec::new(),
            pending_change_flags: vec![false; layout.key_count()].into_boxed_slice(),
            synchronize_epoch: 0,
            synchronize_marks: vec![0; layout.key_count()].into_boxed_slice(),
            synchronize_slots: Vec::with_capacity(layout.key_count()),
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
            self.refresh_entry(outcome.slot);
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
        Ok(self.write_validated(slot, value))
    }

    fn write_validated(
        &mut self,
        slot: BlackboardSlot,
        value: AiBlackboardValue,
    ) -> BlackboardWriteOutcome {
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
        BlackboardWriteOutcome {
            slot,
            generation: self.generation(slot),
            changed,
        }
    }

    /// Atomically synchronizes a complete DTO snapshot into the dense store.
    pub fn synchronize(
        &mut self,
        entries: &[AiBlackboardEntry],
    ) -> Result<Vec<BlackboardSlot>, BlackboardRuntimeError> {
        // 先验证整个快照，再写入和清除旧槽位；失败时值、代次和通知保持原样。
        let synchronize_epoch = self.next_synchronize_epoch();
        self.synchronize_slots.clear();
        for entry in entries {
            let slot = self.resolve_slot(&entry.key)?;
            let generation_index = slot.generation_index() as usize;
            if self.synchronize_marks[generation_index] == synchronize_epoch {
                return Err(BlackboardRuntimeError::DuplicateKey {
                    key: entry.key.clone(),
                });
            }
            self.validate_slot_value(&entry.key, slot, &entry.value)?;
            self.synchronize_marks[generation_index] = synchronize_epoch;
            self.synchronize_slots.push(slot);
        }
        let mut changed = Vec::new();
        for (entry_index, entry) in entries.iter().enumerate() {
            let slot = self.synchronize_slots[entry_index];
            let outcome = self.write_validated(slot, entry.value.clone());
            if outcome.changed {
                changed.push(outcome.slot);
            }
        }
        let layout = Arc::clone(&self.layout);
        for (_, slot) in layout.slots() {
            if self.synchronize_marks[slot.generation_index() as usize] != synchronize_epoch
                && self.clear(slot)
            {
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

    // 代次回绕时清空标记，避免旧标记被误认作本次快照中的键。
    fn next_synchronize_epoch(&mut self) -> u32 {
        let next_epoch = self.synchronize_epoch.wrapping_add(1);
        if next_epoch == 0 {
            self.synchronize_marks.fill(0);
            self.synchronize_epoch = 1;
        } else {
            self.synchronize_epoch = next_epoch;
        }
        self.synchronize_epoch
    }

    pub(crate) fn drain_changed_slots(&mut self) -> Vec<BlackboardSlot> {
        let changed = std::mem::take(&mut self.pending_changes);
        for slot in &changed {
            self.pending_change_flags[slot.generation_index() as usize] = false;
        }
        changed
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

    // 已缓存键按位置直接更新；插入或删除后只修正受影响的后续位置。
    fn refresh_entry(&mut self, slot: BlackboardSlot) {
        let value = self.read(slot);
        let generation_index = slot.generation_index() as usize;
        if let Some(position) = self.entry_positions[generation_index].map(|value| value as usize) {
            match value {
                Some(value) => self.entries_cache[position].value = value,
                None => {
                    self.entries_cache.remove(position);
                    self.entry_positions[generation_index] = None;
                    self.refresh_entry_positions_from(position);
                }
            }
            return;
        }
        let key = self
            .layout
            .key_for_slot(slot)
            .expect("compiled blackboard slot must belong to its layout");
        match (
            self.entries_cache
                .binary_search_by(|entry| entry.key.as_str().cmp(key)),
            value,
        ) {
            (Ok(index), Some(value)) => {
                self.entries_cache[index].value = value;
                self.entry_positions[generation_index] = Some(index as u32);
            }
            (Ok(index), None) => {
                self.entries_cache.remove(index);
                self.refresh_entry_positions_from(index);
            }
            (Err(index), Some(value)) => {
                self.entries_cache
                    .insert(index, AiBlackboardEntry::new(key, value));
                self.refresh_entry_positions_from(index);
            }
            (Err(_), None) => {}
        }
    }

    fn refresh_entry_positions_from(&mut self, start: usize) {
        for (position, entry) in self.entries_cache.iter().enumerate().skip(start) {
            let slot = self
                .layout
                .resolve(&entry.key)
                .expect("cached blackboard key must belong to its layout");
            self.entry_positions[slot.generation_index() as usize] = Some(position as u32);
        }
    }

    fn refresh_entries(&mut self) {
        self.entry_positions.fill(None);
        let mut entries = std::mem::take(&mut self.entries_cache);
        entries.clear();
        entries.reserve(self.layout.key_count());
        for (key, slot) in self.layout.slots() {
            if let Some(value) = self.read(slot) {
                self.entry_positions[slot.generation_index() as usize] = Some(entries.len() as u32);
                entries.push(AiBlackboardEntry::new(key, value));
            }
        }
        self.entries_cache = entries;
    }

    // 同一槽位在通知被取走前只入队一次，保留首次变化的相对顺序。
    fn record_changes(&mut self, changed: &[BlackboardSlot]) {
        for slot in changed {
            let pending = &mut self.pending_change_flags[slot.generation_index() as usize];
            if !*pending {
                *pending = true;
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
        let slot = self.resolve_slot(key)?;
        self.validate_slot_value(key, slot, value)?;
        Ok(slot)
    }

    fn resolve_slot(&self, key: &str) -> Result<BlackboardSlot, BlackboardRuntimeError> {
        self.layout
            .resolve(key)
            .ok_or_else(|| BlackboardRuntimeError::UnknownKey {
                key: key.to_string(),
            })
    }

    fn validate_slot_value(
        &self,
        key: &str,
        slot: BlackboardSlot,
        value: &AiBlackboardValue,
    ) -> Result<(), BlackboardRuntimeError> {
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
        Ok(())
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

#[cfg(test)]
#[path = "store/tests/synchronize_scratch_tests.rs"]
mod synchronize_scratch_tests;

#[cfg(test)]
#[path = "store/tests/entry_position_tests.rs"]
mod entry_position_tests;

#[cfg(test)]
#[path = "tests/store_performance_tests.rs"]
mod performance_tests;
