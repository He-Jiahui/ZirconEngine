use std::collections::{btree_map::Entry, BTreeMap};

use crate::scene::ecs::{storage::StoredComponent, ComponentId, ComponentTicks};

use super::ArchetypeTableError;

/// Owns a fully validated set of values until one archetype table publishes it.
pub(crate) struct ArchetypePreflightedRow {
    components: BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
}

impl ArchetypePreflightedRow {
    pub(super) fn collect(
        components: impl IntoIterator<Item = (ComponentId, StoredComponent, ComponentTicks)>,
    ) -> Result<Self, ArchetypeTableError> {
        let mut values = BTreeMap::new();
        for (component_id, value, ticks) in components {
            match values.entry(component_id) {
                Entry::Vacant(entry) => {
                    entry.insert((value, ticks));
                }
                Entry::Occupied(_) => {
                    return Err(ArchetypeTableError::DuplicateComponentColumn { component_id });
                }
            }
        }
        Ok(Self { components: values })
    }

    pub(super) fn components(&self) -> &BTreeMap<ComponentId, (StoredComponent, ComponentTicks)> {
        &self.components
    }

    pub(super) fn from_validated_components(
        components: BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> Self {
        Self { components }
    }

    pub(super) fn into_components(
        self,
    ) -> BTreeMap<ComponentId, (StoredComponent, ComponentTicks)> {
        self.components
    }
}
