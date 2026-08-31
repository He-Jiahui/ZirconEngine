use serde::{Deserialize, Serialize};

use super::ToolLifecycleEvent;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ToolTransitionRevision(u64);

impl ToolTransitionRevision {
    pub const INITIAL: Self = Self(0);

    pub const fn value(self) -> u64 {
        self.0
    }

    pub(crate) const fn checked_next(self) -> Option<Self> {
        match self.0.checked_add(1) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn for_test(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolTransitionBatch {
    revision: ToolTransitionRevision,
    events: Vec<ToolLifecycleEvent>,
}

impl ToolTransitionBatch {
    pub(crate) fn new(revision: ToolTransitionRevision, events: Vec<ToolLifecycleEvent>) -> Self {
        debug_assert!(!events.is_empty());
        Self { revision, events }
    }

    pub const fn revision(&self) -> ToolTransitionRevision {
        self.revision
    }

    pub fn events(&self) -> &[ToolLifecycleEvent] {
        &self.events
    }
}
