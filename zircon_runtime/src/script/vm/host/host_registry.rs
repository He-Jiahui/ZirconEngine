//! 宿主能力句柄由槽位和代际共同确定；撤销后即使槽位复用，旧脚本句柄也必须保持失效。
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

    /// 撤销使所有旧代际副本失效；槽位可回收，但已耗尽的代际不能绕回旧身份。
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
        let mut records = {
            let state = self.lock_state();
            let live_record_capacity = state.slots.len().saturating_sub(state.free_slots.len());
            let mut records = Vec::with_capacity(live_record_capacity);
            records.extend(state.slots.iter().filter_map(|slot| slot.record.clone()));
            records
        };
        records.sort_unstable_by_key(|record| record.handle.into_raw());
        records
    }

    pub fn is_valid(&self, handle: HostHandle) -> bool {
        let state = self.lock_state();
        let Some(slot) = state.slots.get(handle.index() as usize) else {
            return false;
        };
        slot.generation == handle.generation() && slot.record.is_some()
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
#[path = "tests/host_registry.rs"]
mod tests;
