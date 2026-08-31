use super::*;
use crate::text::font::shared::force_publish_shared_font_database;
use crate::text::font::{
    FontDatabase, font_handle_resolver_snapshot, shared_font_collection_service,
    shared_font_collection_snapshot, shared_font_database_snapshot,
    shared_font_database_test_read_guard, shared_font_database_test_serial_guard,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

const TEST_COLLECTION: TextFontCollectionHandle = TextFontCollectionHandle::new(101);

#[test]
fn generation_change_invalidates_old_slots_without_reinterpreting_backend_ids() {
    let mut registry = FontHandleRegistry::new(TEST_COLLECTION);
    let backend_face = FontFaceId(u64::from(u32::MAX) + 41);
    let first = registry
        .register_unique_pairs(&[(Some(backend_face), None)], 9)
        .into_iter()
        .next()
        .and_then(|(face, _)| face)
        .expect("first handle");

    assert_eq!(registry.resolve_face(first), Some(backend_face));

    let reloaded = registry
        .register_unique_pairs(&[(Some(backend_face), None)], 10)
        .into_iter()
        .next()
        .and_then(|(face, _)| face)
        .expect("reloaded handle");
    assert_eq!(registry.resolve_face(first), None);
    assert_eq!(registry.resolve_face(reloaded), Some(backend_face));
    assert_eq!(reloaded.generation, 10);
}

#[test]
fn shared_database_reload_rejects_pre_reload_handle() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let (generation, database) = shared_font_database_snapshot();
    let backend_face = FontFaceId(1);
    let before_reload = register_font_face_handle(backend_face, generation)
        .expect("pre-reload face should receive a slot");
    assert_eq!(resolve_font_face_handle(before_reload), Some(backend_face));

    let reloaded_generation = force_publish_shared_font_database(&database);

    assert!(reloaded_generation > generation);
    assert_eq!(resolve_font_face_handle(before_reload), None);
    let after_reload = register_font_face_handle(backend_face, reloaded_generation)
        .expect("reloaded face should receive a new-generation slot");
    assert_eq!(resolve_font_face_handle(after_reload), Some(backend_face));
    assert_ne!(before_reload, after_reload);
}

#[test]
fn retained_resolver_snapshot_keeps_an_in_flight_generation_resolvable() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let font_collection = shared_font_collection_snapshot();
    let backend_pair = (Some(FontFaceId(1_103)), Some(InstancedFaceId(1_107)));
    let handles = register_font_handle_batch_for_collection(
        font_collection.service(),
        &[backend_pair],
        font_collection.generation(),
    );
    let resolver = font_handle_resolver_snapshot(&font_collection);
    let (_, database) = shared_font_database_snapshot();

    assert!(force_publish_shared_font_database(&database) > font_collection.generation());
    assert_eq!(resolve_font_handle_batch(&handles), vec![(None, None)]);
    assert_eq!(
        resolve_font_handle_batch_from_snapshot(&resolver, &handles),
        vec![backend_pair]
    );
}

#[test]
fn stale_projection_cannot_roll_registry_generation_back() {
    let mut registry = FontHandleRegistry::new(TEST_COLLECTION);
    let current = registry
        .register_unique_pairs(&[(Some(FontFaceId(7)), None)], 12)
        .into_iter()
        .next()
        .and_then(|(face, _)| face)
        .expect("current generation handle");

    assert_eq!(
        registry.register_unique_pairs(&[(Some(FontFaceId(9)), None)], 11),
        vec![(None, None)]
    );
    assert_eq!(registry.generation, 12);
    assert_eq!(registry.resolve_face(current), Some(FontFaceId(7)));
}

#[test]
fn registry_resolution_rejects_a_generation_change_after_its_initial_probe() {
    let mut registry = FontHandleRegistry::new(TEST_COLLECTION);
    let handle = registry
        .register_unique_pairs(&[(Some(FontFaceId(31)), None)], 7)
        .into_iter()
        .next()
        .and_then(|(face, _)| face)
        .expect("current generation handle");
    let snapshot = FontHandleRegistrySnapshot::from(&registry);

    assert!(!snapshot_matches_font_database_generation(
        &snapshot,
        TEST_COLLECTION,
        7,
        TEST_COLLECTION,
        8
    ));
    assert_eq!(snapshot.resolve_face(handle), Some(FontFaceId(31)));
}

#[test]
fn paired_font_handles_roundtrip_face_and_instance_together() {
    let (generation, _database) = shared_font_database_test_read_guard();
    let face = FontFaceId(23);
    let instance = InstancedFaceId(29);

    let (face_handle, instance_handle) =
        register_font_handles(Some(face), Some(instance), generation);
    let resolved = resolve_font_handles(face_handle, instance_handle);

    assert_eq!(resolved, (Some(face), Some(instance)));
}

#[test]
fn mixed_generation_font_handle_pair_is_rejected_atomically() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let (generation, _database) = shared_font_database_snapshot();
    let registered =
        register_font_handles(Some(FontFaceId(31)), Some(InstancedFaceId(37)), generation);
    let stale_instance = registered.1.map(|handle| {
        TextFontFaceHandle::new(handle.collection, handle.index, generation.wrapping_add(1))
    });
    let before = font_handle_registry_report();

    let resolved = resolve_font_handles(registered.0, stale_instance);
    let after = font_handle_registry_report();

    assert_eq!(resolved, (None, None));
    assert_eq!(
        after.resolution_rejected_pair_count,
        before.resolution_rejected_pair_count + 1
    );
}

