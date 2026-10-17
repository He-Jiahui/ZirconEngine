use std::sync::{Arc, Mutex};

use crate::scene::{
    components::{ActiveSelf, Hierarchy, LocalTransform, Mobility, Name, RenderLayerMask},
    ecs::{Component, LifecycleEventKind, StorageType},
    NodeKind, World,
};

#[derive(Debug, PartialEq, Eq)]
struct RebuiltArchetypeMarker;

zircon_runtime_interface::impl_retained_owner_v1!(RebuiltArchetypeMarker {});
impl Component for RebuiltArchetypeMarker {}

#[derive(Debug, PartialEq, Eq)]
struct RebuiltSparseArchetypeMarker;

zircon_runtime_interface::impl_retained_owner_v1!(RebuiltSparseArchetypeMarker {});
impl Component for RebuiltSparseArchetypeMarker {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}

fn node_record_with_id(id: u64) -> crate::scene::components::NodeRecord {
    let mut source = World::empty();
    let entity = source
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let mut record = source
        .node_record(entity)
        .expect("spawned node must produce a record");
    record.id = id;
    record
}

#[test]
fn node_record_batch_publishes_once_after_all_records_are_visible() {
    let mut world = World::empty();
    let mut first = node_record_with_id(41);
    first.name = "First".to_string();
    let mut second = node_record_with_id(42);
    second.name = "Second".to_string();
    second.parent = Some(first.id);

    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_for_callback = Arc::clone(&observed);
    let second_id = second.id;
    let name_component_id = world.component_id::<Name>();
    world.observe_component_lifecycle::<Name>(
        LifecycleEventKind::Add,
        // SAFETY: Only the external observation log changes after optional World reads;
        // retained World payload is untouched.
        unsafe {
            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, event| {
                observed_for_callback
                    .lock()
                    .expect("test observer lock")
                    .push((
                        event.entity(),
                        world.contains_entity(second_id),
                        world
                            .entity_archetype_component_ids(second_id)
                            .contains(&name_component_id),
                    ));
            })
        },
    );

    let generation_before = world.world_generation();
    world
        .insert_node_records(&[first.clone(), second.clone()])
        .expect("validated batch must commit");

    assert_eq!(world.world_generation(), generation_before + 1);
    assert_eq!(world.node_record(first.id), Some(first));
    assert_eq!(world.node_record(second.id), Some(second));
    let observed = observed.lock().expect("test observer lock");
    assert_eq!(observed.len(), 2);
    assert!(observed
        .iter()
        .all(|(_, second_visible, second_has_final_signature)| {
            *second_visible && *second_has_final_signature
        }));
}

#[test]
fn node_record_batch_assigns_the_complete_fixed_component_signature() {
    let mut world = World::empty();
    let record = node_record_with_id(41);

    world
        .insert_node_record(record.clone())
        .expect("validated record must commit with its final signature");

    let name = world.component_id::<Name>();
    let hierarchy = world.component_id::<Hierarchy>();
    let transform = world.component_id::<LocalTransform>();
    let active = world.component_id::<ActiveSelf>();
    let render_layer_mask = world.component_id::<RenderLayerMask>();
    let mobility = world.component_id::<Mobility>();
    let signature = world.entity_archetype_component_ids(record.id);
    assert!(signature.contains(&name));
    assert!(signature.contains(&hierarchy));
    assert!(signature.contains(&transform));
    assert!(signature.contains(&active));
    assert!(signature.contains(&render_layer_mask));
    assert!(signature.contains(&mobility));
}

#[test]
fn pending_component_row_publishes_dense_and_sparse_values_in_one_transition() {
    let mut world = World::empty();
    let entity = world
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let marker = world.component_id::<RebuiltArchetypeMarker>();
    let sparse_marker = world.component_id::<RebuiltSparseArchetypeMarker>();
    let assignments_before = world.archetype_assignment_count();

    let mut row = world.begin_component_row(entity);
    world.stage_component_row_value(&mut row, RebuiltArchetypeMarker);
    world.stage_component_row_value(&mut row, RebuiltSparseArchetypeMarker);
    world.commit_component_row(entity, row, true);

    assert!(
        world
            .entity_archetype_component_ids(entity)
            .contains(&marker),
        "final signature reconstruction must not depend on a fixed component whitelist"
    );
    assert!(
        world
            .entity_archetype_component_ids(entity)
            .contains(&sparse_marker),
        "final signature reconstruction must preserve sparse canonical storage rows"
    );
    assert!(world.get::<RebuiltArchetypeMarker>(entity).is_some());
    assert!(world.get::<RebuiltSparseArchetypeMarker>(entity).is_some());
    assert_eq!(world.archetype_assignment_count() - assignments_before, 1);
}

