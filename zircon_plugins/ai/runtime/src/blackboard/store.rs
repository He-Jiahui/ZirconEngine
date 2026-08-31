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
#[path = "store/synchronize_scratch_tests.rs"]
mod synchronize_scratch_tests;

#[cfg(test)]
#[path = "store/entry_position_tests.rs"]
mod entry_position_tests;

#[cfg(test)]
mod performance_tests {
    use std::collections::HashSet;
    use std::hint::black_box;
    use std::sync::Arc;
    use std::time::Instant;

    use zircon_runtime::core::framework::ai::{
        AiBlackboardEntry, AiBlackboardSchemaDescriptor, AiBlackboardValue,
    };

    use super::{BlackboardLayout, BlackboardRuntimeError, BlackboardSlot, BlackboardStore};

    const BENCHMARK_KEY_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLE_COUNT: usize = 21;
    const BENCHMARK_WRITE_COUNT: usize = 64;

    #[test]
    fn pending_change_dedup_preserves_first_change_order_and_resets_after_drain() {
        let (layout, _, slots) = benchmark_fixture(3);
        let mut store = BlackboardStore::new(layout);

        store.record_changes(&[slots[2], slots[0], slots[2], slots[1], slots[0]]);
        assert_eq!(store.drain_changed_slots(), [slots[2], slots[0], slots[1]]);

        store.record_changes(&[slots[0], slots[2]]);
        assert_eq!(store.drain_changed_slots(), [slots[0], slots[2]]);
    }

    #[test]
    fn pending_change_tracking_uses_dense_slot_flags_instead_of_linear_scan() {
        let source = include_str!("store.rs");
        let fields = source
            .split("pub struct BlackboardStore {")
            .nth(1)
            .and_then(|body| body.split("impl BlackboardStore").next())
            .expect("BlackboardStore fields");
        let record_changes = source
            .split("fn record_changes(&mut self")
            .nth(1)
            .and_then(|body| body.split("fn clear(").next())
            .expect("record_changes body");

        assert!(fields.contains("pending_change_flags: Box<[bool]>"));
        assert!(!record_changes.contains("pending_changes.contains"));
    }

    #[test]
    fn synchronize_keeps_duplicate_error_precedence_and_atomicity() {
        let (layout, _, _) = benchmark_fixture(2);
        let mut store = BlackboardStore::new(layout);
        store
            .write("key_0000", AiBlackboardValue::Integer(7))
            .expect("initial write");
        store.drain_changed_slots();
        let entries_before = store.entries();

        let error = store
            .synchronize(&[
                AiBlackboardEntry::new("key_0001", AiBlackboardValue::Integer(1)),
                AiBlackboardEntry::new("key_0001", AiBlackboardValue::Bool(true)),
            ])
            .expect_err("duplicate key must win before validating its second value");

        assert_eq!(
            error,
            BlackboardRuntimeError::DuplicateKey {
                key: "key_0001".to_string(),
            }
        );
        assert_eq!(store.entries(), entries_before);
        assert!(store.drain_changed_slots().is_empty());
    }

    #[test]
    fn synchronize_reuses_prevalidated_slots_without_a_hash_dedup_set() {
        let source = include_str!("store.rs");
        let synchronize = source
            .split("pub fn synchronize(")
            .nth(1)
            .and_then(|body| body.split("pub(crate) fn drain_changed_slots").next())
            .expect("synchronize body");

        assert!(synchronize.contains("validated_slots"));
        assert!(!synchronize.contains("HashSet"));
        assert!(!synchronize.contains("write_untracked"));
        assert!(!synchronize.contains("collect::<Vec<_>>()"));
        assert!(synchronize.contains("let mut changed = Vec::new();"));
    }

