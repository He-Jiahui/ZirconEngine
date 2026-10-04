use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Version 8 UUID namespace reserved for deterministic legacy layout migration.
const LEGACY_ID_PREFIX: u128 = 0x9d7e4bcc_2cd7_8d6d_8000_000000000000;

#[derive(Clone, Copy, Debug)]
pub struct DocumentNodeId(uuid::Uuid, bool);

impl Default for DocumentNodeId {
    fn default() -> Self {
        Self(uuid::Uuid::new_v4(), false)
    }
}

impl DocumentNodeId {
    pub(crate) fn nil() -> Self {
        Self(uuid::Uuid::nil(), false)
    }

    pub(crate) fn is_migrated(self) -> bool {
        self.1
    }

    pub(crate) fn migrated_ordinal(ordinal: u64) -> Self {
        Self(
            uuid::Uuid::from_u128(LEGACY_ID_PREFIX | u128::from(ordinal)),
            true,
        )
    }

    pub fn is_nil(self) -> bool {
        self.0.is_nil()
    }
}

impl PartialEq for DocumentNodeId {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for DocumentNodeId {}

impl PartialOrd for DocumentNodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DocumentNodeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for DocumentNodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Serialize for DocumentNodeId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DocumentNodeId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(uuid::Uuid::deserialize(deserializer)?, false))
    }
}

pub(crate) struct DocumentNodeIdRepair {
    reserved: HashSet<DocumentNodeId>,
    seen: HashSet<DocumentNodeId>,
    next_ordinal: u64,
}

impl DocumentNodeIdRepair {
    pub(crate) fn new(reserved: HashSet<DocumentNodeId>) -> Self {
        Self {
            reserved,
            seen: HashSet::new(),
            next_ordinal: 1,
        }
    }

    pub(crate) fn normalize(&mut self, node_id: &mut DocumentNodeId) {
        if !node_id.is_nil() && !node_id.is_migrated() && self.seen.insert(*node_id) {
            return;
        }

        if node_id.is_migrated() && !self.reserved.contains(node_id) && self.seen.insert(*node_id) {
            return;
        }

        loop {
            let candidate = DocumentNodeId::migrated_ordinal(self.next_ordinal);
            self.next_ordinal = self
                .next_ordinal
                .checked_add(1)
                .expect("legacy layout node count exceeds UUID migration space");
            if !self.reserved.contains(&candidate) && self.seen.insert(candidate) {
                *node_id = candidate;
                return;
            }
        }
    }
}

impl std::fmt::Display for DocumentNodeId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, formatter)
    }
}
