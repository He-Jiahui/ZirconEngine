use super::*;
use crate::scene::components::{MeshRenderer, Name};
use crate::scene::ecs::{
    EventReader, EventReaderParam, Local, LocalParam, Query, QueryState, Res, ResParam, Resource,
    SystemStage, SystemState, With,
};
use std::sync::{Arc, Mutex};
use zircon_runtime_interface::world_sync::LifecycleTargetChange;

#[derive(Debug)]
struct RetainedResource(u32);
impl Resource for RetainedResource {}
#[derive(Debug)]
struct LateResource;
impl Resource for LateResource {}
#[derive(Debug)]
struct StagedResource(u32);
impl Resource for StagedResource {}
#[derive(Debug)]
struct RetainedEvent(u32);

#[test]
fn retained_schedule_recompiles_same_generation_query_storage_and_keeps_local_resource_and_event_state(
) {
    let mut world = World::empty();
    let mesh = world.spawn_node(NodeKind::Mesh).unwrap();
    let empty = world.spawn_node(NodeKind::Empty).unwrap();
    world.rename_node(mesh, "original-mesh").unwrap();
    world.rename_node(empty, "original-empty").unwrap();
    world.insert_resource(RetainedResource(7));
    world.register_event::<RetainedEvent>();
    let event_id = world
        .event_type_id::<RetainedEvent>()
        .expect("retained event should be registered");
    let observed = Arc::new(Mutex::new(Vec::new()));
    let observed_for_system = Arc::clone(&observed);
    type Params = (
        QueryState<&'static Name, With<MeshRenderer>>,
        ResParam<RetainedResource>,
        EventReaderParam<RetainedEvent>,
        LocalParam<u32>,
    );
    world
        .register_native_system::<Params, _>(
            "tests.retained.typed",
            SystemStage::Update,
            0,
            move |(mut query, resource, mut events, mut local): (
                Query<'_, &'static Name, With<MeshRenderer>>,
                Res<'_, RetainedResource>,
                EventReader<'_, RetainedEvent>,
                Local<'_, u32>,
            )| {
                *local += 1;
                let names = query
                    .iter_cached_direct()
                    .map(|name| name.0.clone())
                    .collect::<Vec<_>>();
                let values = events.iter().map(|event| event.0).collect::<Vec<_>>();
                observed_for_system
                    .lock()
                    .unwrap()
                    .push((names, resource.0, values, *local));
            },
        )
        .unwrap();
    world.send_event(RetainedEvent(1));
    world.update_events::<RetainedEvent>();
    assert!(world.run_native_scene_system("tests.retained.typed"));
    let original_plan = world.schedule().stage_plan();
    let execution = world.begin_schedule_execution();
    let original_generation = world.archetype_generation();
    let copied = world.clone();
    assert!(!Arc::ptr_eq(
        world.query_storage_owner(),
        copied.query_storage_owner()
    ));
    drop(copied);

    // A valid staged scene shares the descriptor prefix, but its new tables
    // insert the two signatures in the opposite order and reuse the same count.
    let mut staged = World::empty();
    staged.component_registry = world.component_registry.clone();
    let new_empty = staged.spawn_node(NodeKind::Empty).unwrap();
    let new_mesh = staged.spawn_node(NodeKind::Mesh).unwrap();
    staged.rename_node(new_empty, "replacement-empty").unwrap();
    staged.rename_node(new_mesh, "replacement-mesh").unwrap();
    let live_typed_resource_id = world.resource_id::<RetainedResource>();
    staged.insert_resource(StagedResource(11));
    staged.insert_resource(RetainedResource(9));
    assert_ne!(
        staged.resource_id::<RetainedResource>(),
        live_typed_resource_id
    );
    assert_eq!(staged.archetype_generation(), original_generation);
    let live_resource_id = world.resource_id::<LateResource>();
    let retained = world.commit_staged_scene_state(staged).unwrap();
    drop(retained);
    assert!(Arc::ptr_eq(&original_plan, &world.schedule().stage_plan()));
    assert_eq!(world.resource_id::<LateResource>(), live_resource_id);
    assert_eq!(
        world.resource_id::<RetainedResource>(),
        live_typed_resource_id
    );
    assert_ne!(world.resource_id::<StagedResource>(), live_resource_id);
    assert_eq!(world.resource::<StagedResource>().0, 11);
    assert_eq!(world.event_reader_count(event_id), Some(1));
    world.send_event(RetainedEvent(2));
    world.update_events::<RetainedEvent>();
    assert!(world.run_native_scene_system("tests.retained.typed"));
    drop(execution);
    assert!(world.run_native_scene_system("tests.retained.typed"));
    assert_eq!(
        *observed.lock().unwrap(),
        vec![
            (vec!["original-mesh".to_string()], 7, vec![1], 1),
            (vec!["replacement-mesh".to_string()], 9, vec![2], 2),
            (
                vec!["replacement-mesh".to_string()],
                9,
                Vec::<u32>::new(),
                3
            ),
        ]
    );
    assert!(world
        .unregister_native_system("tests.retained.typed")
        .unwrap());
    assert_eq!(world.event_reader_count(event_id), Some(0));
}

