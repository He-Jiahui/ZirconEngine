use crate::scene::components::Name;
use crate::scene::ecs::Mut;
use crate::scene::{DetachedEntityBatchRemoveError, EntityId, NodeKind, SceneError, World};

fn world_facts(world: &World) -> (serde_json::Value, u64, u64, usize) {
    (
        serde_json::to_value(world).unwrap(),
        world.world_generation(),
        world.lifecycle_visibility_revision(),
        world.staged_lifecycle_events.len(),
    )
}

#[test]
fn preparation_normalizes_roots_and_counts_only_affected_cameras() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let camera = world.spawn_node(NodeKind::Camera).unwrap();
    let child = world.spawn_node(NodeKind::Empty).unwrap();
    let survivor = world.spawn_node(NodeKind::Camera).unwrap();
    world.set_parent_checked(camera, Some(root)).unwrap();
    world.set_parent_checked(child, Some(camera)).unwrap();
    world.set_active_camera(camera);
    let before = world_facts(&world);

    let prepared = world
        .prepare_entity_subtrees([child, camera, root, root])
        .unwrap();
    assert_eq!(prepared.normalized_roots(), [root]);
    assert_eq!(prepared.affected_entity_count(), 3);
    assert_eq!(prepared.affected_camera_count(), 1);
    assert_eq!(prepared.world_camera_count(), 2);
    assert_eq!(prepared.world_generation(), before.1);
    assert_eq!(world_facts(&world), before);

    let batch = world.remove_prepared_entity_subtrees(prepared).unwrap();
    assert_eq!(
        batch.entity_ids().collect::<Vec<_>>(),
        [root, camera, child]
    );
    assert_eq!(world.camera_count(), 1);
    assert_eq!(world.active_camera(), survivor);
    world.restore_detached_entity_batch(batch).unwrap();
    assert_eq!(world.camera_count(), 2);
    assert_eq!(world.active_camera(), camera);
    assert_eq!(world.parent_of(child), Some(camera));
}

#[test]
fn stale_preparation_rejects_camera_reparenting_without_detaching_rows() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let camera = world.spawn_node(NodeKind::Camera).unwrap();
    let prepared = world.prepare_entity_subtrees([root]).unwrap();
    assert_eq!(prepared.affected_camera_count(), 0);
    let prepared_generation = prepared.world_generation();
    world.set_parent_checked(camera, Some(root)).unwrap();
    let before = world_facts(&world);

    assert!(matches!(
        world.remove_prepared_entity_subtrees(prepared),
        Err(DetachedEntityBatchRemoveError::Rejected(SceneError::DetachedEntityPreparationStale {
            prepared_generation: captured,
            current_generation,
        })) if captured == prepared_generation && current_generation == before.1
    ));
    assert_eq!(world_facts(&world), before);
    let diagnostics = world
        .ecs_frame_performance_diagnostics()
        .detached_entity_batches;
    assert_eq!(diagnostics.commit_count, 0);
    assert_eq!(diagnostics.lifecycle_events, 0);
    assert_eq!(diagnostics.rejected_preflights, 1);

    let refreshed = world.prepare_entity_subtrees([root]).unwrap();
    assert_eq!(refreshed.affected_camera_count(), 1);
    assert_eq!(refreshed.world_camera_count(), 1);
    drop(refreshed);
    assert_eq!(world_facts(&world), before);
}

#[test]
fn preparation_rejects_unflushed_component_mutations() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let prepared = world.prepare_entity_subtrees([root]).unwrap();
    let mut query = world.query::<Mut<'static, Name>>();
    query.get_mut(&mut world, root).unwrap().0 = "Changed after prepare".to_owned();
    let before = world_facts(&world);

    assert!(matches!(
        world.remove_prepared_entity_subtrees(prepared),
        Err(DetachedEntityBatchRemoveError::Rejected(
            SceneError::DetachedEntityPreparationStale { .. }
        ))
    ));
    assert_eq!(world_facts(&world), before);
}