#[test]
fn node_record_batch_publishes_one_final_archetype_assignment_per_record() {
    let mut world = World::empty();
    let first = node_record_with_id(41);
    let second = node_record_with_id(42);
    let assignments_before = world.archetype_assignment_count();

    world
        .insert_owned_node_records(vec![first, second])
        .expect("prevalidated records should publish their final signatures");

    assert_eq!(world.archetype_assignment_count() - assignments_before, 2);
}

#[test]
fn duplicate_node_record_batch_leaves_world_unchanged() {
    let mut world = World::empty();
    let record = node_record_with_id(41);
    let generation_before = world.world_generation();

    let error = world
        .insert_node_records(&[record.clone(), record])
        .expect_err("duplicate batch identities must fail before commit");

    assert!(matches!(
        error,
        crate::scene::SceneError::DuplicateEntity { .. }
    ));
    assert_eq!(world.world_generation(), generation_before);
    assert!(world.node_records().is_empty());
}

#[test]
fn stale_registry_identity_rejects_the_entire_node_record_batch_before_commit() {
    let mut world = World::empty();
    world
        .register_stable_entity(42)
        .expect("test fixture should create a registry-only identity");
    let first = node_record_with_id(41);
    let second = node_record_with_id(42);
    let generation_before = world.world_generation();

    let error = world
        .insert_owned_node_records(vec![first, second])
        .expect_err("registry identity must be rejected during batch prevalidation");

    assert!(matches!(
        error,
        crate::scene::SceneError::DuplicateEntity { entity: 42 }
    ));
    assert_eq!(world.world_generation(), generation_before);
    assert!(world.node_records().is_empty());
}

#[test]
fn dynamic_parent_with_static_incoming_child_rejects_the_whole_batch() {
    let mut world = World::empty();
    let mut parent = node_record_with_id(41);
    parent.mobility = Mobility::Dynamic;
    let mut child = node_record_with_id(42);
    child.parent = Some(parent.id);
    let generation_before = world.world_generation();

    let error = world
        .insert_owned_node_records(vec![parent, child])
        .expect_err("dynamic parents cannot gain static children in a batch");

    assert!(matches!(
        error,
        crate::scene::SceneError::DynamicMobilityWithStaticChildren { entity: 41 }
    ));
    assert_eq!(world.world_generation(), generation_before);
    assert!(world.node_records().is_empty());
}

mod lifecycle_cancellation {
    use super::*;
    use crate::scene::ecs::{
        ComponentLifecycleEvent, DeferredCommandError, DeferredCommandOperation,
        DeferredCommandTarget, Resource, SystemStage,
    };
    use crate::scene::SceneError;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use zircon_runtime_interface::world_sync::LifecycleTargetChange;

    #[derive(Debug, PartialEq, Eq)]
    struct RecordSentinel(u32);
    impl Resource for RecordSentinel {}