#[test]
fn incompatible_staged_component_ids_reject_before_runtime_state_or_epoch_changes() {
    let mut world = World::empty();
    world.component_id::<Name>();
    let original_owner = world.owner_identity().clone();
    let original_plan = world.schedule().stage_plan();
    world.register_event::<RetainedEvent>();
    let event_id = world
        .event_type_id::<RetainedEvent>()
        .expect("retained event should be registered");
    world.send_event(RetainedEvent(4));
    let mut staged = World::empty();
    staged.component_id::<MeshRenderer>();
    let error = world
        .commit_staged_scene_state(staged)
        .err()
        .expect("incompatible registry");
    assert_eq!(
        error,
        crate::scene::dynamic_scene::DynamicSceneError::IncompatibleRetainedComponentRegistry
    );
    assert!(original_owner.is_same_world(world.owner_identity()));
    assert!(Arc::ptr_eq(&original_plan, &world.schedule().stage_plan()));
    assert_eq!(world.event_store().reader_count(event_id), Some(0));
    assert!(world.events::<RetainedEvent>().is_some());

    let level = crate::scene::LevelSystem::new(
        crate::core::framework::scene::WorldHandle::new(91),
        Arc::new(Mutex::new(world)),
        crate::scene::LevelMetadata::default(),
    );
    let epoch = level.capture_world_replacement_epoch();
    let generation = level.world_generation();
    let mut incompatible = World::empty();
    incompatible.component_id::<MeshRenderer>();
    assert_eq!(
        level.replace_world_if_generation(generation, incompatible),
        Err(crate::scene::dynamic_scene::DynamicSceneError::IncompatibleRetainedComponentRegistry)
    );
    assert_eq!(level.capture_world_replacement_epoch(), epoch);
    assert!(level.with_world(|world| Arc::ptr_eq(&original_plan, &world.schedule().stage_plan())));
}

#[test]
fn complete_world_replacement_keeps_original_system_state_rejected() {
    let mut original = World::empty();
    original.spawn_node(NodeKind::Empty).unwrap();
    let mut state = SystemState::<QueryState<&'static Name>>::new(&mut original).unwrap();
    state.run(&mut original, |mut query| {
        assert_eq!(query.iter().count(), 1);
    });
    let replacement = World::empty();
    assert!(!state.is_bound_to(&replacement));
    let mut replacement = replacement;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        state.run(&mut replacement, |_| {});
    }));
    assert!(result.is_err());
    state.retire(&mut original);
}

struct ExclusiveWorldParam;
impl crate::scene::ecs::SystemParam for ExclusiveWorldParam {
    type State = ();
    type Item<'world> = &'world mut World;

    fn init_state(
        _world: &mut World,
        access: &mut crate::scene::ecs::SystemParamAccess,
    ) -> Result<(), crate::scene::ecs::SystemParamError> {
        access.add_conservative_world_access();
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut (),
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Self::Item<'world> {
        // SAFETY: this test registers sole conservative access and retains no
        // storage/cache loan in State; the Item is the original exclusive grant.
        unsafe { &mut *world }
    }
}

#[test]
fn typed_exclusive_callback_move_restores_the_original_retained_active_tick_slot() {
    let mut world = World::empty();
    let mut state = SystemState::<ExclusiveWorldParam>::new(&mut world).unwrap();
    let mut retained = None;
    state.run(&mut world, |world| {
        assert!(world.active_change_tick_slot().get().is_some());
        retained = Some(std::mem::replace(world, World::empty()));
    });
    let mut original = retained.unwrap();
    assert_eq!(original.active_change_tick_slot().get(), None);
    assert_eq!(world.active_change_tick_slot().get(), None);
    assert!(state.is_bound_to(&original));
    assert!(!state.is_bound_to(&world));
    state.run(&mut original, |world| {
        assert!(world.active_change_tick_slot().get().is_some());
    });
    assert_eq!(original.active_change_tick_slot().get(), None);
    state.retire(&mut original);
}

