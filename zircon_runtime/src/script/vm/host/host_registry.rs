use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use super::super::HostHandle;

const INITIAL_HOST_HANDLE_GENERATION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCapabilityRecord {
    pub handle: HostHandle,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostRegistryError {
    SlotIndexExhausted,
    MissingIndex {
        index: u32,
    },
    VacantSlot {
        index: u32,
        generation: u32,
    },
    GenerationMismatch {
        index: u32,
        expected: u32,
        actual: u32,
    },
    GenerationExhausted {
        index: u32,
        generation: u32,
    },
    FreeSlotOccupied {
        index: u32,
    },
}

impl fmt::Display for HostRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SlotIndexExhausted => formatter.write_str("host handle slot index exhausted"),
            Self::MissingIndex { index } => {
                write!(
                    formatter,
                    "host handle references missing slot index {index}"
                )
            }
            Self::VacantSlot { index, generation } => write!(
                formatter,
                "host handle references vacant slot {index} at generation {generation}"
            ),
            Self::GenerationMismatch {
                index,
                expected,
                actual,
            } => write!(
                formatter,
                "host handle generation mismatch for slot {index}: expected {expected}, received {actual}"
            ),
            Self::GenerationExhausted { index, generation } => write!(
                formatter,
                "host handle generation exhausted for slot {index} at generation {generation}"
            ),
            Self::FreeSlotOccupied { index } => {
                write!(
                    formatter,
                    "host handle free-list slot {index} is still occupied"
                )
            }
        }
    }
}

impl std::error::Error for HostRegistryError {}

#[derive(Clone, Debug, Default)]
pub struct HostRegistry {
    state: Arc<Mutex<HostRegistryState>>,
}

#[derive(Debug, Default)]
struct HostRegistryState {
    slots: Vec<HostRegistrySlot>,
    free_slots: Vec<u32>,
}

#[derive(Debug)]
struct HostRegistrySlot {
    generation: u32,
    record: Option<HostCapabilityRecord>,
}

impl HostRegistry {
    fn lock_state(&self) -> MutexGuard<'_, HostRegistryState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn register_capability(
        &self,
        label: impl Into<String>,
    ) -> Result<HostHandle, HostRegistryError> {
        let label = label.into();
        let mut state = self.lock_state();
        let (index, reused) = if let Some(index) = state.free_slots.pop() {
            (index, true)
        } else {
            let index = u32::try_from(state.slots.len())
                .map_err(|_| HostRegistryError::SlotIndexExhausted)?;
            state.slots.push(HostRegistrySlot {
                generation: INITIAL_HOST_HANDLE_GENERATION,
                record: None,
            });
            (index, false)
        };
        if state
            .slots
            .get(index as usize)
            .ok_or(HostRegistryError::MissingIndex { index })?
            .record
            .is_some()
        {
            if reused {
                state.free_slots.push(index);
            }
            return Err(HostRegistryError::FreeSlotOccupied { index });
        }
        let slot = state
            .slots
            .get_mut(index as usize)
            .ok_or(HostRegistryError::MissingIndex { index })?;
        let handle = HostHandle::from_parts(index, slot.generation);
        slot.record = Some(HostCapabilityRecord { handle, label });
        Ok(handle)
    }

    pub fn resolve(&self, handle: HostHandle) -> Result<HostCapabilityRecord, HostRegistryError> {
        let state = self.lock_state();
        let slot =
            state
                .slots
                .get(handle.index() as usize)
                .ok_or(HostRegistryError::MissingIndex {
                    index: handle.index(),
                })?;
        validate_generation(slot, handle)?;
        slot.record.clone().ok_or(HostRegistryError::VacantSlot {
            index: handle.index(),
            generation: handle.generation(),
        })
    }

    pub fn revoke(&self, handle: HostHandle) -> Result<HostCapabilityRecord, HostRegistryError> {
        let mut state = self.lock_state();
        let slot = state.slots.get_mut(handle.index() as usize).ok_or(
            HostRegistryError::MissingIndex {
                index: handle.index(),
            },
        )?;
        validate_generation(slot, handle)?;
        if slot.record.is_none() {
            return Err(HostRegistryError::VacantSlot {
                index: handle.index(),
                generation: handle.generation(),
            });
        }
        let next_generation =
            slot.generation
                .checked_add(1)
                .ok_or(HostRegistryError::GenerationExhausted {
                    index: handle.index(),
                    generation: handle.generation(),
                })?;
        let record = slot.record.take().ok_or(HostRegistryError::VacantSlot {
            index: handle.index(),
            generation: handle.generation(),
        })?;
        slot.generation = next_generation;
        state.free_slots.push(handle.index());
        Ok(record)
    }

    pub fn capabilities(&self) -> Vec<HostCapabilityRecord> {
        let mut records = self
            .lock_state()
            .slots
            .iter()
            .filter_map(|slot| slot.record.clone())
            .collect::<Vec<_>>();
        records.sort_by_key(|record| record.handle.into_raw());
        records
    }

    pub fn is_valid(&self, handle: HostHandle) -> bool {
        self.resolve(handle).is_ok()
    }
}

