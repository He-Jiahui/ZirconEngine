use std::sync::Arc;

use super::terminal::PreferenceMutationTerminal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceDurabilityState {
    Durable,
    Pending,
    VisibleNotDurable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferenceReadSnapshot {
    generation: u64,
    value: Option<Arc<[u8]>>,
    durability: PreferenceDurabilityState,
    last_terminal: Option<PreferenceMutationTerminal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreferenceEviction {
    generation: u64,
    durability: PreferenceDurabilityState,
    last_terminal: Option<PreferenceMutationTerminal>,
}

impl PreferenceEviction {
    pub(crate) fn new(
        generation: u64,
        durability: PreferenceDurabilityState,
        last_terminal: Option<PreferenceMutationTerminal>,
    ) -> Self {
        Self {
            generation,
            durability,
            last_terminal,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub const fn durability(&self) -> PreferenceDurabilityState {
        self.durability
    }

    pub fn last_terminal(&self) -> Option<&PreferenceMutationTerminal> {
        self.last_terminal.as_ref()
    }
}

impl PreferenceReadSnapshot {
    pub(crate) fn new(
        generation: u64,
        value: Option<Arc<[u8]>>,
        durability: PreferenceDurabilityState,
        last_terminal: Option<PreferenceMutationTerminal>,
    ) -> Self {
        Self {
            generation,
            value,
            durability,
            last_terminal,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    pub fn value(&self) -> Option<&[u8]> {
        self.value.as_deref()
    }

    pub const fn durability(&self) -> PreferenceDurabilityState {
        self.durability
    }

    pub fn last_terminal(&self) -> Option<&PreferenceMutationTerminal> {
        self.last_terminal.as_ref()
    }
}
