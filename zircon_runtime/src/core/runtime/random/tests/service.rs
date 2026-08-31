use zr_contracts::random::{
    RandomAlgorithmId, RandomEntityKey, RandomServiceState, RandomStreamKey, RandomWorldKey,
};

use super::super::{RandomService, RandomServiceError, RandomStream};
use super::key;

#[test]
fn service_derivation_is_stable_and_isolates_world_entity_and_seed_generations() {
    let key = key();
    let service = RandomService::new(0x51a7_2026);
    let derived = service
        .acquire_stream(key)
        .expect("stream admission")
        .release();
    assert_eq!(derived.generator_state(), 0x0de0_7104_6184_94d6);
    assert_eq!(derived.increment(), 0xcedd_aca0_6cc3_4e29);
    assert_eq!(derived.sequence_id().value(), 0x676e_d650_3661_a714);
    assert_eq!(derived.draw_index(), 0);

    let changed_entity = RandomStreamKey::for_entity(
        key.world(),
        RandomEntityKey::new(45, 2),
        key.system(),
        key.purpose(),
        key.authoring_seed(),
    );
    let changed_world_generation = RandomStreamKey::for_entity(
        RandomWorldKey::new(7, 4),
        key.entity().expect("entity-scoped key"),
        key.system(),
        key.purpose(),
        key.authoring_seed(),
    );
    let changed_authoring_seed = RandomStreamKey::for_entity(
        key.world(),
        key.entity().expect("entity-scoped key"),
        key.system(),
        key.purpose(),
        key.authoring_seed().saturating_add(1),
    );
    let changed_entity_state = service
        .acquire_stream(changed_entity)
        .expect("changed entity stream")
        .release();
    let changed_world_state = service
        .acquire_stream(changed_world_generation)
        .expect("changed world stream")
        .release();
    let changed_seed_state = service
        .acquire_stream(changed_authoring_seed)
        .expect("changed seed stream")
        .release();
    assert_ne!(derived, changed_entity_state);
    assert_ne!(derived, changed_world_state);
    assert_ne!(derived, changed_seed_state);
}

#[test]
fn service_state_restores_authority_but_checkpoint_restores_stream_progress() {
    let service = RandomService::new(0x2244);
    let mut stream = service.acquire_stream(key()).expect("stream admission");
    stream.try_next_u32().expect("first draw");
    stream.release();
    let authority = service.snapshot();
    let checkpoint = service.checkpoint().expect("idle registry checkpoint");
    assert_eq!(checkpoint.streams()[0].master_seed_generation(), 0);

    let empty_registry = RandomService::from_state(authority);
    let restored_registry = RandomService::from_checkpoint(checkpoint.clone())
        .expect("checkpoint within the default stream capacity");
    assert_eq!(empty_registry.registered_stream_count(), 0);
    assert_eq!(restored_registry.registered_stream_count(), 1);

    let derived_again = empty_registry
        .acquire_stream(key())
        .expect("empty registry derives")
        .snapshot();
    let resumed = restored_registry
        .acquire_stream(key())
        .expect("checkpoint registry resumes")
        .snapshot();
    assert_eq!(derived_again.draw_index(), 0);
    assert_eq!(resumed.draw_index(), 1);

    let unseen_key = RandomStreamKey::for_entity(
        key().world(),
        RandomEntityKey::new(46, 2),
        key().system(),
        key().purpose(),
        key().authoring_seed(),
    );
    let expected_unseen = RandomService::from_state(checkpoint.service_state())
        .acquire_stream(unseen_key)
        .expect("authority-only service derives unseen key")
        .snapshot();
    let restored_unseen = restored_registry
        .acquire_stream(unseen_key)
        .expect("checkpoint service derives unseen key")
        .snapshot();
    assert_eq!(restored_unseen, expected_unseen);
}

#[test]
fn reseed_rejects_generation_exhaustion_without_mutating_seed_authority() {
    let state = RandomServiceState::new(RandomAlgorithmId::Pcg32XshRrV1, 0x2244, u64::MAX);
    let mut service = RandomService::from_state(state);
    service
        .acquire_stream(key())
        .expect("stream admission before failed reseed")
        .release();
    let before = service.snapshot();
    let checkpoint_before = service.checkpoint().expect("idle registry checkpoint");

    assert_eq!(
        service.reseed(0x6688),
        Err(RandomServiceError::SeedGenerationExhausted {
            generation: u64::MAX,
        })
    );
    assert_eq!(service.snapshot(), before);
    assert_eq!(service.checkpoint(), Ok(checkpoint_before));
}

#[test]
fn detached_stream_restore_preserves_the_exact_next_draw_and_draw_index() {
    let service = RandomService::new(11);
    let mut original = service.acquire_stream(key()).expect("stream admission");
    let _ = original.try_next_u32().expect("first draw");
    let state = original.snapshot();
    let mut restored = RandomStream::from_state(state).expect("valid snapshot");

    assert_eq!(original.try_next_u32(), restored.try_next_u32());
    assert_eq!(original.draw_index(), restored.draw_index());
}
