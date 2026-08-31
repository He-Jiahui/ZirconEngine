use super::super::{RandomService, RandomServiceError, RandomServiceLimits};
use super::keyed;

#[test]
fn capacity_rejects_only_new_keys_and_eviction_releases_admission() {
    let service = RandomService::with_limits(53, RandomServiceLimits::new(1));
    service
        .acquire_stream(keyed(1))
        .expect("first key is within capacity")
        .release();

    assert_eq!(
        service
            .acquire_stream(keyed(2))
            .expect_err("a second registered key exceeds capacity"),
        RandomServiceError::StreamCapacityExceeded { capacity: 1 }
    );
    service
        .acquire_stream(keyed(1))
        .expect("an existing key can resume at capacity")
        .release();
    service
        .evict_stream(keyed(1))
        .expect("parked stream eviction")
        .expect("first key was registered");
    service
        .acquire_stream(keyed(2))
        .expect("eviction releases one admission slot")
        .release();
}

#[test]
fn checkpoint_restore_rejects_a_registry_larger_than_the_runtime_limit() {
    let source = RandomService::new(59);
    source
        .acquire_stream(keyed(1))
        .expect("first key admission")
        .release();
    source
        .acquire_stream(keyed(2))
        .expect("second key admission")
        .release();
    let checkpoint = source.checkpoint().expect("idle registry checkpoint");

    assert_eq!(
        RandomService::from_checkpoint_with_limits(checkpoint, RandomServiceLimits::new(1))
            .expect_err("checkpoint exceeds destination capacity"),
        RandomServiceError::StreamCapacityExceeded { capacity: 1 }
    );
}

#[test]
fn entity_and_world_eviction_are_atomic_against_active_leases() {
    let service = RandomService::new(61);
    service
        .acquire_stream(keyed(44))
        .expect("first entity stream")
        .release();
    service
        .acquire_stream(keyed(45))
        .expect("second entity stream")
        .release();

    let key = keyed(44);
    let active = service.acquire_stream(key).expect("active entity lease");
    assert_eq!(
        service
            .evict_entity(key.world(), key.entity().expect("entity-scoped stream key"),)
            .expect_err("target entity has an active lease"),
        RandomServiceError::StreamScopeEvictionBlocked { active_leases: 1 }
    );
    assert_eq!(service.registered_stream_count(), 2);
    drop(active);

    let entity_evicted = service
        .evict_entity(key.world(), key.entity().expect("entity-scoped stream key"))
        .expect("idle entity scope eviction");
    assert_eq!(entity_evicted.len(), 1);
    assert_eq!(entity_evicted[0].master_seed_generation(), 0);
    assert_eq!(service.registered_stream_count(), 1);
    let world_evicted = service
        .evict_world(key.world())
        .expect("idle world scope eviction");
    assert_eq!(world_evicted.len(), 1);
    assert_eq!(world_evicted[0].master_seed_generation(), 0);
    assert_eq!(service.registered_stream_count(), 0);
}

#[test]
fn scope_eviction_checkpoints_follow_the_reseeded_authority_generation() {
    let mut service = RandomService::new(67);
    service.reseed(71).expect("idle service should reseed");
    let key = keyed(44);
    service
        .acquire_stream(key)
        .expect("reseeded entity stream")
        .release();

    let evicted = service
        .evict_entity(key.world(), key.entity().expect("entity-scoped stream key"))
        .expect("idle entity scope eviction");

    assert_eq!(evicted.len(), 1);
    assert_eq!(evicted[0].master_seed_generation(), 1);
}
