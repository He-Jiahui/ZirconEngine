use std::any::TypeId;

use crate::scene::World;
use crate::scene::ecs::{ArchetypeId, ComponentId, ComponentStorageLocation, StableEntityLocation};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueryComponentBinding {
    Table {
        component_id: ComponentId,
        rust_type_id: TypeId,
        column_slot: usize,
    },
    SparseSet {
        component_id: ComponentId,
        rust_type_id: TypeId,
    },
}

impl QueryComponentBinding {
    pub(crate) const fn component_id(self) -> ComponentId {
        match self {
            Self::Table { component_id, .. } | Self::SparseSet { component_id, .. } => component_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CachedArchetypePlan {
    archetype_id: ArchetypeId,
    membership_generation: u64,
    bindings: Vec<QueryComponentBinding>,
}

impl CachedArchetypePlan {
    pub(crate) fn new(
        archetype_id: ArchetypeId,
        membership_generation: u64,
        bindings: Vec<QueryComponentBinding>,
    ) -> Self {
        Self {
            archetype_id,
            membership_generation,
            bindings,
        }
    }

    pub(crate) const fn archetype_id(&self) -> ArchetypeId {
        self.archetype_id
    }

    pub(crate) const fn membership_generation(&self) -> u64 {
        self.membership_generation
    }

    pub(crate) fn bindings(&self) -> &[QueryComponentBinding] {
        &self.bindings
    }

    pub(crate) fn estimated_heap_bytes(&self) -> usize {
        self.bindings
            .capacity()
            .saturating_mul(std::mem::size_of::<QueryComponentBinding>())
    }

    pub(crate) fn refresh_membership_generation(&mut self, generation: u64) {
        self.membership_generation = generation;
    }

    pub(crate) fn write_component_locations(
        &self,
        world: &World,
        stable_location: StableEntityLocation,
        output: &mut Vec<ComponentStorageLocation>,
    ) -> bool {
        output.clear();
        if stable_location.location.archetype_id != self.archetype_id {
            return false;
        }
        output.reserve(self.bindings.len());
        for binding in &self.bindings {
            match *binding {
                QueryComponentBinding::Table {
                    component_id,
                    rust_type_id,
                    column_slot,
                } => output.push(
                    ComponentStorageLocation::table(
                        component_id,
                        stable_location.internal,
                        self.archetype_id,
                        stable_location.location.table_row,
                        column_slot,
                    )
                    .with_rust_type_id(rust_type_id),
                ),
                QueryComponentBinding::SparseSet {
                    component_id,
                    rust_type_id,
                } => {
                    let Some(location) = world
                        .query_sparse_component_location(component_id, stable_location.internal)
                    else {
                        output.clear();
                        return false;
                    };
                    output.push(location.with_rust_type_id(rust_type_id));
                }
            }
        }
        true
    }
}

pub(crate) fn find_cached_archetype_plan(
    plans: &[CachedArchetypePlan],
    archetype: ArchetypeId,
) -> Option<&CachedArchetypePlan> {
    let index = plans
        .binary_search_by_key(&archetype, CachedArchetypePlan::archetype_id)
        .ok()?;
    plans.get(index)
}

pub(crate) fn project_entity_from_plans(
    plans: &[CachedArchetypePlan],
    world: &World,
    entity: crate::scene::EntityId,
    component_locations: &mut Vec<ComponentStorageLocation>,
) -> Option<StableEntityLocation> {
    let stable_location = world.internal_entity_location(entity)?;
    let plan = find_cached_archetype_plan(plans, stable_location.location.archetype_id)?;
    plan.write_component_locations(world, stable_location, component_locations)
        .then_some(stable_location)
}