#[test]
fn preparation_cannot_cross_world_clone_or_persistence_boundaries() {
    let mut source = World::empty();
    let root = source.spawn_node(NodeKind::Empty).unwrap();
    let decoded = serde_json::from_value(serde_json::to_value(&source).unwrap()).unwrap();
    let mut independent = World::empty();
    assert_eq!(independent.spawn_node(NodeKind::Empty).unwrap(), root);

    for mut target in [source.clone(), decoded, independent] {
        // Equal ids and revisions must not grant access to another World's rows.
        target.world_generation = source.world_generation;
        assert_eq!(target.world_generation(), source.world_generation());
        let prepared = source.prepare_entity_subtrees([root]).unwrap();
        let before = world_facts(&target);
        assert!(matches!(
            target.remove_prepared_entity_subtrees(prepared),
            Err(DetachedEntityBatchRemoveError::Rejected(
                SceneError::DetachedEntityPreparationWorldMismatch
            ))
        ));
        assert_eq!(world_facts(&target), before);
        assert!(source.contains_entity(root));
    }
}

#[test]
fn preparation_survives_world_move_but_not_an_earlier_detach() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let first = world.prepare_entity_subtrees([root]).unwrap();
    let second = world.prepare_entity_subtrees([root]).unwrap();
    let mut moved = Box::new(world);

    let batch = moved.remove_prepared_entity_subtrees(first).unwrap();
    let before = world_facts(&moved);
    assert!(matches!(
        moved.remove_prepared_entity_subtrees(second),
        Err(DetachedEntityBatchRemoveError::Rejected(
            SceneError::DetachedEntityPreparationStale { .. }
        ))
    ));
    assert_eq!(world_facts(&moved), before);
    moved.restore_detached_entity_batch(batch).unwrap();
    assert!(moved.contains_entity(root));
}

#[test]
fn preparation_includes_unflushed_hierarchy_changes_in_camera_counts() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let camera = world.spawn_node(NodeKind::Camera).unwrap();
    assert!(world.corrupt_hierarchy_parent_with_pending_mutation_for_tests(camera, Some(root)));
    let before = world_facts(&world);

    let prepared = world.prepare_entity_subtrees([root]).unwrap();
    assert_eq!(prepared.affected_entity_count(), 2);
    assert_eq!(prepared.affected_camera_count(), 1);
    assert_eq!(prepared.world_camera_count(), 1);
    assert_eq!(world_facts(&world), before);
}

#[test]
fn exhausted_world_generation_cannot_issue_a_preparation() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    world.world_generation = world.world_generation.advanced_by(u64::MAX);
    let before = world_facts(&world);

    assert!(matches!(
        world.prepare_entity_subtrees([root]),
        Err(SceneError::DetachedEntityPreparationGenerationExhausted)
    ));
    assert_eq!(world_facts(&world), before);
}

#[test]
#[ignore = "managed Runtime08 2/128 camera subtree with 100k unrelated entity gate"]
fn prepared_camera_subtree_managed_scale_fixture() {
    const UNRELATED_ENTITIES: EntityId = 100_000;
    for camera_count in [2, 128] {
        let mut world = World::empty();
        let root = world.spawn_node(NodeKind::Empty).unwrap();
        for _ in 0..camera_count {
            let camera = world.spawn_node(NodeKind::Camera).unwrap();
            world.set_parent_checked(camera, Some(root)).unwrap();
        }
        for offset in 0..UNRELATED_ENTITIES {
            world.spawn_empty_at(1_000_000 + offset).unwrap();
        }
        world.reset_ecs_frame_performance_diagnostics();
        let before = world_facts(&world);
        let prepared = world.prepare_entity_subtrees([root]).unwrap();
        assert_eq!(prepared.normalized_roots(), [root]);
        assert_eq!(prepared.affected_entity_count(), camera_count + 1);
        assert_eq!(prepared.affected_camera_count(), camera_count);
        assert_eq!(prepared.world_camera_count(), camera_count);
        drop(prepared);
        assert_eq!(world_facts(&world), before);
        let diagnostics = world
            .ecs_frame_performance_diagnostics()
            .detached_entity_batches;
        assert_eq!(diagnostics.commit_count, 0);
        assert_eq!(diagnostics.lifecycle_events, 0);
        assert_eq!(diagnostics.full_world_clone_bytes, 0);
        assert_eq!(diagnostics.node_record_clone_bytes, 0);
        assert_eq!(diagnostics.rollback_bytes, 0);
    }
}

