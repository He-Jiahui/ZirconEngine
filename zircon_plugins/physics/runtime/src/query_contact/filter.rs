use zircon_runtime::core::framework::physics::{
    PhysicsColliderSyncState, PhysicsQueryFilter, PhysicsRayCastQuery, PhysicsSettings,
};

pub(super) fn collider_matches_query(
    query: &PhysicsRayCastQuery,
    collider: &PhysicsColliderSyncState,
) -> bool {
    collider_matches_filter(&query.filter, collider)
}

pub(super) fn collider_matches_filter(
    filter: &PhysicsQueryFilter,
    collider: &PhysicsColliderSyncState,
) -> bool {
    (filter.include_sensors || !collider.sensor)
        && filter
            .collision_mask
            .is_none_or(|mask| collider_matches_query_mask(collider, mask))
        && !filter.excluded_entities.contains(&collider.entity)
        && filter
            .required_collision_group
            .is_none_or(|group| collider.collision_group == group)
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
