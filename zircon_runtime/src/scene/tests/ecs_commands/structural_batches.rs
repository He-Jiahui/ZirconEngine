use super::*;

#[test]
fn deferred_command_success_report_counts_applied_commands_without_errors() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Target".to_string()),)).unwrap();

    {
        let mut commands = world.commands();
        commands.insert(entity, Health(7));
        commands.entity(entity).insert((Marker,));
    }

    let report = world.apply_deferred();

    assert_eq!(report.applied_count(), 2);
    assert_eq!(report.error_count(), 0);
    assert!(report.is_success());
    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
}

#[test]
fn command_queue_on_despawned_entity_target_is_reported_not_silently_dropped() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Removed".to_string()), Health(1)))
        .unwrap();
    world.remove_entity(entity).unwrap();

    {
        let mut commands = world.commands();
        commands.insert(entity, Health(2));
        commands.remove::<Health>(entity);
        commands.despawn(entity);
    }

    let report = world.apply_deferred();
    let errors = report.errors();

    assert_eq!(report.applied_count(), 3);
    assert_eq!(report.error_count(), 3);
    assert!(!report.is_success());
    assert_eq!(errors[0].operation(), DeferredCommandOperation::Insert);
    assert_eq!(errors[0].target(), &DeferredCommandTarget::Resolved(entity));
    assert!(matches!(
        errors[0].error(),
        crate::scene::SceneError::MissingEntity { entity: missing, .. } if *missing == entity
    ));
    assert_eq!(errors[1].operation(), DeferredCommandOperation::Remove);
    assert_eq!(errors[1].target(), &DeferredCommandTarget::Resolved(entity));
    assert!(matches!(
        errors[1].error(),
        crate::scene::SceneError::MissingEntity { entity: missing, .. } if *missing == entity
    ));
    assert_eq!(errors[2].operation(), DeferredCommandOperation::Despawn);
    assert_eq!(errors[2].target(), &DeferredCommandTarget::Resolved(entity));
    assert!(matches!(
        errors[2].error(),
        crate::scene::SceneError::MissingEntity { entity: missing, .. } if *missing == entity
    ));
    assert!(!world.has_deferred_commands());
    assert!(world.get::<Health>(entity).is_none());
}

#[test]
fn deferred_command_report_resolves_published_spawns_only_after_the_barrier() {
    let mut world = World::empty();
    let spawned = world.commands().spawn_empty().into_deferred_entity();

    let report = world.apply_deferred();
    assert_eq!(report.applied_count(), 1);
    assert_eq!(report.error_count(), 0);
    let entity = report
        .resolve(&spawned)
        .expect("published spawn must expose a barrier resolution");
    assert!(world.contains_entity(entity));
}

#[test]
fn stale_worker_deferred_entity_never_aliases_a_later_compiled_lane_run() {
    let mut world = World::empty();
    let key = DeferredSystemKey::compiled(3, 17, "tests.stale_worker_handle");
    let mut worker = WorkerCommandBuffer::with_capacity(17, "tests.stale_worker_handle", 1);

    worker.bind_compiled_key(key.clone());
    worker.begin_run();
    let first = worker
        .commands()
        .spawn((Name("first worker spawn".to_string()),))
        .into_deferred_entity();
    world.merge_worker_command_buffer(&mut worker);
    let first_report = world.apply_deferred();
    world.reclaim_worker_command_buffer(&mut worker);
    let first_entity = first_report
        .resolve(&first)
        .expect("first worker spawn must resolve at its barrier");

    worker.bind_compiled_key(key);
    worker.begin_run();
    let second = worker
        .commands()
        .spawn((Name("second worker spawn".to_string()),))
        .into_deferred_entity();
    world.merge_worker_command_buffer(&mut worker);
    let second_report = world.apply_deferred();
    world.reclaim_worker_command_buffer(&mut worker);
    let second_entity = second_report
        .resolve(&second)
        .expect("second worker spawn must resolve at its barrier");
    assert_ne!(first_entity, second_entity);

    world.commands().entity_deferred(&first).insert((Marker,));
    let stale_report = world.apply_deferred();
    assert_eq!(stale_report.error_count(), 1);
    assert!(world.get::<Marker>(first_entity).is_none());
    assert!(world.get::<Marker>(second_entity).is_none());

    let next_entity = world.spawn(()).expect("next synchronous spawn");
    assert_eq!(next_entity, second_entity + 1);
}

