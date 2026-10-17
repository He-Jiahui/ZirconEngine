use std::sync::Arc;

use serde::{Deserialize, Serialize};
use zircon_runtime::scene::NodeId;

use super::engine::EditCommandError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SceneSelection {
    items: Arc<[NodeId]>,
    primary: Option<NodeId>,
}

impl SceneSelection {
    pub(crate) fn new(items: Vec<NodeId>, primary: Option<NodeId>) -> Self {
        Self {
            items: Arc::from(items),
            primary,
        }
    }

    pub(crate) fn items(&self) -> &[NodeId] {
        self.items.as_ref()
    }

    pub(crate) const fn primary(&self) -> Option<NodeId> {
        self.primary
    }

    #[cfg(test)]
    fn shares_items_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.items, &other.items)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionSnapshot {
    generation: u64,
    state: Arc<SelectionState>,
}

#[derive(Debug, PartialEq, Eq)]
enum SelectionState {
    Empty,
    Scene(SceneSelection),
    #[cfg(test)]
    FixtureValue(u64),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionJournal {
    Empty {
        generation: u64,
    },
    Scene {
        generation: u64,
        items: Vec<NodeId>,
        primary: Option<NodeId>,
    },
    #[cfg(test)]
    FixtureValue {
        generation: u64,
        value: u64,
    },
}

impl Default for SelectionSnapshot {
    fn default() -> Self {
        Self::empty(0)
    }
}

impl SelectionSnapshot {
    /// Empty and scalar fixture states have the same closed Arc allocation
    /// layout. Meter it from a stack value before constructing a snapshot.
    pub fn scalar_retention_bound_v1(
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<u64, EditCommandError> {
        use zircon_runtime_interface::retained_memory::{RetainedBoundBuilderV1, RetainedBytesV1};
        let mut meter = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
        meter.layout(std::mem::size_of::<Self>(), std::mem::align_of::<Self>())?;
        meter.arc_payload(&SelectionState::Empty)?;
        Ok(meter.bytes().get())
    }

    /// Bounds both Arc allocations from a borrowed item count, before a
    /// preview/context binding can allocate the corresponding scene snapshot.
    pub(crate) fn scene_projection_retention_bound_v1(
        count: usize,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<u64, EditCommandError> {
        use zircon_runtime_interface::retained_memory::{
            align_up_v1, RetainedBoundBuilderV1, RetainedBytesV1, RetentionContractError,
        };
        let mut meter = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
        meter.items(count)?;
        meter.layout(std::mem::size_of::<Self>(), std::mem::align_of::<Self>())?;
        let state_align = std::mem::align_of::<SelectionState>().max(8);
        let state = align_up_v1(16, state_align)?
            .checked_add(std::mem::size_of::<SelectionState>())
            .ok_or(RetentionContractError::Overflow)?;
        meter.layout(align_up_v1(state, state_align)?, state_align)?;
        let items_align = std::mem::align_of::<NodeId>().max(8);
        let items = count
            .checked_mul(std::mem::size_of::<NodeId>())
            .and_then(|n| n.checked_add(16))
            .ok_or(RetentionContractError::Overflow)?;
        meter.layout(align_up_v1(items, items_align)?, items_align)?;
        Ok(meter.bytes().get())
    }

    pub(crate) fn empty(generation: u64) -> Self {
        Self {
            generation,
            state: Arc::new(SelectionState::Empty),
        }
    }

    pub(crate) fn scene(generation: u64, selection: SceneSelection) -> Self {
        Self {
            generation,
            state: Arc::new(SelectionState::Scene(selection)),
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn journal_projection(&self) -> SelectionJournal {
        match self.state.as_ref() {
            SelectionState::Empty => SelectionJournal::Empty {
                generation: self.generation,
            },
            SelectionState::Scene(selection) => SelectionJournal::Scene {
                generation: self.generation,
                items: selection.items().to_vec(),
                primary: selection.primary(),
            },
            #[cfg(test)]
            SelectionState::FixtureValue(value) => SelectionJournal::FixtureValue {
                generation: self.generation,
                value: *value,
            },
        }
    }

    pub(crate) fn scene_selection(&self) -> Result<SceneSelection, EditCommandError> {
        match self.state.as_ref() {
            SelectionState::Scene(selection) => Ok(selection.clone()),
            _ => Err(EditCommandError::InvariantViolation {
                invariant:
                    "scene transaction selection snapshot must use the scene selection model",
            }),
        }
    }

    #[cfg(test)]
    pub(crate) fn fixture_value(generation: u64, value: u64) -> Self {
        Self {
            generation,
            state: Arc::new(SelectionState::FixtureValue(value)),
        }
    }

    #[cfg(test)]
    pub(crate) fn fixture_value_ref(&self) -> Result<u64, EditCommandError> {
        match self.state.as_ref() {
            SelectionState::FixtureValue(value) => Ok(*value),
            _ => Err(EditCommandError::InvariantViolation {
                invariant: "fixture selection snapshots contain an unsigned integer",
            }),
        }
    }

    #[cfg(test)]
    pub(crate) fn shares_payload_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
}

#[cfg(test)]
#[path = "tests/selection.rs"]
mod tests;

impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for SceneSelection {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        m.arc_payload(self.items.as_ref())
    }
}
unsafe impl zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1 for SceneSelection {}
impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for SelectionState {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        match self {
            Self::Scene(value) => {
                zircon_runtime_interface::retained_memory::RetainedOwnerV1::retained_bound_v1(
                    value, m,
                )
            }
            Self::Empty => Ok(()),
            #[cfg(test)]
            Self::FixtureValue(_) => Ok(()),
        }
    }
}
unsafe impl zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1 for SelectionState {}
impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for SelectionSnapshot {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        m.arc_payload(self.state.as_ref())
    }
}
unsafe impl zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1
    for SelectionSnapshot
{
}
