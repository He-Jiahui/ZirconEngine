use super::*;
use crate::scene::ecs::{Mut, StorageType, With};
use std::panic::{catch_unwind, AssertUnwindSafe};

#[derive(Debug, PartialEq, Eq)]
struct SparseValue(u32);

impl Component for SparseValue {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

#[test]
fn moved_world_keeps_binding_but_clone_and_deserialization_require_rebind() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Owner".to_string()),)).unwrap();
    let mut query = NameQuery::new(&mut world);
    let mut query_clone = query.clone();
    let mut moved = Box::new(world);
    assert!(query.is_bound_to(&moved));
    assert_eq!(
        query
            .iter_cached(&moved)
            .map(|(id, _)| id)
            .collect::<Vec<_>>(),
        vec![entity]
    );

    let mut cloned = moved.as_ref().clone();
    let mut restored: World =
        serde_json::from_str(&serde_json::to_string(&*moved).unwrap()).unwrap();
    assert_eq!(moved.archetype_generation(), cloned.archetype_generation());
    assert!(!query.is_bound_to(&cloned));
    assert!(!query.is_bound_to(&restored));
    assert!(query_clone.is_bound_to(&moved));
    assert!(catch_unwind(AssertUnwindSafe(|| query.update_cache(&cloned))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| query_clone.update_cache(&restored))).is_err());

    query.rebind(&mut cloned).unwrap();
    query_clone.rebind(&mut restored).unwrap();
    assert!(query.is_bound_to(&cloned));
    assert!(!query.is_bound_to(&moved));
    assert!(query_clone.is_bound_to(&restored));
    assert_eq!(
        query
            .iter_cached(&cloned)
            .map(|(_, name)| name.0.as_str())
            .collect::<Vec<_>>(),
        vec!["Owner"]
    );
    assert_eq!(
        query_clone
            .iter_cached(&restored)
            .map(|(_, name)| name.0.as_str())
            .collect::<Vec<_>>(),
        vec!["Owner"]
    );
    let same_owner = NameQuery::new(&mut moved);
    assert!(catch_unwind(AssertUnwindSafe(|| query.conflicts_with(&same_owner))).is_err());
}

#[test]
fn cached_entry_points_reject_other_owner_and_full_rebind_rebuilds_component_ids() {
    let mut original = World::empty();
    let mut other = World::empty();
    let original_health = original.component_id::<Health>();
    original.component_id::<SparseValue>();
    other.component_id::<SparseValue>();
    let other_health = other.component_id::<Health>();
    assert_ne!(original_health, other_health);
    original.spawn((Health(10), SparseValue(20))).unwrap();
    let other_entity = other.spawn((Health(30), SparseValue(40))).unwrap();
    assert_eq!(
        original.archetype_generation(),
        other.archetype_generation()
    );
    type ReadQuery = QueryState<(EntityId, &'static Health), With<SparseValue>>;
    let mut query = ReadQuery::new(&mut original);
    let before = query.cache_stats();
    assert!(catch_unwind(AssertUnwindSafe(|| query.update_cache(&other))).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| query.iter_cached(&other).count())).is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| query
        .iter_cached_direct(&other)
        .count()))
    .is_err());
    let after = query.cache_stats();
    assert_eq!(after.cache_hits, before.cache_hits);
    assert_eq!(after.cache_misses, before.cache_misses);
    assert_eq!(after.cache_rebuilds, before.cache_rebuilds);
    assert_eq!(after.cached_revision, before.cached_revision);
    assert_eq!(after.cached_archetype_count, before.cached_archetype_count);
    let other_query = ReadQuery::new(&mut other);
    assert!(catch_unwind(AssertUnwindSafe(|| query.conflicts_with(&other_query))).is_err());
    assert_eq!(
        query
            .iter(&other)
            .map(|(id, health)| (id, health.0))
            .collect::<Vec<_>>(),
        vec![(other_entity, 30)]
    );

    let mut mutable = QueryState::<Mut<'static, Health>>::new(&mut original);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = mutable.get_mut(&mut other, other_entity);
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| mutable.iter_mut(&mut other).count())).is_err());
    query.rebind(&mut other).unwrap();
    mutable.rebind(&mut other).unwrap();
    assert!(query.is_bound_to(&other));
    assert!(query.access().reads().contains(&other_health));
    assert_eq!(
        query
            .iter_cached_direct(&other)
            .map(|(id, health)| (id, health.0))
            .collect::<Vec<_>>(),
        vec![(other_entity, 30)]
    );
    mutable.get_mut(&mut other, other_entity).unwrap().0 += 1;
    assert_eq!(
        query
            .iter_cached(&other)
            .map(|(_, health)| health.0)
            .collect::<Vec<_>>(),
        vec![31]
    );

    let mut sparse = QueryState::<Mut<'static, SparseValue>>::new(&mut original);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = sparse.get_mut(&mut other, other_entity);
    }))
    .is_err());
    sparse.rebind(&mut other).unwrap();
    sparse.get_mut(&mut other, other_entity).unwrap().0 += 2;
    let sparse_read = QueryState::<&SparseValue>::new(&mut other);
    assert_eq!(
        sparse_read
            .iter(&other)
            .map(|value| value.0)
            .collect::<Vec<_>>(),
        vec![42]
    );
    let mut same_owner = ReadQuery::new(&mut other);
    query.rebind(&mut other).unwrap();
    assert!(!query.conflicts_with(&same_owner));
    other.spawn((Health(50), SparseValue(60))).unwrap();
    assert_eq!(query.iter_cached(&other).count(), 2);
    assert_eq!(same_owner.iter_cached(&other).count(), 2);
}