mod lifecycle_cancellation {
    use super::*;
    use crate::scene::ecs::{
        Component, ComponentLifecycleEvent, DeferredCommandError, DeferredCommandOperation,
        DeferredCommandTarget, LifecycleEventKind, Resource, StorageType, SystemStage,
    };
    use crate::scene::{DetachedEntityBatchRemoveError, DetachedEntityBatchRestoreError};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };
    use zircon_runtime_interface::world_sync::LifecycleTargetChange;

    #[derive(Debug, PartialEq, Eq)]
    struct OwnedTable(Box<str>);
    zircon_runtime_interface::impl_retained_owner_v1!(OwnedTable { 0 });
    impl Component for OwnedTable {}
    #[derive(Debug, PartialEq, Eq)]
    struct OwnedSparse(u64);
    zircon_runtime_interface::impl_retained_owner_v1!(OwnedSparse { 0 });
    impl Component for OwnedSparse {
        const STORAGE_TYPE: StorageType = StorageType::SparseSet;
    }
    #[derive(Debug, PartialEq, Eq)]
    struct DetachedSentinel(u32);
    impl Resource for DetachedSentinel {}

    macro_rules! observe_rows {
        ($world:expr, $kinds:expr, $callback:expr) => {{
            for kind in $kinds {
                // SAFETY: All four callers of this local macro are audited below:
                // cancellation_case only reads rows, records external counters, replaces
                // the World/Schedule and installs the sentinel (resource/diagnostics/queue);
                // its queued sentinel body only increments an external counter. The other
                // callers count events or register one schedule body that only counts.
                // No caller changes component/dynamic payload or recursively dispatches;
                // replacement ownership interrupts replay before adopting its existing rows.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    $world.observe_component_lifecycle::<OwnedTable>(
                        kind,
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            $callback.clone(),
                        ),
                    );
                    $world.observe_component_lifecycle::<OwnedSparse>(
                        kind,
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            $callback.clone(),
                        ),
                    );
                }
            }
        }};
    }

    fn fixture(fresh: bool) -> World {
        let mut world = World::empty();
        world.component_id::<OwnedTable>();
        world.component_id::<OwnedSparse>();
        for id in [1, 2, 90] {
            world
                .spawn_at(
                    id,
                    (
                        OwnedTable(
                            if fresh {
                                format!("fresh-{id}")
                            } else {
                                format!("owned-{id}")
                            }
                            .into_boxed_str(),
                        ),
                        OwnedSparse(if fresh { id + 100 } else { id + 10 }),
                    ),
                )
                .unwrap();
        }
        world
    }

    fn install_sentinel(world: &mut World, calls: Arc<AtomicUsize>) {
        world.insert_resource(DetachedSentinel(87));
        world.record_deferred_command_error(DeferredCommandError::new(
            DeferredCommandOperation::Remove,
            DeferredCommandTarget::Resolved(1),
            SceneError::Message("detached replacement diagnostics".into()),
        ));
        world.commands().queue_fn(move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
        });
        world.reset_ecs_frame_performance_diagnostics();
    }

    fn cancellation_case(restoring: bool, mode: u32, final_callback: bool) {
        let mut world = fixture(false);
        let ticks = [
            (
                world.component_change_ticks::<OwnedTable>(1),
                world.component_change_ticks::<OwnedSparse>(1),
            ),
            (
                world.component_change_ticks::<OwnedTable>(2),
                world.component_change_ticks::<OwnedSparse>(2),
            ),
        ];
        let mut batch = restoring.then(|| world.remove_entity_subtrees([1, 2]).unwrap());
        world.reset_ecs_frame_performance_diagnostics();
        let original_owner = world.owner_identity().clone();
        let original_plan = world.schedule().stage_plan();
        let boundary = if final_callback { 8 } else { 1 };
        let expected = if mode == 2 {
            LifecycleTargetChange::Schedule
        } else {
            LifecycleTargetChange::World
        };
        let callbacks = Arc::new(AtomicUsize::new(0));
        let leaked = Arc::new(AtomicUsize::new(0));
        let fresh_calls = Arc::new(AtomicUsize::new(0));
        let generation = Arc::new(Mutex::new(None));
        let kinds = if restoring {
            [LifecycleEventKind::Add, LifecycleEventKind::Insert]
        } else {
            [LifecycleEventKind::Remove, LifecycleEventKind::Despawn]
        };
        let mut fresh = fixture(true);
        assert_eq!(
            world.component_id::<OwnedTable>(),
            fresh.component_id::<OwnedTable>()
        );
        assert_eq!(
            world.component_id::<OwnedSparse>(),
            fresh.component_id::<OwnedSparse>()
        );
        let leaked_callback = {
            let leaked = Arc::clone(&leaked);
            move |_: &mut World, _: &ComponentLifecycleEvent| {
                leaked.fetch_add(1, Ordering::SeqCst);
            }
        };
        observe_rows!(fresh, kinds, leaked_callback);
        install_sentinel(&mut fresh, Arc::clone(&fresh_calls));
        let fresh = Arc::new(Mutex::new(Some(fresh)));
        let callback = {
            let callbacks = Arc::clone(&callbacks);
            let fresh_calls = Arc::clone(&fresh_calls);
            let generation = Arc::clone(&generation);
            let original_plan = Arc::clone(&original_plan);
            move |world: &mut World, _: &ComponentLifecycleEvent| {
                assert_eq!(world.contains_entity(1), restoring);
                assert_eq!(world.contains_entity(2), restoring);
                if restoring {
                    assert_eq!(
                        world.get::<OwnedTable>(1),
                        Some(&OwnedTable("owned-1".into()))
                    );
                    assert_eq!(world.get::<OwnedSparse>(2), Some(&OwnedSparse(12)));
                }
                if callbacks.fetch_add(1, Ordering::SeqCst) + 1 == boundary {
                    if mode == 2 {
                        *world.schedule_mut() = Default::default();
                        install_sentinel(world, Arc::clone(&fresh_calls));
                    } else {
                        let mut replacement = fresh.lock().unwrap().take().unwrap();
                        if mode == 1 {
                            *replacement.schedule_mut() = std::mem::take(world.schedule_mut());
                            assert!(Arc::ptr_eq(
                                &original_plan,
                                &replacement.schedule().stage_plan()
                            ));
                        }
                        *world = replacement;
                    }
                    *generation.lock().unwrap() = Some(world.world_generation());
                }
            }
        };
        observe_rows!(world, kinds, callback);
        let (error, retained) = if restoring {
            let error = world
                .restore_detached_entity_batch(batch.take().unwrap())
                .unwrap_err();
            assert!(matches!(
                &error,
                DetachedEntityBatchRestoreError::Cancelled { .. }
            ));
            error.into_parts()
        } else {
            let prepared = world.prepare_entity_subtrees([1, 2]).unwrap();
            let error = match world.remove_prepared_entity_subtrees(prepared) {
                Err(error) => error,
                Ok(_) => panic!("replacement during remove replay must return cancellation"),
            };
            assert!(matches!(
                &error,
                DetachedEntityBatchRemoveError::Cancelled { .. }
            ));
            error.into_parts()
        };
        assert_eq!(error.lifecycle_target_change(), Some(expected));
        assert_eq!(callbacks.load(Ordering::SeqCst), boundary);
        assert_eq!(leaked.load(Ordering::SeqCst), 0);
        assert_eq!(
            world.world_generation(),
            generation.lock().unwrap().unwrap()
        );
        assert_eq!(
            world.get_resource::<DetachedSentinel>(),
            Some(&DetachedSentinel(87))
        );
        assert_eq!(
            original_owner.is_same_world(world.owner_identity()),
            mode == 2
        );
        assert_eq!(
            world.get::<OwnedTable>(90),
            Some(&OwnedTable(
                if mode == 2 { "owned-90" } else { "fresh-90" }.into()
            ))
        );
        if mode != 2 {
            assert_eq!(
                world.get::<OwnedTable>(1),
                Some(&OwnedTable("fresh-1".into()))
            );
            assert_eq!(world.get::<OwnedSparse>(2), Some(&OwnedSparse(102)));
        } else {
            assert_eq!(world.contains_entity(1), restoring);
            assert_eq!(world.contains_entity(2), restoring);
        }
        let stats = world
            .ecs_frame_performance_diagnostics()
            .detached_entity_batches;
        assert_eq!(stats.commit_count, 0);
        assert_eq!(stats.moved_rows, 0);
        assert_eq!(stats.lifecycle_events, 0);
        assert_eq!(stats.generation_advances, 0);
        let diagnostics = world.take_deferred_command_errors();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].error(),
            &SceneError::Message("detached replacement diagnostics".into())
        );
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
        assert!(world.apply_deferred().is_success());
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
        if restoring {
            assert!(
                retained.is_none(),
                "post-publication restore cancellation cannot return consumed rows"
            );
        } else {
            let retained = retained.expect("cancelled removal still owns every detached row");
            assert_eq!(retained.entity_ids().collect::<Vec<_>>(), [1, 2]);
            // Compatible cross-World restore remains supported, with exact owned
            // values and ticks. Never restore the payload into the replacement.
            let mut compatible = fixture(true);
            let seed = compatible.remove_entity_subtrees([1, 2]).unwrap();
            drop(seed);
            compatible.restore_detached_entity_batch(retained).unwrap();
            for (index, id) in [1, 2].into_iter().enumerate() {
                assert_eq!(
                    compatible.get::<OwnedTable>(id),
                    Some(&OwnedTable(format!("owned-{id}").into_boxed_str()))
                );
                assert_eq!(
                    compatible.get::<OwnedSparse>(id),
                    Some(&OwnedSparse(id + 10))
                );
                assert_eq!(
                    compatible.component_change_ticks::<OwnedTable>(id),
                    ticks[index].0
                );
                assert_eq!(
                    compatible.component_change_ticks::<OwnedSparse>(id),
                    ticks[index].1
                );
            }
        }
    }

    #[test]
    fn detached_remove_cancellation_retains_owned_rows_and_skips_replacement_tail_and_statistics() {
        for mode in 0..3 {
            for final_callback in [false, true] {
                cancellation_case(false, mode, final_callback);
            }
        }
    }

    #[test]
    fn detached_restore_cancellation_reports_consumed_rows_and_skips_replacement_tail_and_statistics(
    ) {
        for mode in 0..3 {
            for final_callback in [false, true] {
                cancellation_case(true, mode, final_callback);
            }
        }
    }

    #[test]
    fn detached_remove_and_restore_keep_valid_registration_pending_through_every_phase() {
        let mut world = fixture(false);
        let mut batch = None;
        for restoring in [false, true] {
            let original = world.schedule().stage_plan();
            let callbacks = Arc::new(AtomicUsize::new(0));
            let systems = Arc::new(AtomicUsize::new(0));
            let system_id = if restoring {
                "tests.detached.restore-pending"
            } else {
                "tests.detached.remove-pending"
            };
            let kinds = if restoring {
                [LifecycleEventKind::Add, LifecycleEventKind::Insert]
            } else {
                [LifecycleEventKind::Remove, LifecycleEventKind::Despawn]
            };
            let callback = {
                let original = Arc::clone(&original);
                let callbacks = Arc::clone(&callbacks);
                let systems = Arc::clone(&systems);
                move |world: &mut World, _: &ComponentLifecycleEvent| {
                    assert_eq!(world.contains_entity(1), restoring);
                    assert_eq!(world.contains_entity(2), restoring);
                    assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                    if callbacks.fetch_add(1, Ordering::SeqCst) == 0 {
                        let systems = Arc::clone(&systems);
                        world
                            .register_native_system::<(), _>(
                                system_id,
                                SystemStage::Update,
                                0,
                                move |()| {
                                    systems.fetch_add(1, Ordering::SeqCst);
                                },
                            )
                            .unwrap();
                    }
                    assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                }
            };
            observe_rows!(world, kinds, callback);
            if restoring {
                world
                    .restore_detached_entity_batch(batch.take().unwrap())
                    .unwrap();
            } else {
                batch = Some(world.remove_entity_subtrees([1, 2]).unwrap());
            }
            assert_eq!(callbacks.load(Ordering::SeqCst), 8);
            assert_eq!(systems.load(Ordering::SeqCst), 0);
            assert!(!Arc::ptr_eq(&original, &world.schedule().stage_plan()));
            assert!(world.run_native_scene_system(system_id));
            assert_eq!(systems.load(Ordering::SeqCst), 1);
            assert_eq!(
                world
                    .ecs_frame_performance_diagnostics()
                    .detached_entity_batches
                    .commit_count,
                if restoring { 2 } else { 1 }
            );
        }
    }

    #[test]
    fn detached_remove_and_restore_preserve_enclosing_staging_and_its_event_prefix() {
        let mut world = fixture(false);
        let callbacks = Arc::new(AtomicUsize::new(0));
        let callback = {
            let callbacks = Arc::clone(&callbacks);
            move |_: &mut World, _: &ComponentLifecycleEvent| {
                callbacks.fetch_add(1, Ordering::SeqCst);
            }
        };
        observe_rows!(
            world,
            [
                LifecycleEventKind::Add,
                LifecycleEventKind::Insert,
                LifecycleEventKind::Remove,
                LifecycleEventKind::Despawn
            ],
            callback
        );
        world.record_staged_lifecycle_events = true;
        let component = world.component_id::<OwnedTable>();
        world.trigger_component_lifecycle(LifecycleEventKind::Add, 90, component);
        let prefix = world.staged_lifecycle_events.clone();
        let batch = world.remove_entity_subtrees([1, 2]).unwrap();
        assert_eq!(callbacks.load(Ordering::SeqCst), 0);
        assert!(world.record_staged_lifecycle_events);
        assert_eq!(
            &world.staged_lifecycle_events[..prefix.len()],
            prefix.as_slice()
        );
        assert_eq!(world.staged_lifecycle_events.len(), prefix.len() + 8);
        world.restore_detached_entity_batch(batch).unwrap();
        assert!(world.record_staged_lifecycle_events);
        assert_eq!(
            &world.staged_lifecycle_events[..prefix.len()],
            prefix.as_slice()
        );
        assert_eq!(world.staged_lifecycle_events.len(), prefix.len() + 16);
        assert_eq!(callbacks.load(Ordering::SeqCst), 0);
        world.record_staged_lifecycle_events = false;
        let execution = world.begin_schedule_execution();
        for event in std::mem::take(&mut world.staged_lifecycle_events) {
            assert!(execution.is_bound_to(&world));
            world.dispatch_component_lifecycle(event);
        }
        assert!(execution.is_bound_to(&world));
        assert_eq!(callbacks.load(Ordering::SeqCst), 17);
    }

    #[test]
    fn detached_observer_panic_preserves_the_original_panic_and_releases_the_original_execution() {
        for restoring in [false, true] {
            let mut world = fixture(false);
            let mut batch = restoring.then(|| world.remove_entity_subtrees([1, 2]).unwrap());
            let original = world.schedule().stage_plan();
            let retained = Arc::new(Mutex::new(None));
            let saved = Arc::clone(&retained);
            let systems = Arc::new(AtomicUsize::new(0));
            let observed = Arc::clone(&systems);
            let kind = if restoring {
                LifecycleEventKind::Add
            } else {
                LifecycleEventKind::Remove
            };
            world.observe_component_lifecycle::<OwnedTable>(kind,
                // SAFETY: The only producer registered here increments an external atomic.
                // Then the World is retired and the callback panics. No history component
                // or dynamic presence is added; replacement rows cannot be adopted by replay.
                unsafe { crate::core::framework::retained_memory::RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, _| {
                let observed = Arc::clone(&observed);
                world
                    .register_native_system::<(), _>(
                        "tests.detached.panic-pending",
                        SystemStage::Update,
                        0,
                        move |()| {
                            observed.fetch_add(1, Ordering::SeqCst);
                        },
                    )
                    .unwrap();
                *saved.lock().unwrap() = Some(std::mem::replace(world, World::empty()));
                panic!("detached original observer panic");
            }) });
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if restoring {
                    world
                        .restore_detached_entity_batch(batch.take().unwrap())
                        .unwrap();
                } else {
                    let _ = world.remove_entity_subtrees([1, 2]).unwrap();
                }
            }));
            let panic = result.unwrap_err();
            assert_eq!(
                panic.downcast_ref::<&'static str>(),
                Some(&"detached original observer panic")
            );
            assert_eq!(world.stable_entity_ids().count(), 0);
            let mut old = retained.lock().unwrap().take().unwrap();
            assert_eq!(old.contains_entity(1), restoring);
            assert!(!old.record_staged_lifecycle_events);
            assert!(!Arc::ptr_eq(&original, &old.schedule().stage_plan()));
            assert!(old.run_native_scene_system("tests.detached.panic-pending"));
            assert_eq!(systems.load(Ordering::SeqCst), 1);
        }
    }
}

