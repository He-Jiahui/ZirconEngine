use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::scene::EntityId;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityRemap {
    mappings: BTreeMap<EntityId, EntityId>,
}

impl EntityRemap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, source: EntityId, target: EntityId) -> Option<EntityId> {
        self.mappings.insert(source, target)
    }

    pub fn get(&self, source: EntityId) -> Option<EntityId> {
        self.mappings.get(&source).copied()
    }

    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (EntityId, EntityId)> + '_ {
        self.mappings
            .iter()
            .map(|(source, target)| (*source, *target))
    }
}