#[test]
fn retained_commit_lifecycle_replacement_cancels_remaining_replay_and_level_publication() {
    use crate::scene::dynamic_scene::DynamicSceneError;
    use crate::scene::ecs::LifecycleEventKind;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration};

    for panic_after_replacement in [false, true] {
        let mut world = World::empty();
        let retained = Arc::new(Mutex::new(None));
        let retained_for_observer = Arc::clone(&retained);
        let first_calls = Arc::new(AtomicUsize::new(0));
        let first_for_observer = Arc::clone(&first_calls);
        let old_later_calls = Arc::new(AtomicUsize::new(0));
        let fresh_calls = Arc::new(AtomicUsize::new(0));
        let fresh_for_observer = Arc::clone(&fresh_calls);
        world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |world, _| {
            first_for_observer.fetch_add(1, Ordering::SeqCst);
            let mut replacement = World::empty();
            let fresh = Arc::clone(&fresh_for_observer);
            replacement.observe_component_lifecycle::<Name>(
                LifecycleEventKind::Add,
                move |_, _| {
                    fresh.fetch_add(1, Ordering::SeqCst);
                },
            );
            replacement.commands().queue_fn(|world| {
                world.spawn_node(NodeKind::Empty).unwrap();
            });
            *retained_for_observer.lock().unwrap() = Some(std::mem::replace(world, replacement));
            if panic_after_replacement {
                panic!("original retained lifecycle panic");
            }
        });
        let later = Arc::clone(&old_later_calls);
        world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |_, _| {
            later.fetch_add(1, Ordering::SeqCst);
        });
        let level =
            crate::scene::DefaultLevelManager::default().create_level(world, Default::default());
        let _watch = level.watch_world(WatchRegistration::new(WatchKey::WorldStructure));
        let original_owner = level.with_world(|world| world.owner_identity().clone());
        let execution = level.with_world_mut(|world| world.begin_schedule_execution());
        let expected_generation = level.world_generation();
        let (mut staged, _) = level
            .with_world_mut(|world| world.clone_for_dynamic_scene_staging(16 * 1024 * 1024))
            .unwrap();
        let first = staged.spawn_node(NodeKind::Empty).unwrap();
        let second = staged.spawn_node(NodeKind::Empty).unwrap();
        staged.rename_node(first, "staged-first").unwrap();
        staged.rename_node(second, "staged-second").unwrap();
        assert!(!staged.staged_lifecycle_events.is_empty());

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            level.replace_world_if_generation(expected_generation, staged)
        }));
        if panic_after_replacement {
            let panic = result.unwrap_err();
            assert_eq!(
                panic.downcast_ref::<&'static str>(),
                Some(&"original retained lifecycle panic")
            );
        } else {
            assert_eq!(
                result.unwrap(),
                Err(DynamicSceneError::LifecycleTargetReplacedDuringCommit {
                    target: LifecycleTargetChange::World,
                })
            );
        }
        assert_eq!(first_calls.load(Ordering::SeqCst), 1);
        assert_eq!(old_later_calls.load(Ordering::SeqCst), 0);
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
        assert!(level.drain_world_invalidations().is_empty());
        assert_eq!(
            level.with_world(|world| execution.target_change(world)),
            Some(LifecycleTargetChange::World)
        );
        let mut original = retained.lock().unwrap().take().unwrap();
        assert!(original_owner.is_same_world(original.owner_identity()));
        assert!(execution.is_bound_to(&original));
        assert!(!original.record_staged_lifecycle_events);
        let mut state = SystemState::<QueryState<&'static Name>>::new(&mut original).unwrap();
        state.run(&mut original, |mut query| {
            let names = query.iter().map(|name| name.0.clone()).collect::<Vec<_>>();
            assert_eq!(
                names,
                vec!["staged-first".to_string(), "staged-second".to_string()]
            );
        });
        state.retire(&mut original);
        drop(execution);
        assert!(level
            .with_world_mut(|world| world.apply_deferred())
            .is_success());
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
    }
}

#[derive(Debug)]
struct ReplayFirstComponent;
impl crate::scene::ecs::Component for ReplayFirstComponent {}

#[derive(Debug)]
struct ReplaySecondComponent;
impl crate::scene::ecs::Component for ReplaySecondComponent {}

