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