fn validate_generation(
    slot: &HostRegistrySlot,
    handle: HostHandle,
) -> Result<(), HostRegistryError> {
    if slot.generation != handle.generation() {
        return Err(HostRegistryError::GenerationMismatch {
            index: handle.index(),
            expected: slot.generation,
            actual: handle.generation(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    struct PanickingLabel;

    impl From<PanickingLabel> for String {
        fn from(_: PanickingLabel) -> Self {
            panic!("intentional label conversion panic");
        }
    }

    #[test]
    fn dead_host_object_access_returns_error_not_ub() {
        let registry = HostRegistry::default();
        let handle = registry.register_capability("test.capability").unwrap();
        registry.revoke(handle).unwrap();

        assert!(matches!(
            registry.resolve(handle),
            Err(HostRegistryError::GenerationMismatch { index, .. })
                if index == handle.index()
        ));
        assert!(!registry.is_valid(handle));
    }

    #[test]
    fn stale_handle_remains_invalid_after_slot_reuse() {
        let registry = HostRegistry::default();
        let stale = registry.register_capability("first").unwrap();
        registry.revoke(stale).unwrap();
        let current = registry.register_capability("second").unwrap();

        assert_eq!(current.index(), stale.index());
        assert_eq!(current.generation(), stale.generation() + 1);
        assert!(matches!(
            registry.resolve(stale),
            Err(HostRegistryError::GenerationMismatch { .. })
        ));
        assert_eq!(registry.resolve(current).unwrap().label, "second");
    }

    #[test]
    fn forged_current_generation_for_vacant_slot_is_rejected_as_vacant() {
        let registry = HostRegistry::default();
        let handle = registry.register_capability("first").unwrap();
        registry.revoke(handle).unwrap();
        let vacant = HostHandle::from_parts(handle.index(), handle.generation() + 1);

        assert!(matches!(
            registry.resolve(vacant),
            Err(HostRegistryError::VacantSlot { index, .. }) if index == handle.index()
        ));
    }

    #[test]
    fn generation_exhaustion_keeps_live_record_valid() {
        let registry = HostRegistry::default();
        let handle = registry.register_capability("last-generation").unwrap();
        {
            let mut state = registry.lock_state();
            state.slots[handle.index() as usize].generation = u32::MAX;
            state.slots[handle.index() as usize]
                .record
                .as_mut()
                .unwrap()
                .handle = HostHandle::from_parts(handle.index(), u32::MAX);
        }
        let exhausted = HostHandle::from_parts(handle.index(), u32::MAX);

        assert!(matches!(
            registry.revoke(exhausted),
            Err(HostRegistryError::GenerationExhausted { .. })
        ));
        assert!(registry.is_valid(exhausted));
    }

    #[test]
    fn host_registry_accessors_recover_poisoned_handle_lock() {
        let registry = HostRegistry::default();

        let poison_result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = registry.state.lock().unwrap();
            panic!("poison host handle registry");
        }));
        assert!(poison_result.is_err());

        let handle = registry.register_capability("test.capability").unwrap();
        assert!(registry.is_valid(handle));
        assert_eq!(
            registry.resolve(handle).unwrap(),
            HostCapabilityRecord {
                handle,
                label: "test.capability".to_string(),
            }
        );
        assert_eq!(registry.capabilities().len(), 1);
    }

    #[test]
    fn panicking_label_conversion_does_not_consume_reusable_slot() {
        let registry = HostRegistry::default();
        let first = registry.register_capability("first").unwrap();
        registry.revoke(first).unwrap();

        let panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = registry.register_capability(PanickingLabel);
        }));
        assert!(panic.is_err());

        let reused = registry.register_capability("reused").unwrap();
        assert_eq!(reused.index(), first.index());
        assert_eq!(reused.generation(), first.generation() + 1);
        assert_eq!(registry.capabilities().len(), 1);
    }
}