    #[test]
    fn single_key_writes_keep_the_entry_cache_sorted_and_current() {
        let (layout, _, _) = benchmark_fixture(3);
        let mut store = BlackboardStore::new(layout);

        store
            .write("key_0002", AiBlackboardValue::Integer(2))
            .expect("insert last key");
        store
            .write("key_0000", AiBlackboardValue::Integer(0))
            .expect("insert first key");
        store
            .write("key_0002", AiBlackboardValue::Integer(22))
            .expect("update existing key");
        store
            .write("key_0001", AiBlackboardValue::Integer(1))
            .expect("insert middle key");

        assert_eq!(
            store.entries(),
            [
                AiBlackboardEntry::new("key_0000", AiBlackboardValue::Integer(0)),
                AiBlackboardEntry::new("key_0001", AiBlackboardValue::Integer(1)),
                AiBlackboardEntry::new("key_0002", AiBlackboardValue::Integer(22)),
            ]
        );
    }

    #[test]
    fn single_key_write_refreshes_only_its_cached_entry() {
        let source = include_str!("store.rs");
        let write = source
            .split("pub fn write(")
            .nth(1)
            .and_then(|body| body.split("fn write_untracked").next())
            .expect("write body");
        let refresh_entry = source
            .split("fn refresh_entry(")
            .nth(1)
            .and_then(|body| body.split("fn refresh_entries").next())
            .expect("refresh_entry body");

        assert!(write.contains("self.refresh_entry(outcome.slot)"));
        assert!(!write.contains("self.refresh_entries()"));
        assert!(refresh_entry.contains("binary_search_by"));
        assert!(!refresh_entry.contains("self.layout.slots()"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn indexed_pending_blackboard_changes_release_benchmark_evidence() {
        let (layout, _, slots) = benchmark_fixture(BENCHMARK_KEY_COUNT);
        let mut optimized_store = BlackboardStore::new(layout);
        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || legacy_record_changes(black_box(&slots)),
            || {
                optimized_store.record_changes(black_box(&slots));
                black_box(optimized_store.drain_changed_slots().len())
            },
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);
        let legacy_comparisons = BENCHMARK_KEY_COUNT * (BENCHMARK_KEY_COUNT - 1) / 2;

        println!(
            "PERF_RESULT plugins15_indexed_pending_blackboard_changes slots={BENCHMARK_KEY_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_slot_comparisons_per_sample={legacy_comparisons} optimized_flag_lookups_per_sample={BENCHMARK_KEY_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 2 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 50% of legacy P95 {legacy_p95}ns"
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn prevalidated_blackboard_synchronize_release_benchmark_evidence() {
        let (layout, entries, _) = benchmark_fixture(BENCHMARK_KEY_COUNT);
        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || benchmark_legacy_synchronize(&layout, black_box(&entries)),
            || benchmark_optimized_synchronize(&layout, black_box(&entries)),
        );
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);

        println!(
            "PERF_RESULT plugins15_prevalidated_blackboard_synchronize entries={BENCHMARK_KEY_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_layout_lookups_per_sample={} optimized_layout_lookups_per_sample={BENCHMARK_KEY_COUNT} legacy_hash_inserts_per_sample={BENCHMARK_KEY_COUNT} optimized_hash_inserts_per_sample=0 legacy_clear_slot_buffer_allocations_per_sample=1 optimized_clear_slot_buffer_allocations_per_sample=0 legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}",
            BENCHMARK_KEY_COUNT * 2
        );
        assert!(
            optimized_p95 * 20 <= legacy_p95 * 17,
            "optimized P95 {optimized_p95}ns must be no more than 85% of legacy P95 {legacy_p95}ns"
        );
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn incremental_blackboard_entry_refresh_release_benchmark_evidence() {
        let (layout, entries, _) = benchmark_fixture(BENCHMARK_KEY_COUNT);
        let key = entries[BENCHMARK_KEY_COUNT / 2].key.clone();
        let mut legacy_store = BlackboardStore::new(layout.clone());
        legacy_store
            .synchronize(&entries)
            .expect("populate legacy store");
        legacy_store.drain_changed_slots();
        let mut optimized_store = BlackboardStore::new(layout);
        optimized_store
            .synchronize(&entries)
            .expect("populate optimized store");
        optimized_store.drain_changed_slots();
        let mut legacy_value = BENCHMARK_KEY_COUNT as i64;
        let mut optimized_value = BENCHMARK_KEY_COUNT as i64;

        let (legacy_samples, optimized_samples) = benchmark_paired_samples(
            || benchmark_legacy_single_key_writes(&mut legacy_store, &key, &mut legacy_value),
            || {
                benchmark_optimized_single_key_writes(
                    &mut optimized_store,
                    &key,
                    &mut optimized_value,
                )
            },
        );
        assert_eq!(legacy_store.entries(), optimized_store.entries());
        let legacy_p50 = percentile(&legacy_samples, 50);
        let legacy_p95 = percentile(&legacy_samples, 95);
        let optimized_p50 = percentile(&optimized_samples, 50);
        let optimized_p95 = percentile(&optimized_samples, 95);
        let legacy_ns = benchmark_samples_csv(&legacy_samples);
        let optimized_ns = benchmark_samples_csv(&optimized_samples);
        let legacy_entry_visits = BENCHMARK_KEY_COUNT * BENCHMARK_WRITE_COUNT;

        println!(
            "PERF_RESULT plugins15_incremental_blackboard_entry_refresh keys={BENCHMARK_KEY_COUNT} writes_per_sample={BENCHMARK_WRITE_COUNT} samples={BENCHMARK_SAMPLE_COUNT} sample_pairs={BENCHMARK_SAMPLE_COUNT} sample_order=alternating percentile_method=nearest_rank legacy_entry_visits_per_sample={legacy_entry_visits} optimized_full_entry_scans_per_sample=0 optimized_binary_searches_per_sample={BENCHMARK_WRITE_COUNT} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} legacy_ns={legacy_ns} optimized_ns={optimized_ns}"
        );
        assert!(
            optimized_p95 * 5 <= legacy_p95,
            "optimized P95 {optimized_p95}ns must be no more than 20% of legacy P95 {legacy_p95}ns"
        );
    }

    fn benchmark_fixture(
        key_count: usize,
    ) -> (
        Arc<BlackboardLayout>,
        Vec<AiBlackboardEntry>,
        Vec<BlackboardSlot>,
    ) {
        let mut descriptor = AiBlackboardSchemaDescriptor::new("benchmark", "Benchmark");
        let mut entries = Vec::with_capacity(key_count);
        for index in 0..key_count {
            let key = format!("key_{index:04}");
            descriptor = descriptor.with_key(key.clone(), "integer", false);
            entries.push(AiBlackboardEntry::new(
                key,
                AiBlackboardValue::Integer(index as i64),
            ));
        }
        let layout = Arc::new(BlackboardLayout::from_schema(&descriptor).expect("valid layout"));
        let slots = entries
            .iter()
            .map(|entry| layout.resolve(&entry.key).expect("compiled slot"))
            .collect();
        (layout, entries, slots)
    }

    fn legacy_record_changes(slots: &[BlackboardSlot]) -> usize {
        let mut pending = Vec::with_capacity(slots.len());
        let mut comparisons = 0_usize;
        for slot in slots {
            let duplicate = pending.iter().any(|existing| {
                comparisons += 1;
                existing == slot
            });
            if !duplicate {
                pending.push(*slot);
            }
        }
        black_box(comparisons);
        black_box(pending.len())
    }

    fn benchmark_legacy_synchronize(
        layout: &Arc<BlackboardLayout>,
        entries: &[AiBlackboardEntry],
    ) -> usize {
        let mut store = BlackboardStore::new(layout.clone());
        let changed = legacy_synchronize(&mut store, entries).expect("legacy synchronize");
        black_box(store.entries_ref());
        black_box(&changed);
        black_box(changed.len())
    }

    fn legacy_synchronize(
        store: &mut BlackboardStore,
        entries: &[AiBlackboardEntry],
    ) -> Result<Vec<BlackboardSlot>, BlackboardRuntimeError> {
        let mut seen = HashSet::with_capacity(entries.len());
        let mut present = vec![false; store.layout.key_count()];
        for entry in entries {
            if !seen.insert(entry.key.as_str()) {
                return Err(BlackboardRuntimeError::DuplicateKey {
                    key: entry.key.clone(),
                });
            }
            let slot = store.validate_write(&entry.key, &entry.value)?;
            present[slot.generation_index() as usize] = true;
        }
        let mut changed = Vec::new();
        for entry in entries {
            let outcome = store.write_untracked(&entry.key, entry.value.clone())?;
            if outcome.changed {
                changed.push(outcome.slot);
            }
        }
        let slots = store
            .layout
            .slots()
            .map(|(_, slot)| slot)
            .collect::<Vec<_>>();
        for slot in slots {
            if !present[slot.generation_index() as usize] && store.clear(slot) {
                let generation = &mut store.generations[slot.generation_index() as usize];
                *generation = generation.wrapping_add(1);
                changed.push(slot);
            }
        }
        if !changed.is_empty() {
            store.refresh_entries();
            store.record_changes(&changed);
        }
        Ok(changed)
    }

    fn benchmark_optimized_synchronize(
        layout: &Arc<BlackboardLayout>,
        entries: &[AiBlackboardEntry],
    ) -> usize {
        let mut store = BlackboardStore::new(layout.clone());
        let changed = store.synchronize(entries).expect("optimized synchronize");
        black_box(store.entries_ref());
        black_box(&changed);
        black_box(changed.len())
    }

    fn benchmark_legacy_single_key_writes(
        store: &mut BlackboardStore,
        key: &str,
        next_value: &mut i64,
    ) -> usize {
        for _ in 0..BENCHMARK_WRITE_COUNT {
            *next_value = next_value.wrapping_add(1);
            let outcome = store
                .write_untracked(
                    black_box(key),
                    AiBlackboardValue::Integer(black_box(*next_value)),
                )
                .expect("legacy write");
            if outcome.changed {
                store.refresh_entries();
                store.record_changes(std::slice::from_ref(&outcome.slot));
            }
        }
        black_box(store.entries_ref());
        black_box(store.drain_changed_slots().len())
    }

    fn benchmark_optimized_single_key_writes(
        store: &mut BlackboardStore,
        key: &str,
        next_value: &mut i64,
    ) -> usize {
        for _ in 0..BENCHMARK_WRITE_COUNT {
            *next_value = next_value.wrapping_add(1);
            black_box(
                store
                    .write(
                        black_box(key),
                        AiBlackboardValue::Integer(black_box(*next_value)),
                    )
                    .expect("optimized write"),
            );
        }
        black_box(store.entries_ref());
        black_box(store.drain_changed_slots().len())
    }

    fn benchmark_paired_samples(
        mut legacy: impl FnMut() -> usize,
        mut optimized: impl FnMut() -> usize,
    ) -> (Vec<u128>, Vec<u128>) {
        black_box(legacy());
        black_box(optimized());
        let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLE_COUNT);
        for sample_index in 0..BENCHMARK_SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                legacy_samples.push(benchmark_sample(&mut legacy));
                optimized_samples.push(benchmark_sample(&mut optimized));
            } else {
                optimized_samples.push(benchmark_sample(&mut optimized));
                legacy_samples.push(benchmark_sample(&mut legacy));
            }
        }
        (legacy_samples, optimized_samples)
    }

    fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
        let started = Instant::now();
        black_box(operation());
        started.elapsed().as_nanos()
    }

    fn benchmark_samples_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        assert!(!sorted.is_empty());
        assert!((1..=100).contains(&percentile));
        let index = (sorted.len() * percentile).div_ceil(100) - 1;
        sorted[index]
    }
}
