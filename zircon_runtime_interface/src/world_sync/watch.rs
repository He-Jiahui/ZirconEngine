use serde::{Deserialize, Serialize};

use crate::resource::ResourceId;

use super::EntityId;

/// A runtime-owned fact family that a view or tool can subscribe to.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WatchKey {
    Subtree { root: EntityId },
    ComponentType { type_name: String },
    Asset { resource_id: ResourceId },
    WorldStructure,
}

/// A transport request for one independently revocable watch.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WatchRegistration {
    pub key: WatchKey,
}

impl WatchRegistration {
    pub fn new(key: WatchKey) -> Self {
        Self { key }
    }
}

/// Runtime-issued opaque subscription identity; runtime never stores editor view ids.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WatchToken(u64);

impl WatchToken {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}