#[test]
fn opaque_next_window_cannot_reuse_a_resolved_deferred_entity_handle() {
    let mut world = World::empty();
    let spawned = world.commands().spawn_empty().into_deferred_entity();
    let report_handle = spawned.clone();

    world.commands().queue_fn(move |world| {
        world.commands().entity_deferred(&spawned).insert((Marker,));
    });

    let first_report = world.apply_deferred();
    let entity = first_report
        .resolve(&report_handle)
        .expect("the original spawn must resolve in its own apply window");
    assert!(world.contains_entity(entity));
    assert!(world.has_deferred_commands());

    let next_report = world.apply_deferred();
    assert_eq!(next_report.error_count(), 1);
    assert!(matches!(
        next_report.errors()[0].target(),
        DeferredCommandTarget::Pending(_)
    ));
    assert!(world.get::<Marker>(entity).is_none());
}

#[test]
fn deferred_entity_builder_publishes_one_final_bundle_transaction() {
    let mut world = World::empty();
    world.reset_ecs_frame_performance_diagnostics();

    let spawned = {
        let mut commands = world.commands();
        let mut entity = commands.spawn((Name("Final row".to_string()),));
        entity.insert((Health(42), Marker));
        entity.into_deferred_entity()
    };

    let report = world.apply_deferred();
    let entity = report
        .resolve(&spawned)
        .expect("the final-row spawn must resolve at the barrier");
    let diagnostics = world.ecs_frame_performance_diagnostics();

    assert_eq!(world.get::<Health>(entity), Some(&Health(42)));
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
    assert_eq!(diagnostics.bundle_transactions.committed_transactions, 1);
    assert_eq!(
        diagnostics.bundle_transactions.final_archetype_transitions,
        1
    );
    assert_eq!(diagnostics.bundle_transactions.intermediate_signatures, 0);
}

#[test]
fn deferred_final_row_segment_finishes_before_an_opaque_command_barrier() {
    let mut world = World::empty();
    world.reset_ecs_frame_performance_diagnostics();

    let spawned = {
        let mut commands = world.commands();
        let mut entity = commands.spawn((Name("Barrier row".to_string()),));
        entity.insert((Health(7), Marker));
        let spawned = entity.into_deferred_entity();
        commands.queue_fn(|world| {
            let transactions = world
                .ecs_frame_performance_diagnostics()
                .bundle_transactions
                .committed_transactions;
            world.insert_resource(DeferredBarrierObservation(
                usize::try_from(transactions).expect("transaction count must fit usize"),
            ));
        });
        spawned
    };

    let report = world.apply_deferred();
    let entity = report
        .resolve(&spawned)
        .expect("the structural segment must publish before the opaque command");

    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
    assert_eq!(
        world.get_resource::<DeferredBarrierObservation>(),
        Some(&DeferredBarrierObservation(1))
    );
}

#[test]
fn deferred_final_row_segment_keeps_opaque_nested_commands_in_the_next_window() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Nested barrier target".to_string()),))
        .expect("fixture entity must spawn");

    {
        let mut commands = world.commands();
        commands.insert(entity, Marker);
        commands.queue_fn(move |world| {
            world.commands().insert(entity, Health(9));
        });
    }

    let first = world.apply_deferred();

    assert!(first.is_success());
    assert_eq!(first.applied_count(), 2);
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
    assert!(world.get::<Health>(entity).is_none());
    assert!(world.has_deferred_commands());

    let second = world.apply_deferred();

    assert!(second.is_success());
    assert_eq!(second.applied_count(), 1);
    assert_eq!(world.get::<Health>(entity), Some(&Health(9)));
}