#[test]
fn direct_spawn_lifecycle_replay_keeps_valid_registration_pending_through_later_component_events() {
    use crate::scene::ecs::{ComponentLifecycleEvent, LifecycleEventKind};
    use std::sync::atomic::{AtomicUsize, Ordering};

    const SYSTEM_ID: &str = "tests.lifecycle-replay.spawn-native";
    let mut world = World::empty();
    let original_plan = world.schedule().stage_plan();
    let observer_calls = Arc::new(AtomicUsize::new(0));
    let system_calls = Arc::new(AtomicUsize::new(0));
    let callback = || {
        let plan = Arc::clone(&original_plan);
        let observed = Arc::clone(&observer_calls);
        let new_calls = Arc::clone(&system_calls);
        move |world: &mut World, _: &ComponentLifecycleEvent| {
            assert!(Arc::ptr_eq(&plan, &world.schedule().stage_plan()));
            if observed.fetch_add(1, Ordering::SeqCst) == 0 {
                let calls = Arc::clone(&new_calls);
                world
                    .register_native_system::<(), _>(SYSTEM_ID, SystemStage::Update, 0, move |()| {
                        calls.fetch_add(1, Ordering::SeqCst);
                    })
                    .unwrap();
            }
            assert!(Arc::ptr_eq(&plan, &world.schedule().stage_plan()));
        }
    };
    world.observe_component_lifecycle::<ReplayFirstComponent>(LifecycleEventKind::Add, callback());
    world.observe_component_lifecycle::<ReplaySecondComponent>(LifecycleEventKind::Add, callback());

    // Both typed values take the public direct bundle path, with no command
    // queue or outer tick lease that could hide a missing replay owner.
    world
        .spawn((ReplayFirstComponent, ReplaySecondComponent))
        .unwrap();
    assert_eq!(observer_calls.load(Ordering::SeqCst), 2);
    assert_eq!(system_calls.load(Ordering::SeqCst), 0);
    assert!(!Arc::ptr_eq(&original_plan, &world.schedule().stage_plan()));
    assert!(world.run_native_scene_system(SYSTEM_ID));
    assert_eq!(system_calls.load(Ordering::SeqCst), 1);
    assert!(world.unregister_native_system(SYSTEM_ID).unwrap());
}

#[test]
fn direct_despawn_lifecycle_replay_keeps_valid_registration_pending_through_remaining_removals() {
    use crate::scene::ecs::{ComponentLifecycleEvent, LifecycleEventKind};
    use std::sync::atomic::{AtomicUsize, Ordering};

    const SYSTEM_ID: &str = "tests.lifecycle-replay.despawn-native";
    let mut world = World::empty();
    let entity = world
        .spawn((ReplayFirstComponent, ReplaySecondComponent))
        .unwrap();
    let original_plan = world.schedule().stage_plan();
    let observer_calls = Arc::new(AtomicUsize::new(0));
    let system_calls = Arc::new(AtomicUsize::new(0));
    let callback = || {
        let plan = Arc::clone(&original_plan);
        let observed = Arc::clone(&observer_calls);
        let new_calls = Arc::clone(&system_calls);
        move |world: &mut World, _: &ComponentLifecycleEvent| {
            assert!(Arc::ptr_eq(&plan, &world.schedule().stage_plan()));
            if observed.fetch_add(1, Ordering::SeqCst) == 0 {
                let calls = Arc::clone(&new_calls);
                world
                    .register_native_system::<(), _>(SYSTEM_ID, SystemStage::Update, 0, move |()| {
                        calls.fetch_add(1, Ordering::SeqCst);
                    })
                    .unwrap();
            }
            assert!(Arc::ptr_eq(&plan, &world.schedule().stage_plan()));
        }
    };
    world.observe_component_lifecycle::<ReplayFirstComponent>(
        LifecycleEventKind::Remove,
        callback(),
    );
    world.observe_component_lifecycle::<ReplaySecondComponent>(
        LifecycleEventKind::Remove,
        callback(),
    );

    // Invoke the same production despawn transaction without apply_deferred's
    // enclosing lease, so this protects the despawn replay's own lifetime.
    world
        .begin_deferred_bundle_insertion(entity)
        .unwrap()
        .finish_with_deferred_despawn()
        .unwrap();
    assert!(!world.contains_entity(entity));
    assert_eq!(observer_calls.load(Ordering::SeqCst), 2);
    assert_eq!(system_calls.load(Ordering::SeqCst), 0);
    assert!(!Arc::ptr_eq(&original_plan, &world.schedule().stage_plan()));
    assert!(world.run_native_scene_system(SYSTEM_ID));
    assert_eq!(system_calls.load(Ordering::SeqCst), 1);
    assert!(world.unregister_native_system(SYSTEM_ID).unwrap());
}
