use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::platform::{
    PreferenceDurabilityState, PreferenceKey, PreferenceMutationTerminal,
    PreferencePersistenceFailureProjection, PreferenceReadSnapshot, PreferenceStorageError,
    PreferenceStorageErrorKind, PreferenceStorageOperation,
};
use crate::core::runtime::BoundedKeyedIoTerminal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PreferenceOverlayLimits {
    pub max_entries: usize,
    pub max_retained_bytes: usize,
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
        retained_bytes: usize,
        operation: PreferenceStorageOperation,
    ) -> Result<PreferenceOverlayReservation, PreferenceStorageError> {
        let mut state = lock(&self.state);
        let entries = state
            .entries
            .len()
            .checked_add(state.reserved_entries)
            .and_then(|entries| entries.checked_add(1))
            .ok_or_else(|| capacity_error(operation, "overlay entry count overflow"))?;
        if entries > self.limits.max_entries {
            return Err(capacity_error(operation, "overlay entry capacity exceeded"));
        }
        let bytes = state
            .retained_bytes
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
        state.reserved_entries += 1;
        state.reserved_bytes += retained_bytes;
        drop(state);
        Ok(PreferenceOverlayReservation {
            overlay: self.clone(),
            generation,
            retained_bytes,
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
        result: Result<Option<Arc<[u8]>>, PreferencePersistenceFailureProjection>,
    ) {
        self.complete_if_generation(key, generation, |entry| match result {
            Ok(value) => {
                entry.value = value;
                entry.durability = PreferenceDurabilityState::Durable;
                entry.last_terminal = Some(PreferenceMutationTerminal::Durable);
            }
            Err(failure) => entry.record_failure(failure),
        });
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
    ) {
        self.complete_if_generation(key, generation, |entry| {
            if entry.last_terminal.is_some() {
                return;
            }
            if let Some(terminal) = map_lane_terminal(lane_terminal, None) {
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

    fn complete_if_generation(
        &self,
        key: &PreferenceKey,
        generation: u64,
        complete: impl FnOnce(&mut OverlayEntry),
    ) {
        let mut state = lock(&self.state);
        let Some(entry) = state.entries.get_mut(key) else {
            return;
        };
        if entry.generation == generation {
            complete(entry);
        }
    }
}

pub(super) struct PreferenceOverlayReservation {
    overlay: PreferenceOverlay,
    generation: u64,
    retained_bytes: usize,
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
        state.reserved_entries = state.reserved_entries.saturating_sub(1);
        state.reserved_bytes = state.reserved_bytes.saturating_sub(self.retained_bytes);
        if state
            .entries
            .get(&key)
            .is_some_and(|entry| entry.generation >= self.generation)
        {
            drop(state);
            self.installed = true;
            return;
        }
        if let Some(previous) = state.entries.remove(&key) {
            state.retained_bytes = state.retained_bytes.saturating_sub(previous.retained_bytes);
        }
        state.retained_bytes = state.retained_bytes.saturating_add(self.retained_bytes);
        state.entries.insert(
            key,
            OverlayEntry {
                generation: self.generation,
                value,
                durability,
                last_terminal: None,
                retained_bytes: self.retained_bytes,
            },
        );
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
        state.reserved_entries = state.reserved_entries.saturating_sub(1);
        state.reserved_bytes = state.reserved_bytes.saturating_sub(self.retained_bytes);
    }
}

#[derive(Default)]
struct OverlayState {
    next_generation: u64,
    retained_bytes: usize,
    reserved_entries: usize,
    reserved_bytes: usize,
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