#[test]
fn deferred_final_row_segment_hides_insert_then_remove_lifecycle() {
    let mut world = World::empty();
    let health_events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let marker_events = Arc::new(std::sync::Mutex::new(Vec::new()));

    for kind in [
        LifecycleEventKind::Add,
        LifecycleEventKind::Insert,
        LifecycleEventKind::Remove,
    ] {
        let health_events = Arc::clone(&health_events);
        world.observe_component_lifecycle::<Health>(kind, move |_world, _event| {
            health_events
                .lock()
                .expect("health lifecycle events")
                .push(kind);
        });
    }
    for kind in [LifecycleEventKind::Add, LifecycleEventKind::Insert] {
        let marker_events = Arc::clone(&marker_events);
        world.observe_component_lifecycle::<Marker>(kind, move |_world, _event| {
            marker_events
                .lock()
                .expect("marker lifecycle events")
                .push(kind);
        });
    }
    world.reset_ecs_frame_performance_diagnostics();

    let spawned = {
        let mut commands = world.commands();
        let mut entity = commands.spawn_empty();
        entity.insert((Health(7), Marker));
        entity.remove::<Health>();
        entity.into_deferred_entity()
    };

    let report = world.apply_deferred();
    let entity = report
        .resolve(&spawned)
        .expect("the final-row spawn must resolve at the barrier");
    let diagnostics = world.ecs_frame_performance_diagnostics();

    assert!(world.get::<Health>(entity).is_none());
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
    assert!(
        health_events
            .lock()
            .expect("health lifecycle events")
            .is_empty()
    );
    assert_eq!(
        marker_events
            .lock()
            .expect("marker lifecycle events")
            .as_slice(),
        &[LifecycleEventKind::Add, LifecycleEventKind::Insert]
    );
    assert_eq!(diagnostics.bundle_transactions.committed_transactions, 1);
    assert_eq!(
        diagnostics.bundle_transactions.final_archetype_transitions,
        1
    );
    assert_eq!(diagnostics.bundle_transactions.intermediate_signatures, 0);
}

#[test]
fn deferred_final_row_segment_aborts_until_an_opaque_barrier_after_staging_failure() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Failed segment".to_string()), Health(1)))
        .expect("fixture entity must spawn");
    world.reset_ecs_frame_performance_diagnostics();

    {
        let mut commands = world.commands();
        commands.insert_bundle(entity, (Health(2), Health(3)));
        commands.insert(entity, Marker);
        commands.queue_fn(move |world| {
            let marker_is_visible = if world.contains_component::<Marker>(entity) {
                1
            } else {
                0
            };
            world.insert_resource(DeferredBarrierObservation(marker_is_visible));
        });
        commands.insert(entity, Marker);
    }

    let report = world.apply_deferred();
    let diagnostics = world.ecs_frame_performance_diagnostics();

    assert_eq!(report.applied_count(), 4);
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].operation(),
        DeferredCommandOperation::InsertBundle
    );
    assert!(matches!(
        report.errors()[0].error(),
        crate::scene::SceneError::DuplicateBundleComponentType
    ));
    assert_eq!(world.get::<Health>(entity), Some(&Health(1)));
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
    assert_eq!(
        world.get_resource::<DeferredBarrierObservation>(),
        Some(&DeferredBarrierObservation(0))
    );
    assert_eq!(diagnostics.bundle_transactions.committed_transactions, 1);
    assert_eq!(diagnostics.bundle_transactions.intermediate_signatures, 0);
}

#[test]
fn deferred_structural_batch_aborts_all_targets_before_an_opaque_barrier() {
    let mut world = World::empty();
    let first = world
        .spawn((Name("First failed batch target".to_string()), Health(1)))
        .expect("first fixture entity must spawn");
    let second = world
        .spawn((Name("Second failed batch target".to_string()), Health(2)))
        .expect("second fixture entity must spawn");
    world.reset_ecs_frame_performance_diagnostics();

    {
        let mut commands = world.commands();
        commands.insert(first, Marker);
        commands.insert_bundle(second, (Health(3), Health(4)));
        commands.queue_fn(move |world| {
            let first_marker_is_visible = if world.contains_component::<Marker>(first) {
                1
            } else {
                0
            };
            world.insert_resource(DeferredBarrierObservation(first_marker_is_visible));
        });
    }

    let report = world.apply_deferred();
    let diagnostics = world.ecs_frame_performance_diagnostics();

    assert_eq!(report.applied_count(), 3);
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].operation(),
        DeferredCommandOperation::InsertBundle
    );
    assert!(matches!(
        report.errors()[0].error(),
        crate::scene::SceneError::DuplicateBundleComponentType
    ));
    assert!(world.get::<Marker>(first).is_none());
    assert_eq!(world.get::<Health>(first), Some(&Health(1)));
    assert_eq!(world.get::<Health>(second), Some(&Health(2)));
    assert_eq!(
        world.get_resource::<DeferredBarrierObservation>(),
        Some(&DeferredBarrierObservation(0))
    );
    assert_eq!(diagnostics.bundle_transactions.committed_transactions, 0);
    assert_eq!(diagnostics.bundle_transactions.intermediate_signatures, 0);
}