#[test]
fn detached_entity_observer_bound_survives_snapshot_and_restore() {
    use crate::core::framework::retained_memory::RetainedEntityObserverV1;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use zircon_runtime_interface::retained_memory::{
        verified_std_model_v1, RetainedBoundBuilderV1, RetainedOwnerV1,
    };
    struct RetainedEventV1;
    let model = verified_std_model_v1().unwrap();
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty).unwrap();
    let observed = Arc::new(AtomicUsize::new(0));
    world.observe_entity_event::<RetainedEventV1>(
        root,
        RetainedEntityObserverV1::new(
            (Vec::<u8>::with_capacity(16 * 1024), Arc::clone(&observed)),
            |state, _, _, _| {
                state.1.fetch_add(1, Ordering::SeqCst);
            },
        ),
    );
    let mut before = RetainedBoundBuilderV1::new(model);
    world
        .detached_subtrees_retention_bound_v1(&[root], &mut before)
        .unwrap();
    let batch = world.remove_entity_subtrees([root]).unwrap();
    let mut detached = RetainedBoundBuilderV1::new(model);
    batch.retained_bound_v1(&mut detached).unwrap();
    assert!(detached.bytes().get() >= 16 * 1024);
    assert!(before.bytes().get() >= detached.bytes().get());
    assert_eq!(observed.load(Ordering::SeqCst), 0);

    world.restore_detached_entity_batch(batch).unwrap();
    world.trigger_entity_event(root, RetainedEventV1);
    assert_eq!(observed.load(Ordering::SeqCst), 1);
    let batch = world.remove_entity_subtrees([root]).unwrap();
    let mut detached_again = RetainedBoundBuilderV1::new(model);
    batch.retained_bound_v1(&mut detached_again).unwrap();
    assert!(detached_again.bytes().get() >= 16 * 1024);
    world.restore_detached_entity_batch(batch).unwrap();
    world.trigger_entity_event(root, RetainedEventV1);
    assert_eq!(observed.load(Ordering::SeqCst), 2);
}
