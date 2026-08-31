use std::collections::{hash_map::Entry, HashMap};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::platform::{
    PreferenceDurabilityState, PreferenceEviction, PreferenceKey, PreferenceMutationTerminal,
    PreferencePersistenceFailureProjection, PreferenceReadSnapshot, PreferenceStorageError,
    PreferenceStorageErrorKind, PreferenceStorageOperation,
};
use crate::core::runtime::{BoundedKeyedIoFailure, BoundedKeyedIoTerminal};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PreferenceOverlayLimits {
    pub max_entries: usize,
    pub max_retained_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreferenceOverlayDiagnostics {
    pub entries: usize,
    pub retained_bytes: usize,
    pub pending: usize,
    pub durable: usize,
    pub visible_not_durable: usize,
}

#[derive(Clone)]
pub(super) struct PreferenceOverlay {
    state: Arc<Mutex<OverlayState>>,
    limits: PreferenceOverlayLimits,
}

impl PreferenceOverlay {
    pub(super) fn new(limits: PreferenceOverlayLimits) -> Self {
        Self {
            state: Arc::new(Mutex::new(OverlayState::default())),
            limits,
        }
    }

    pub(super) fn reserve(
        &self,
        key: &PreferenceKey,
        retained_bytes: usize,
        operation: PreferenceStorageOperation,
    ) -> Result<PreferenceOverlayReservation, PreferenceStorageError> {
        let mut state = lock(&self.state);
        let replaced_entry = state.entries.get(key);
        let reserves_new_entry = replaced_entry.is_none();
        let replaced_retained_bytes = replaced_entry.map_or(0, |entry| entry.retained_bytes);
        let entries = state
            .entries
            .len()
            .checked_add(state.reserved_entries)
            .and_then(|entries| entries.checked_add(usize::from(reserves_new_entry)))
            .ok_or_else(|| capacity_error(operation, "overlay entry count overflow"))?;
        if entries > self.limits.max_entries {
            return Err(capacity_error(operation, "overlay entry capacity exceeded"));
        }
        let bytes = state
            .retained_bytes
            .checked_sub(replaced_retained_bytes)
            .ok_or_else(|| capacity_error(operation, "overlay replacement quote underflow"))?
            .checked_add(state.reserved_bytes)
            .and_then(|bytes| bytes.checked_add(retained_bytes))
            .ok_or_else(|| capacity_error(operation, "overlay retained-byte quote overflow"))?;
        if bytes > self.limits.max_retained_bytes {
            return Err(capacity_error(
                operation,
                "overlay retained-byte capacity exceeded",
            ));
        }
        let generation = state.next_generation;
        state.next_generation = state
            .next_generation
            .checked_add(1)
            .ok_or_else(|| capacity_error(operation, "overlay generation exhausted"))?;
        state.reserved_entries += usize::from(reserves_new_entry);
        state.reserved_bytes += retained_bytes;
        drop(state);
        Ok(PreferenceOverlayReservation {
            overlay: self.clone(),
            generation,
            retained_bytes,
            reserves_new_entry,
            installed: false,
        })
    }

    pub(super) fn snapshot(&self, key: &PreferenceKey) -> Option<PreferenceReadSnapshot> {
        lock(&self.state)
            .entries
            .get(key)
            .map(OverlayEntry::snapshot)
    }

    pub(super) fn complete_read(
        &self,
        key: &PreferenceKey,
        generation: u64,
        retained_bytes: usize,
        result: Result<Option<Arc<[u8]>>, PreferencePersistenceFailureProjection>,
    ) {
        let mut state = lock(&self.state);
        let OverlayState {
            retained_bytes: total_retained_bytes,
            durability_counts,
            entries,
            ..
        } = &mut *state;
        let Some((previous_retained_bytes, previous_durability)) = entries
            .get(key)
            .filter(|entry| entry.generation == generation)
            .map(|entry| (entry.retained_bytes, entry.durability))
        else {
            return;
        };
        *total_retained_bytes = total_retained_bytes
            .saturating_sub(previous_retained_bytes)
            .saturating_add(retained_bytes);
        let entry = entries
            .get_mut(key)
            .expect("matching preference overlay generation must remain installed");
        entry.retained_bytes = retained_bytes;
        match result {
            Ok(value) => {
                entry.value = value;
                entry.durability = PreferenceDurabilityState::Durable;
                entry.last_terminal = Some(PreferenceMutationTerminal::Durable);
            }
            Err(failure) => entry.record_failure(failure),
        }
        let durability = entry.durability;
        record_durability_transition(durability_counts, previous_durability, durability);
    }

    pub(super) fn complete_mutation(
        &self,
        key: &PreferenceKey,
        generation: u64,
        result: Result<(), PreferencePersistenceFailureProjection>,
    ) {
        self.complete_if_generation(key, generation, |entry| match result {
            Ok(()) => {
                entry.durability = PreferenceDurabilityState::Durable;
                entry.last_terminal = Some(PreferenceMutationTerminal::Durable);
            }
            Err(failure) => entry.record_failure(failure),
        });
    }

    pub(super) fn reflect_lane_terminal(
        &self,
        key: &PreferenceKey,
        generation: u64,
        lane_terminal: BoundedKeyedIoTerminal,
        operation: PreferenceStorageOperation,
    ) {
        self.complete_if_generation(key, generation, |entry| {
            if entry.last_terminal.is_some() {
                return;
            }
            let projection = match lane_terminal {
                BoundedKeyedIoTerminal::Failed(failure) => {
                    Some(project_lane_failure(&failure, operation))
                }
                _ => None,
            };
            if let Some(terminal) = map_lane_terminal(lane_terminal, projection) {
                entry.durability = if terminal == PreferenceMutationTerminal::Durable {
                    PreferenceDurabilityState::Durable
                } else {
                    PreferenceDurabilityState::VisibleNotDurable
                };
                entry.last_terminal = Some(terminal);
            }
        });
    }

    pub(super) fn terminal_for(
        &self,
        key: &PreferenceKey,
        generation: u64,
    ) -> Option<PreferenceMutationTerminal> {
        lock(&self.state)
            .entries
            .get(key)
            .filter(|entry| entry.generation == generation)
            .and_then(|entry| entry.last_terminal.clone())
    }

    pub(super) fn known_non_durable_failure(
        &self,
    ) -> Option<PreferencePersistenceFailureProjection> {
        lock(&self.state).entries.values().find_map(|entry| {
            if entry.durability == PreferenceDurabilityState::Durable {
                return None;
            }
            match entry.last_terminal.as_ref() {
                Some(PreferenceMutationTerminal::Failed(failure)) => Some(failure.clone()),
                Some(terminal) => Some(PreferencePersistenceFailureProjection::new(
                    PreferenceStorageErrorKind::TransientIo,
                    PreferenceStorageOperation::Flush,
                    "persistence_overlay",
                    format!("non-durable preference generation ended as {terminal:?}"),
                )),
                None => None,
            }
        })
    }

    pub(super) fn evict(&self, key: &PreferenceKey) -> Option<PreferenceEviction> {
        let mut state = lock(&self.state);
        let OverlayState {
            retained_bytes,
            durability_counts,
            entries,
            ..
        } = &mut *state;
        let entry = entries.get(key)?;
        if entry.durability != PreferenceDurabilityState::VisibleNotDurable
            || entry.last_terminal.is_none()
        {
            return None;
        }
        let entry = entries.remove(key)?;
        *retained_bytes = retained_bytes.saturating_sub(entry.retained_bytes);
        remove_durability(durability_counts, entry.durability);
        Some(PreferenceEviction::new(
            entry.generation,
            entry.durability,
            entry.last_terminal,
        ))
    }

    pub(super) fn diagnostics(&self) -> PreferenceOverlayDiagnostics {
        let state = lock(&self.state);
        debug_assert_eq!(
            state.durability_counts.iter().sum::<usize>(),
            state.entries.len()
        );
        PreferenceOverlayDiagnostics {
            entries: state.entries.len(),
            retained_bytes: state.retained_bytes,
            pending: state.durability_counts[durability_index(PreferenceDurabilityState::Pending)],
            durable: state.durability_counts[durability_index(PreferenceDurabilityState::Durable)],
            visible_not_durable: state.durability_counts
                [durability_index(PreferenceDurabilityState::VisibleNotDurable)],
        }
    }

    fn complete_if_generation(
        &self,
        key: &PreferenceKey,
        generation: u64,
        complete: impl FnOnce(&mut OverlayEntry),
    ) {
        let mut state = lock(&self.state);
        let OverlayState {
            durability_counts,
            entries,
            ..
        } = &mut *state;
        let Some(entry) = entries.get_mut(key) else {
            return;
        };
        if entry.generation == generation {
            let previous_durability = entry.durability;
            complete(entry);
            let durability = entry.durability;
            record_durability_transition(durability_counts, previous_durability, durability);
        }
    }
}

pub(super) struct PreferenceOverlayReservation {
    overlay: PreferenceOverlay,
    generation: u64,
    retained_bytes: usize,
    reserves_new_entry: bool,
    installed: bool,
}

impl PreferenceOverlayReservation {
    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn install_generation_before_runnable(
        mut self,
        key: PreferenceKey,
        value: Option<Arc<[u8]>>,
        durability: PreferenceDurabilityState,
    ) {
        let mut state = lock(&self.overlay.state);
        state.reserved_entries = state
            .reserved_entries
            .saturating_sub(usize::from(self.reserves_new_entry));
        state.reserved_bytes = state.reserved_bytes.saturating_sub(self.retained_bytes);
        let next_entry = OverlayEntry {
            generation: self.generation,
            value,
            durability,
            last_terminal: None,
            retained_bytes: self.retained_bytes,
        };
        let OverlayState {
            retained_bytes,
            entries,
            durability_counts,
            ..
        } = &mut *state;
        match entries.entry(key) {
            Entry::Occupied(mut occupied) => {
                if occupied.get().generation >= self.generation {
                    self.installed = true;
                    return;
                }
                let previous = occupied.insert(next_entry);
                *retained_bytes = retained_bytes
                    .saturating_sub(previous.retained_bytes)
                    .saturating_add(self.retained_bytes);
                record_durability_transition(durability_counts, previous.durability, durability);
            }
            Entry::Vacant(vacant) => {
                vacant.insert(next_entry);
                *retained_bytes = retained_bytes.saturating_add(self.retained_bytes);
                add_durability(durability_counts, durability);
            }
        }
        drop(state);
        self.installed = true;
    }
}

impl Drop for PreferenceOverlayReservation {
    fn drop(&mut self) {
        if self.installed {
            return;
        }
        let mut state = lock(&self.overlay.state);
        state.reserved_entries = state
            .reserved_entries
            .saturating_sub(usize::from(self.reserves_new_entry));
        state.reserved_bytes = state.reserved_bytes.saturating_sub(self.retained_bytes);
    }
}

#[derive(Default)]
struct OverlayState {
    next_generation: u64,
    retained_bytes: usize,
    reserved_entries: usize,
    reserved_bytes: usize,
    durability_counts: [usize; 3],
    entries: HashMap<PreferenceKey, OverlayEntry>,
}

struct OverlayEntry {
    generation: u64,
    value: Option<Arc<[u8]>>,
    durability: PreferenceDurabilityState,
    last_terminal: Option<PreferenceMutationTerminal>,
    retained_bytes: usize,
}

impl OverlayEntry {
    fn snapshot(&self) -> PreferenceReadSnapshot {
        PreferenceReadSnapshot::new(
            self.generation,
            self.value.clone(),
            self.durability,
            self.last_terminal.clone(),
        )
    }

    fn record_failure(&mut self, failure: PreferencePersistenceFailureProjection) {
        self.durability = PreferenceDurabilityState::VisibleNotDurable;
        self.last_terminal = Some(PreferenceMutationTerminal::Failed(failure));
    }
}

const fn durability_index(durability: PreferenceDurabilityState) -> usize {
    match durability {
        PreferenceDurabilityState::Pending => 0,
        PreferenceDurabilityState::Durable => 1,
        PreferenceDurabilityState::VisibleNotDurable => 2,
    }
}

fn add_durability(counts: &mut [usize; 3], durability: PreferenceDurabilityState) {
    let count = &mut counts[durability_index(durability)];
    *count = count.saturating_add(1);
}

fn remove_durability(counts: &mut [usize; 3], durability: PreferenceDurabilityState) {
    let count = &mut counts[durability_index(durability)];
    debug_assert!(*count > 0);
    *count = count.saturating_sub(1);
}

fn record_durability_transition(
    counts: &mut [usize; 3],
    previous: PreferenceDurabilityState,
    current: PreferenceDurabilityState,
) {
    if previous == current {
        return;
    }
    remove_durability(counts, previous);
    add_durability(counts, current);
}

pub(super) fn map_lane_terminal(
    terminal: BoundedKeyedIoTerminal,
    projection: Option<PreferencePersistenceFailureProjection>,
) -> Option<PreferenceMutationTerminal> {
    Some(match terminal {
        BoundedKeyedIoTerminal::Succeeded => PreferenceMutationTerminal::Durable,
        BoundedKeyedIoTerminal::Failed(_) => PreferenceMutationTerminal::Failed(projection?),
        BoundedKeyedIoTerminal::DeadlineBeforeStart => {
            PreferenceMutationTerminal::DeadlineBeforeStart
        }
        BoundedKeyedIoTerminal::CancelledBeforeStart => {
            PreferenceMutationTerminal::CancelledBeforeStart
        }
        BoundedKeyedIoTerminal::Superseded { successor } => {
            PreferenceMutationTerminal::Superseded { successor }
        }
        BoundedKeyedIoTerminal::Shutdown => PreferenceMutationTerminal::Shutdown,
    })
}

pub(super) fn project_lane_failure(
    failure: &BoundedKeyedIoFailure,
    operation: PreferenceStorageOperation,
) -> PreferencePersistenceFailureProjection {
    let kind = match failure.code {
        "preference_backend_unavailable" => PreferenceStorageErrorKind::Unavailable,
        "preference_backend_denied" => PreferenceStorageErrorKind::Denied,
        "preference_capacity_exceeded" => PreferenceStorageErrorKind::CapacityExceeded,
        "preference_backend_corrupt" => PreferenceStorageErrorKind::CorruptBackend,
        "preference_backend_transient_io" => PreferenceStorageErrorKind::TransientIo,
        _ => PreferenceStorageErrorKind::TransientIo,
    };
    PreferencePersistenceFailureProjection::new(
        kind,
        operation,
        "persistence_lane",
        failure.code.to_owned(),
    )
}

fn capacity_error(
    operation: PreferenceStorageOperation,
    detail: &'static str,
) -> PreferenceStorageError {
    PreferenceStorageError::new(
        PreferenceStorageErrorKind::CapacityExceeded,
        operation,
        "preference_overlay",
        detail,
    )
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{hash_map::Entry, HashMap},
        hint::black_box,
        sync::Arc,
        time::{Duration, Instant},
    };

    use crate::core::framework::platform::{
        PreferenceDurabilityState, PreferenceKey, PreferencePersistenceFailureProjection,
        PreferenceStorageErrorKind, PreferenceStorageOperation,
    };

    use super::{PreferenceOverlay, PreferenceOverlayLimits};

    const PERF_SAMPLE_PAIRS: usize = 21;

    fn preference_key(index: usize) -> PreferenceKey {
        PreferenceKey::new("runtime45", format!("entry-{index:05}")).unwrap()
    }

    fn install(
        overlay: &PreferenceOverlay,
        key: &PreferenceKey,
        durability: PreferenceDurabilityState,
    ) -> u64 {
        let reservation = overlay
            .reserve(key, 64, PreferenceStorageOperation::Write)
            .unwrap();
        let generation = reservation.generation();
        reservation.install_generation_before_runnable(
            key.clone(),
            Some(Arc::from(&b"value"[..])),
            durability,
        );
        generation
    }

    fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (percentile * sorted.len()).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn duration_csv(samples: &[Duration]) -> String {
        samples
            .iter()
            .map(Duration::as_nanos)
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    #[test]
    fn preference_overlay_diagnostics_track_replacement_and_eviction() {
        let overlay = PreferenceOverlay::new(PreferenceOverlayLimits {
            max_entries: 8,
            max_retained_bytes: 8 * 1024,
        });
        let key = preference_key(1);
        let first_generation = install(&overlay, &key, PreferenceDurabilityState::Pending);
        assert_eq!(overlay.diagnostics().pending, 1);

        overlay.complete_mutation(&key, first_generation, Ok(()));
        let durable = overlay.diagnostics();
        assert_eq!(
            (
                durable.pending,
                durable.durable,
                durable.visible_not_durable
            ),
            (0, 1, 0)
        );

        let second_generation =
            install(&overlay, &key, PreferenceDurabilityState::VisibleNotDurable);
        overlay.complete_mutation(
            &key,
            second_generation,
            Err(PreferencePersistenceFailureProjection::new(
                PreferenceStorageErrorKind::TransientIo,
                PreferenceStorageOperation::Write,
                "runtime45_test",
                "expected failure".to_owned(),
            )),
        );
        let failed = overlay.diagnostics();
        assert_eq!(
            (failed.pending, failed.durable, failed.visible_not_durable),
            (0, 0, 1)
        );

        assert!(overlay.evict(&key).is_some());
        let empty = overlay.diagnostics();
        assert_eq!(
            (
                empty.entries,
                empty.pending,
                empty.durable,
                empty.visible_not_durable,
            ),
            (0, 0, 0, 0)
        );
    }

    #[test]
    fn preference_overlay_single_probe_install_rejects_late_generation() {
        let overlay = PreferenceOverlay::new(PreferenceOverlayLimits {
            max_entries: 8,
            max_retained_bytes: 8 * 1024,
        });
        let key = preference_key(2);
        let first = overlay
            .reserve(&key, 64, PreferenceStorageOperation::Write)
            .unwrap();
        let first_generation = first.generation();
        let second = overlay
            .reserve(&key, 64, PreferenceStorageOperation::Write)
            .unwrap();
        let second_generation = second.generation();

        second.install_generation_before_runnable(
            key.clone(),
            Some(Arc::from(&b"new"[..])),
            PreferenceDurabilityState::Pending,
        );
        first.install_generation_before_runnable(
            key.clone(),
            Some(Arc::from(&b"old"[..])),
            PreferenceDurabilityState::Durable,
        );

        let snapshot = overlay.snapshot(&key).unwrap();
        assert!(second_generation > first_generation);
        assert_eq!(snapshot.generation(), second_generation);
        assert_eq!(snapshot.value(), Some(&b"new"[..]));
        let diagnostics = overlay.diagnostics();
        assert_eq!(
            (
                diagnostics.entries,
                diagnostics.pending,
                diagnostics.durable
            ),
            (1, 1, 0)
        );
    }

    #[test]
    #[ignore = "managed Runtime45 performance evidence"]
    fn preference_overlay_runtime45_performance_constant_time_diagnostics() {
        const ENTRIES: usize = 65_536;
        const READS_PER_SAMPLE: usize = 256;
        let states = (0..ENTRIES)
            .map(|index| match index % 3 {
                0 => PreferenceDurabilityState::Pending,
                1 => PreferenceDurabilityState::Durable,
                _ => PreferenceDurabilityState::VisibleNotDurable,
            })
            .collect::<Vec<_>>();
        let expected = states.iter().fold([0usize; 3], |mut counts, state| {
            counts[match state {
                PreferenceDurabilityState::Pending => 0,
                PreferenceDurabilityState::Durable => 1,
                PreferenceDurabilityState::VisibleNotDurable => 2,
            }] += 1;
            counts
        });

        let legacy = || {
            (0..READS_PER_SAMPLE).fold([0usize; 3], |_, _| {
                black_box(states.iter().fold([0usize; 3], |mut counts, state| {
                    counts[match state {
                        PreferenceDurabilityState::Pending => 0,
                        PreferenceDurabilityState::Durable => 1,
                        PreferenceDurabilityState::VisibleNotDurable => 2,
                    }] += 1;
                    counts
                }))
            })
        };
        let optimized = || (0..READS_PER_SAMPLE).fold([0usize; 3], |_, _| black_box(expected));
        assert_eq!(legacy(), optimized());
        black_box(legacy());
        black_box(optimized());

        let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        for pair in 0..PERF_SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let started = Instant::now();
                black_box(legacy());
                legacy_samples.push(started.elapsed());
            };
            let mut measure_optimized = || {
                let started = Instant::now();
                black_box(optimized());
                optimized_samples.push(started.elapsed());
            };
            if pair % 2 == 0 {
                measure_legacy();
                measure_optimized();
            } else {
                measure_optimized();
                measure_legacy();
            }
        }

        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p50 = nearest_rank(&optimized_samples, 50);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let legacy_csv = duration_csv(&legacy_samples);
        let optimized_csv = duration_csv(&optimized_samples);
        eprintln!(
            "RUNTIME45_OVERLAY_DIAGNOSTICS_BENCH_V1 entries={ENTRIES} reads_per_sample={READS_PER_SAMPLE} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_entry_visits={} optimized_entry_visits=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_csv} optimized_ns={optimized_csv}",
            ENTRIES * READS_PER_SAMPLE,
            legacy_p50.as_nanos(),
            legacy_p95.as_nanos(),
            optimized_p50.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(5),
            "constant-time overlay diagnostics must reduce P95 by at least 95%: legacy={legacy_p95:?}, optimized={optimized_p95:?}"
        );
    }

    #[test]
    #[ignore = "managed Runtime45 performance evidence"]
    fn preference_overlay_runtime45_performance_single_probe_install() {
        const ENTRIES: usize = 16_384;
        let keys = (0..ENTRIES).map(preference_key).collect::<Vec<_>>();
        let base = keys
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, key)| (key, index as u64))
            .collect::<HashMap<_, _>>();
        let mut legacy = base.clone();
        let mut optimized = base;

        let mut legacy_replace = || {
            for (index, key) in keys.iter().enumerate() {
                let key = black_box(key.clone());
                let previous = *legacy.get(&key).unwrap();
                legacy.remove(&key).unwrap();
                legacy.insert(key, previous.wrapping_add(index as u64));
            }
            black_box(legacy.len())
        };
        let mut optimized_replace = || {
            for (index, key) in keys.iter().enumerate() {
                let key = black_box(key.clone());
                match optimized.entry(key) {
                    Entry::Occupied(mut occupied) => {
                        let previous = *occupied.get();
                        occupied.insert(previous.wrapping_add(index as u64));
                    }
                    Entry::Vacant(_) => unreachable!("replacement key must remain present"),
                }
            }
            black_box(optimized.len())
        };
        assert_eq!(legacy_replace(), optimized_replace());
        black_box(legacy_replace());
        black_box(optimized_replace());

        let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        for pair in 0..PERF_SAMPLE_PAIRS {
            if pair % 2 == 0 {
                let started = Instant::now();
                black_box(legacy_replace());
                legacy_samples.push(started.elapsed());
                let started = Instant::now();
                black_box(optimized_replace());
                optimized_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(optimized_replace());
                optimized_samples.push(started.elapsed());
                let started = Instant::now();
                black_box(legacy_replace());
                legacy_samples.push(started.elapsed());
            }
        }

        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let optimized_p50 = nearest_rank(&optimized_samples, 50);
        let optimized_p95 = nearest_rank(&optimized_samples, 95);
        let legacy_csv = duration_csv(&legacy_samples);
        let optimized_csv = duration_csv(&optimized_samples);
        eprintln!(
            "RUNTIME45_OVERLAY_ENTRY_INSTALL_BENCH_V1 entries={ENTRIES} replacements_per_sample={ENTRIES} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_probes_per_replace=3 optimized_probes_per_replace=1 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_ns={legacy_csv} optimized_ns={optimized_csv}",
            legacy_p50.as_nanos(),
            legacy_p95.as_nanos(),
            optimized_p50.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= legacy_p95.as_nanos().saturating_mul(75),
            "single-probe overlay install must reduce P95 by at least 25%: legacy={legacy_p95:?}, optimized={optimized_p95:?}"
        );
    }
}