#[test]
fn deferred_structural_batch_rejects_parent_links_to_a_pending_despawn() {
    let mut world = World::empty();
    let parent = world
        .spawn((Name("Pending despawn parent".to_string()),))
        .expect("parent fixture must spawn");
    let child = world
        .spawn((Name("Pending despawn child".to_string()),))
        .expect("child fixture must spawn");
    world.reset_ecs_frame_performance_diagnostics();

    {
        let mut commands = world.commands();
        commands.despawn(parent);
        commands.insert(
            child,
            Hierarchy {
                parent: Some(parent),
            },
        );
    }

    let report = world.apply_deferred();
    let diagnostics = world.ecs_frame_performance_diagnostics();

    assert_eq!(report.applied_count(), 2);
    assert_eq!(report.error_count(), 1);
    assert_eq!(
        report.errors()[0].operation(),
        DeferredCommandOperation::Insert
    );
    assert!(matches!(
        report.errors()[0].error(),
        crate::scene::SceneError::MissingParent {
            child: error_child,
            parent: error_parent,
        } if *error_child == child && *error_parent == parent
    ));
    assert!(world.contains_entity(parent));
    assert_eq!(world.parent_of(child), None);
    assert_eq!(diagnostics.bundle_transactions.committed_transactions, 0);
}

#[test]
fn deferred_final_row_segment_cancels_spawn_then_despawn_without_publication() {
    let mut world = World::empty();
    let before = world
        .spawn(())
        .expect("fixture entity must establish the allocator baseline");
    let marker_events = Arc::new(std::sync::Mutex::new(Vec::new()));
    for kind in [
        LifecycleEventKind::Add,
        LifecycleEventKind::Insert,
        LifecycleEventKind::Remove,
        LifecycleEventKind::Despawn,
    ] {
        let marker_events = Arc::clone(&marker_events);
        world.observe_component_lifecycle::<Marker>(kind, move |_world, _event| {
            marker_events
                .lock()
                .expect("marker lifecycle events")
                .push(kind);
        });
    }
    world.reset_ecs_frame_performance_diagnostics();

    let spawned = {
        let mut commands = world.commands();
        let mut entity = commands.spawn_empty();
        entity.insert((Marker,));
        entity.despawn();
        entity.into_deferred_entity()
    };

    let report = world.apply_deferred();
    let diagnostics = world.ecs_frame_performance_diagnostics();

    assert!(report.is_success());
    assert!(report.resolve(&spawned).is_none());
    assert!(
        marker_events
            .lock()
            .expect("marker lifecycle events")
            .is_empty()
    );
    assert_eq!(diagnostics.bundle_transactions.committed_transactions, 0);
    assert_eq!(
        diagnostics.bundle_transactions.final_archetype_transitions,
        0
    );
    assert_eq!(diagnostics.bundle_transactions.intermediate_signatures, 0);
    let after = world
        .spawn(())
        .expect("the next live entity must consume a fresh id");
    assert_eq!(after, before + 2);
}

