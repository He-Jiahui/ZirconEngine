use std::collections::HashSet;

use zircon_runtime::core::framework::physics::{
    PhysicsColliderSyncState, PhysicsQueryFilter, PhysicsSettings,
};
use zircon_runtime::core::framework::scene::EntityId;

const HASHED_EXCLUSION_THRESHOLD: usize = 8;

pub(crate) struct PreparedPhysicsQueryFilter<'a> {
    filter: &'a PhysicsQueryFilter,
    excluded_entities: PreparedExcludedEntities<'a>,
}

enum PreparedExcludedEntities<'a> {
    Linear(&'a [EntityId]),
    Hashed(HashSet<EntityId>),
}

impl<'a> PreparedPhysicsQueryFilter<'a> {
    pub(crate) fn new(filter: &'a PhysicsQueryFilter) -> Self {
        let excluded_entities = if filter.excluded_entities.len() <= HASHED_EXCLUSION_THRESHOLD {
            PreparedExcludedEntities::Linear(&filter.excluded_entities)
        } else {
            PreparedExcludedEntities::Hashed(filter.excluded_entities.iter().copied().collect())
        };
        Self {
            filter,
            excluded_entities,
        }
    }

    pub(crate) fn matches(&self, collider: &PhysicsColliderSyncState) -> bool {
        (self.filter.include_sensors || !collider.sensor)
            && self
                .filter
                .collision_mask
                .is_none_or(|mask| collider_matches_query_mask(collider, mask))
            && !self.excluded_entities.contains(collider.entity)
            && self
                .filter
                .required_collision_group
                .is_none_or(|group| collider.collision_group == group)
    }
}

impl PreparedExcludedEntities<'_> {
    fn contains(&self, entity: EntityId) -> bool {
        match self {
            Self::Linear(entities) => entities.contains(&entity),
            Self::Hashed(entities) => entities.contains(&entity),
        }
    }
}

pub(super) fn colliders_can_interact(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
    settings: &PhysicsSettings,
) -> bool {
    let Some(left_layer) = collision_layer_bit(left.layer) else {
        return false;
    };
    let Some(right_layer) = collision_layer_bit(right.layer) else {
        return false;
    };

    left.collision_mask & right_layer != 0
        && right.collision_mask & left_layer != 0
        && collision_matrix_allows(settings, left.layer, right.layer)
        && collision_matrix_allows(settings, right.layer, left.layer)
}

fn collider_matches_query_mask(collider: &PhysicsColliderSyncState, query_mask: u32) -> bool {
    collision_layer_bit(collider.layer).is_some_and(|layer_bit| query_mask & layer_bit != 0)
}

fn collision_matrix_allows(
    settings: &PhysicsSettings,
    source_layer: u32,
    target_layer: u32,
) -> bool {
    let Some(row) = settings.collision_matrix.get(source_layer as usize) else {
        return false;
    };
    let Some(target_bit) = 1_u64.checked_shl(target_layer) else {
        return false;
    };
    row & target_bit != 0
}

fn collision_layer_bit(layer: u32) -> Option<u32> {
    1_u32.checked_shl(layer)
}

#[cfg(test)]
mod tests {
    use zircon_runtime::core::framework::physics::PhysicsColliderShape;
    use zircon_runtime::core::math::Transform;

    use super::*;

    #[test]
    fn prepared_filter_hashes_large_exclusion_sets_and_preserves_membership() {
        let filter = PhysicsQueryFilter {
            excluded_entities: (1..=32).collect(),
            ..PhysicsQueryFilter::default()
        };
        let prepared = PreparedPhysicsQueryFilter::new(&filter);

        assert!(matches!(
            &prepared.excluded_entities,
            PreparedExcludedEntities::Hashed(_)
        ));
        assert!(!prepared.matches(&collider(17)));
        assert!(prepared.matches(&collider(33)));
    }

    #[test]
    fn prepared_filter_keeps_small_exclusion_sets_allocation_free() {
        let filter = PhysicsQueryFilter {
            excluded_entities: vec![2, 4, 6],
            ..PhysicsQueryFilter::default()
        };
        let prepared = PreparedPhysicsQueryFilter::new(&filter);

        assert!(matches!(
            &prepared.excluded_entities,
            PreparedExcludedEntities::Linear(_)
        ));
        assert!(!prepared.matches(&collider(4)));
        assert!(prepared.matches(&collider(5)));
    }

    fn collider(entity: EntityId) -> PhysicsColliderSyncState {
        PhysicsColliderSyncState {
            entity,
            shape: PhysicsColliderShape::Sphere { radius: 1.0 },
            sensor: false,
            layer: 0,
            collision_group: 0,
            collision_mask: u32::MAX,
            material: None,
            material_override: None,
            transform: Transform::default(),
        }
    }
}