    macro_rules! observe_node_phases {
        ($world:expr, $callback:expr) => {{
            for kind in [LifecycleEventKind::Add, LifecycleEventKind::Insert] {
                $world.observe_component_lifecycle::<Name>(
                    kind,
                    // SAFETY: Every local macro caller only observes, registers counter-only
                    // systems, or retires the owner with fixed diagnostic resources; no history
                    // payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            $callback.clone(),
                        )
                    },
                );
                $world.observe_component_lifecycle::<Hierarchy>(
                    kind,
                    // SAFETY: Every local macro caller only observes, registers counter-only
                    // systems, or retires the owner with fixed diagnostic resources; no history
                    // payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            $callback.clone(),
                        )
                    },
                );
                $world.observe_component_lifecycle::<LocalTransform>(
                    kind,
                    // SAFETY: Every local macro caller only observes, registers counter-only
                    // systems, or retires the owner with fixed diagnostic resources; no history
                    // payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            $callback.clone(),
                        )
                    },
                );
                $world.observe_component_lifecycle::<ActiveSelf>(
                    kind,
                    // SAFETY: Every local macro caller only observes, registers counter-only
                    // systems, or retires the owner with fixed diagnostic resources; no history
                    // payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            $callback.clone(),
                        )
                    },
                );
                $world.observe_component_lifecycle::<RenderLayerMask>(
                    kind,
                    // SAFETY: Every local macro caller only observes, registers counter-only
                    // systems, or retires the owner with fixed diagnostic resources; no history
                    // payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            $callback.clone(),
                        )
                    },
                );
                $world.observe_component_lifecycle::<Mobility>(
                    kind,
                    // SAFETY: Every local macro caller only observes, registers counter-only
                    // systems, or retires the owner with fixed diagnostic resources; no history
                    // payload grows.
                    unsafe {
                        use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                        RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                            $callback.clone(),
                        )
                    },
                );
            }
        }};
    }

    fn install_sentinel(world: &mut World, entity: u64, calls: Arc<AtomicUsize>) {
        world.insert_resource(RecordSentinel(83));
        world.record_deferred_command_error(DeferredCommandError::new(
            DeferredCommandOperation::Remove,
            DeferredCommandTarget::Resolved(entity),
            SceneError::Message("record replacement diagnostics".into()),
        ));
        world.commands().queue_fn(move |_| {
            calls.fetch_add(1, Ordering::SeqCst);
        });
    }

    #[test]
    fn node_record_batch_registration_keeps_every_callback_on_the_original_plan() {
        let mut world = World::empty();
        let original = world.schedule().stage_plan();
        let callbacks = Arc::new(AtomicUsize::new(0));
        let systems = Arc::new(AtomicUsize::new(0));
        let callback = {
            let original = Arc::clone(&original);
            let callbacks = Arc::clone(&callbacks);
            let systems = Arc::clone(&systems);
            move |world: &mut World, _: &ComponentLifecycleEvent| {
                assert!(world.contains_entity(41) && world.contains_entity(42));
                assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                if callbacks.fetch_add(1, Ordering::SeqCst) == 0 {
                    let systems = Arc::clone(&systems);
                    world
                        .register_native_system::<(), _>(
                            "tests.records.pending",
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
        observe_node_phases!(world, callback);
        world
            .insert_node_records(&[node_record_with_id(41), node_record_with_id(42)])
            .unwrap();
        assert_eq!(callbacks.load(Ordering::SeqCst), 24);
        assert_eq!(systems.load(Ordering::SeqCst), 0);
        assert!(!Arc::ptr_eq(&original, &world.schedule().stage_plan()));
        assert!(world.run_native_scene_system("tests.records.pending"));
        assert_eq!(systems.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn public_record_entries_report_first_and_final_callback_target_changes() {
        // Exercise spawn_node, one record and a real two-record batch. Every
        // phase shares one counter, so the test does not depend on component order.
        for entry in 0..3 {
            for mode in 0..3 {
                for final_callback in [false, true] {
                    let ids = if entry == 0 {
                        vec![1]
                    } else if entry == 1 {
                        vec![41]
                    } else {
                        vec![41, 42]
                    };
                    let total = ids.len() * 12;
                    let boundary = if final_callback { total } else { 1 };
                    let expected_target = if mode == 2 {
                        LifecycleTargetChange::Schedule
                    } else {
                        LifecycleTargetChange::World
                    };
                    let mut world = World::empty();
                    let owner = world.owner_identity().clone();
                    let original_plan = world.schedule().stage_plan();
                    let callbacks = Arc::new(AtomicUsize::new(0));
                    let leaked = Arc::new(AtomicUsize::new(0));
                    let fresh_calls = Arc::new(AtomicUsize::new(0));
                    let generation = Arc::new(Mutex::new(None));
                    let mut fresh = World::empty();
                    let fresh_records = ids
                        .iter()
                        .copied()
                        .map(|id| {
                            let mut record = node_record_with_id(id);
                            record.name = format!("Fresh{id}");
                            record
                        })
                        .collect();
                    fresh.insert_owned_node_records(fresh_records).unwrap();
                    let leaked_callback = {
                        let leaked = Arc::clone(&leaked);
                        move |_: &mut World, _: &ComponentLifecycleEvent| {
                            leaked.fetch_add(1, Ordering::SeqCst);
                        }
                    };
                    observe_node_phases!(fresh, leaked_callback);
                    assert_eq!(world.component_id::<Name>(), fresh.component_id::<Name>());
                    install_sentinel(&mut fresh, ids[0], Arc::clone(&fresh_calls));
                    let fresh = Arc::new(Mutex::new(Some(fresh)));
                    let callback = {
                        let callbacks = Arc::clone(&callbacks);
                        let fresh_calls = Arc::clone(&fresh_calls);
                        let generation = Arc::clone(&generation);
                        let ids = ids.clone();
                        let original_plan = Arc::clone(&original_plan);
                        move |world: &mut World, _: &ComponentLifecycleEvent| {
                            assert!(ids.iter().all(|id| world.contains_entity(*id)));
                            if callbacks.fetch_add(1, Ordering::SeqCst) + 1 == boundary {
                                if mode == 2 {
                                    *world.schedule_mut() = Default::default();
                                    install_sentinel(world, ids[0], Arc::clone(&fresh_calls));
                                } else {
                                    let mut replacement = fresh.lock().unwrap().take().unwrap();
                                    if mode == 1 {
                                        *replacement.schedule_mut() =
                                            std::mem::take(world.schedule_mut());
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
                    observe_node_phases!(world, callback);
                    let result = match entry {
                        0 => world.spawn_node(NodeKind::Empty).map(|_| ()),
                        1 => world.insert_node_record(node_record_with_id(41)),
                        2 => world.insert_node_records(&[
                            node_record_with_id(41),
                            node_record_with_id(42),
                        ]),
                        _ => unreachable!(),
                    };
                    let error = result.unwrap_err();
                    assert_eq!(error.lifecycle_target_change(), Some(expected_target));
                    assert!(matches!(
                        error,
                        SceneError::LifecycleTargetReplaced {
                            operation: "insert node records",
                            ..
                        }
                    ));
                    assert_eq!(callbacks.load(Ordering::SeqCst), boundary);
                    assert_eq!(leaked.load(Ordering::SeqCst), 0);
                    assert_eq!(
                        world.world_generation(),
                        generation.lock().unwrap().unwrap()
                    );
                    assert_eq!(
                        world.get_resource::<RecordSentinel>(),
                        Some(&RecordSentinel(83))
                    );
                    assert_eq!(owner.is_same_world(world.owner_identity()), mode == 2);
                    if mode != 2 {
                        for id in ids {
                            assert_eq!(world.get::<Name>(id), Some(&Name(format!("Fresh{id}"))));
                        }
                    }
                    let diagnostics = world.take_deferred_command_errors();
                    assert_eq!(diagnostics.len(), 1);
                    assert_eq!(
                        diagnostics[0].error(),
                        &SceneError::Message("record replacement diagnostics".into())
                    );
                    assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
                    assert!(world.apply_deferred().is_success());
                    assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
                }
            }
        }
    }

    #[test]
    fn node_record_batch_preserves_the_enclosing_staging_prefix() {
        let mut world = World::empty();
        let prefix_entity = world.spawn_node(NodeKind::Empty).unwrap();
        let callbacks = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&callbacks);
        world.observe_component_lifecycle::<Name>(
            LifecycleEventKind::Add,
            // SAFETY: Only an external atomic counter changes; retained World payload and
            // dynamic JSON are untouched.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_, _| {
                    observed.fetch_add(1, Ordering::SeqCst);
                })
            },
        );
        world.record_staged_lifecycle_events = true;
        let name = world.component_id::<Name>();
        world.trigger_component_lifecycle(LifecycleEventKind::Add, prefix_entity, name);
        let prefix = world.staged_lifecycle_events.clone();
        world
            .insert_owned_node_records(vec![node_record_with_id(41), node_record_with_id(42)])
            .unwrap();
        assert!(world.record_staged_lifecycle_events);
        assert_eq!(
            &world.staged_lifecycle_events[..prefix.len()],
            prefix.as_slice()
        );
        assert_eq!(world.staged_lifecycle_events.len(), prefix.len() + 24);
        assert_eq!(callbacks.load(Ordering::SeqCst), 0);
        world.record_staged_lifecycle_events = false;
        let execution = world.begin_schedule_execution();
        for event in std::mem::take(&mut world.staged_lifecycle_events) {
            assert!(execution.is_bound_to(&world));
            world.dispatch_component_lifecycle(event);
        }
        assert!(execution.is_bound_to(&world));
        assert_eq!(callbacks.load(Ordering::SeqCst), 3);
    }
}