#[test]
fn deferred_final_row_segment_despawn_discards_pending_insert() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Despawn target".to_string()), Health(3)))
        .expect("fixture entity must spawn");
    let marker_events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let health_events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed_removed_entity = Arc::new(std::sync::Mutex::new(false));
    for kind in [
        LifecycleEventKind::Add,
        LifecycleEventKind::Insert,
        LifecycleEventKind::Remove,
        LifecycleEventKind::Despawn,
    ] {
        let marker_events = Arc::clone(&marker_events);
        world.observe_component_lifecycle::<Marker>(kind, move |_world, _event| {
            marker_events
                .lock()
                .expect("marker lifecycle events")
                .push(kind);
        });
        let health_events = Arc::clone(&health_events);
        world.observe_component_lifecycle::<Health>(kind, move |_world, _event| {
            health_events
                .lock()
                .expect("health lifecycle events")
                .push(kind);
        });
    }
    let observed_removed_entity_for_remove = Arc::clone(&observed_removed_entity);
    world.observe_component_lifecycle::<Health>(
        LifecycleEventKind::Remove,
        move |world, _event| {
            *observed_removed_entity_for_remove
                .lock()
                .expect("removed entity observation") = !world.contains_entity(entity);
        },
    );
    world.reset_ecs_frame_performance_diagnostics();

    {
        let mut commands = world.commands();
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((Marker,));
        entity_commands.despawn();
    }

    let report = world.apply_deferred();
    let diagnostics = world.ecs_frame_performance_diagnostics();

    assert!(report.is_success());
    assert!(!world.contains_entity(entity));
    assert!(
        marker_events
            .lock()
            .expect("marker lifecycle events")
            .is_empty()
    );
    assert_eq!(
        health_events
            .lock()
            .expect("health lifecycle events")
            .as_slice(),
        &[LifecycleEventKind::Remove, LifecycleEventKind::Despawn]
    );
    assert!(
        *observed_removed_entity
            .lock()
            .expect("removed entity observation")
    );
    assert_eq!(diagnostics.bundle_transactions.committed_transactions, 0);
    assert_eq!(
        diagnostics.bundle_transactions.final_archetype_transitions,
        0
    );
    assert_eq!(diagnostics.bundle_transactions.intermediate_signatures, 0);
}

#[test]
fn deferred_final_row_segment_despawn_detaches_existing_children() {
    let mut world = World::empty();
    let parent = world
        .spawn((Name("Deferred parent".to_string()),))
        .expect("fixture parent must spawn");
    let child = world
        .spawn((Name("Deferred child".to_string()),))
        .expect("fixture child must spawn");
    world
        .set_parent_checked(child, Some(parent))
        .expect("fixture parent assignment must succeed");

    {
        let mut commands = world.commands();
        commands.despawn(parent);
    }

    let report = world.apply_deferred();

    assert!(report.is_success());
    assert!(!world.contains_entity(parent));
    assert!(world.contains_entity(child));
    assert_eq!(world.parent_of(child), None);
}

#[test]
fn deferred_final_row_segment_publishes_existing_sparse_removal_once() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Sparse target".to_string()), SparseMarker))
        .expect("fixture entity must spawn");
    let sparse_events = Arc::new(std::sync::Mutex::new(Vec::new()));
    for kind in [
        LifecycleEventKind::Add,
        LifecycleEventKind::Insert,
        LifecycleEventKind::Remove,
    ] {
        let sparse_events = Arc::clone(&sparse_events);
        world.observe_component_lifecycle::<SparseMarker>(kind, move |_world, _event| {
            sparse_events
                .lock()
                .expect("sparse lifecycle events")
                .push(kind);
        });
    }
    world.reset_ecs_frame_performance_diagnostics();

    {
        let mut commands = world.commands();
        let mut entity_commands = commands.entity(entity);
        entity_commands.insert((Marker,));
        entity_commands.remove::<SparseMarker>();
    }

    let report = world.apply_deferred();
    let diagnostics = world.ecs_frame_performance_diagnostics();

    assert!(report.is_success());
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
    assert!(world.get::<SparseMarker>(entity).is_none());
    assert_eq!(
        sparse_events
            .lock()
            .expect("sparse lifecycle events")
            .as_slice(),
        &[LifecycleEventKind::Remove]
    );
    assert_eq!(diagnostics.bundle_transactions.committed_transactions, 1);
    assert_eq!(
        diagnostics.bundle_transactions.final_archetype_transitions,
        1
    );
    assert_eq!(diagnostics.bundle_transactions.intermediate_signatures, 0);
}