#[test]
fn font_handle_registry_recovers_after_writer_lock_poisoning() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let poison_result = catch_unwind(AssertUnwindSafe(|| {
        let font_collection = shared_font_collection_service();
        let _registry = font_collection
            .handle_registry()
            .registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        panic!("poison font handle registry for recovery coverage");
    }));
    assert!(poison_result.is_err());
    let generation = shared_font_database_generation();
    let backend_face = FontFaceId(4_001);

    let handle = register_font_face_handle(backend_face, generation)
        .expect("poison recovery must still register a face");

    assert_eq!(resolve_font_face_handle(handle), Some(backend_face));
}

#[test]
fn independently_owned_font_collections_cannot_resolve_each_others_handles() {
    let first_collection = FontCollectionService::from_database(FontDatabase::default());
    let second_collection = FontCollectionService::from_database(FontDatabase::default());
    let backend_pair = (Some(FontFaceId(4_101)), Some(InstancedFaceId(4_103)));
    let generation = first_collection.generation();

    assert_eq!(generation, second_collection.generation());
    let first =
        register_font_handle_batch_for_collection(&first_collection, &[backend_pair], generation);
    let second =
        register_font_handle_batch_for_collection(&second_collection, &[backend_pair], generation);

    assert_ne!(first, second);
    assert_eq!(
        resolve_font_handle_batch_for_collection(&first_collection, &first),
        vec![backend_pair]
    );
    assert_eq!(
        resolve_font_handle_batch_for_collection(&second_collection, &second),
        vec![backend_pair]
    );
    assert_eq!(
        resolve_font_handle_batch_for_collection(&first_collection, &second),
        vec![(None, None)]
    );
    assert_eq!(
        resolve_font_handle_batch_for_collection(&second_collection, &first),
        vec![(None, None)]
    );
}

#[test]
fn font_handle_batch_projection_and_resolution_deduplicate_repeated_pairs() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let (generation, _database) = shared_font_database_snapshot();
    let repeated = (Some(FontFaceId(1_001)), Some(InstancedFaceId(2_001)));
    let distinct = (Some(FontFaceId(1_002)), Some(InstancedFaceId(2_002)));
    let pairs = vec![repeated, repeated, distinct, repeated];

    let before_registration = font_handle_registry_report();
    let registered = register_font_handle_batch(&pairs, generation);
    let after_registration = font_handle_registry_report();

    assert_eq!(registered.len(), pairs.len());
    assert_eq!(registered[0], registered[1]);
    assert_eq!(registered[0], registered[3]);
    assert_ne!(registered[0], registered[2]);
    assert_eq!(
        after_registration.registration_batch_count,
        before_registration.registration_batch_count + 1
    );
    assert_eq!(
        after_registration.registration_lock_acquire_count,
        before_registration.registration_lock_acquire_count + 1
    );
    assert_eq!(
        after_registration.registration_snapshot_publish_count,
        before_registration.registration_snapshot_publish_count + 1
    );
    assert_eq!(
        after_registration.registration_unique_pair_count,
        before_registration.registration_unique_pair_count + 2
    );

    let before_resolution = font_handle_registry_report();
    let resolved = resolve_font_handle_batch(&registered);
    let after_resolution = font_handle_registry_report();

    assert_eq!(resolved, pairs);
    assert_eq!(
        after_resolution.resolution_batch_count,
        before_resolution.resolution_batch_count + 1
    );
    assert_eq!(
        after_resolution.resolution_snapshot_acquire_count,
        before_resolution.resolution_snapshot_acquire_count + 1
    );
    assert_eq!(
        after_resolution.resolution_unique_pair_count,
        before_resolution.resolution_unique_pair_count + 2
    );
}

#[test]
fn reported_font_handle_batch_keeps_registration_metrics_local_to_one_call() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let (_, database) = shared_font_database_snapshot();
    let generation = force_publish_shared_font_database(&database);
    let repeated = (Some(FontFaceId(91_001)), Some(InstancedFaceId(92_001)));
    let distinct = (Some(FontFaceId(91_002)), Some(InstancedFaceId(92_002)));
    let pairs = [repeated, repeated, distinct, repeated];

    let (first, first_report) = register_font_handle_batch_with_report(&pairs, generation);
    let (second, second_report) = register_font_handle_batch_with_report(&pairs, generation);

    assert_eq!(first, second);
    assert_eq!(first_report.registration_batch_count, 1);
    assert_eq!(first_report.registration_lock_acquire_count, 1);
    assert_eq!(first_report.registration_unique_pair_count, 2);
    assert_eq!(first_report.registration_rejected_pair_count, 0);
    assert_eq!(first_report.registration_snapshot_publish_count, 1);
    assert_eq!(second_report.registration_batch_count, 1);
    assert_eq!(second_report.registration_lock_acquire_count, 1);
    assert_eq!(second_report.registration_unique_pair_count, 2);
    assert_eq!(second_report.registration_rejected_pair_count, 0);
    assert_eq!(second_report.registration_snapshot_publish_count, 0);
}

#[test]
fn repeated_font_handle_batch_does_not_republish_an_unchanged_snapshot() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let (_, database) = shared_font_database_snapshot();
    let generation = force_publish_shared_font_database(&database);
    let pairs = [(Some(FontFaceId(7_001)), Some(InstancedFaceId(8_001)))];

    let before = font_handle_registry_report();
    let first = register_font_handle_batch(&pairs, generation);
    let after_first = font_handle_registry_report();
    let second = register_font_handle_batch(&pairs, generation);
    let after_second = font_handle_registry_report();

    assert_eq!(first, second);
    assert_eq!(
        after_first.registration_snapshot_publish_count,
        before.registration_snapshot_publish_count + 1
    );
    assert_eq!(
        after_second.registration_snapshot_publish_count,
        after_first.registration_snapshot_publish_count
    );
    assert_eq!(
        after_second.registration_lock_acquire_count,
        after_first.registration_lock_acquire_count + 1
    );
}
