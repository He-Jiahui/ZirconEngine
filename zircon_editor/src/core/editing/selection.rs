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
                invariant: "scene transaction selection snapshot must use the scene selection model",
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
mod tests {
    use super::{SceneSelection, SelectionSnapshot};

    #[test]
    fn typed_scene_selection_handles_share_their_payload_at_scale() {
        for count in [1_u64, 100, 10_000] {
            let model = SceneSelection::new((1..=count).collect(), Some(count));
            let snapshot = SelectionSnapshot::scene(count, model.clone());
            let cloned = snapshot.clone();
            let restored = cloned.scene_selection().unwrap();

            assert_eq!(restored.items().len(), count as usize);
            assert_eq!(restored.primary(), Some(count));
            assert!(snapshot.shares_payload_with(&cloned));
            assert!(model.shares_items_with(&restored));
        }
    }
}
