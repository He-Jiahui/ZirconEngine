use std::collections::BTreeMap;

use crate::scene::ecs::{storage::StoredComponent, ComponentId, ComponentTicks};
use crate::scene::EntityId;

pub(crate) struct ArchetypeTakenRow {
    entity: EntityId,
    swapped_entity: Option<EntityId>,
    components: BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
}

impl ArchetypeTakenRow {
    pub(super) fn new(
        entity: EntityId,
        swapped_entity: Option<EntityId>,
        components: BTreeMap<ComponentId, (StoredComponent, ComponentTicks)>,
    ) -> Self {
        Self {
            entity,
            swapped_entity,
            components,
        }
    }

    pub(crate) fn entity(&self) -> EntityId {
        self.entity
    }

    pub(crate) fn swapped_entity(&self) -> Option<EntityId> {
        self.swapped_entity
    }

    pub(crate) fn into_components(
        self,
    ) -> BTreeMap<ComponentId, (StoredComponent, ComponentTicks)> {
        self.components
    }
}
