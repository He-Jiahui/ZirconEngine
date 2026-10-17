use super::admission::WorldCallback;
use super::support::try_run_test_stage;
use super::*;
use crate::scene::ecs::{FunctionRuntimeSceneSystem, SceneSystemMetadata};

fn replacement_world(marker: Arc<AtomicUsize>) -> World {
    let mut world = World::empty();
    world.commands().queue_fn(move |_| {
        marker.fetch_add(1, Ordering::SeqCst);
    });
    world
}

#[test]
fn runtime_replacement_preserves_callback_error_or_panic_and_cancels_old_dispatch_cleanup() {
    for outcome in 0..3 {
        let core = CoreRuntime::new();
        let level = test_level(RuntimeExtensionRegistry::default());
        let old_late = Arc::new(AtomicUsize::new(0));
        let new_same_id = Arc::new(AtomicUsize::new(0));
        let replacement_marker = Arc::new(AtomicUsize::new(0));
        let original_same_id = Arc::clone(&new_same_id);
        let marker = Arc::clone(&replacement_marker);
        let old_late_for_system = Arc::clone(&old_late);
        level.with_world_mut(|world| {
            world
                .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                    SceneSystemMetadata::new("tests.replace.runtime", SystemStage::Update, -10),
                    move |context| {
                        let mut replacement = replacement_world(Arc::clone(&marker));
                        let calls = Arc::clone(&original_same_id);
                        replacement
                            .register_boxed_runtime_scene_system(Box::new(
                                FunctionRuntimeSceneSystem::new(
                                    SceneSystemMetadata::new(
                                        "tests.replace.runtime",
                                        SystemStage::Update,
                                        -10,
                                    ),
                                    move |_| {
                                        calls.fetch_add(1, Ordering::SeqCst);
                                        Ok(())
                                    },
                                ),
                            ))
                            .unwrap();
                        context.level.replace(replacement);
                        match outcome {
                            1 => Err(CoreError::Initialization(
                                "original callback".to_string(),
                                "original replacement error".to_string(),
                            )),
                            2 => panic!("original replacement panic"),
                            _ => Ok(()),
                        }
                    },
                )))
                .unwrap();
            world
                .register_native_system::<(), _>(
                    "tests.replace.late",
                    SystemStage::Update,
                    20,
                    move |()| {
                        old_late_for_system.fetch_add(1, Ordering::SeqCst);
                    },
                )
                .unwrap();
        });
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            try_run_test_stage(&core.handle(), &level)
        }));
        match outcome {
            0 => assert!(result
                .unwrap()
                .unwrap_err()
                .into_parts()
                .1
                .to_string()
                .contains("schedule was replaced")),
            1 => match result.unwrap().unwrap_err().into_parts().1 {
                CoreError::Initialization(owner, message) => {
                    assert_eq!(owner, "original callback");
                    assert_eq!(message, "original replacement error");
                }
                other => panic!("unexpected replacement error: {other}"),
            },
            2 => assert_eq!(
                result.unwrap_err().downcast_ref::<&'static str>(),
                Some(&"original replacement panic")
            ),
            _ => unreachable!(),
        }
        assert_eq!(old_late.load(Ordering::SeqCst), 0);
        assert_eq!(new_same_id.load(Ordering::SeqCst), 0);
        assert_eq!(replacement_marker.load(Ordering::SeqCst), 0);
        assert!(level.with_world(World::has_deferred_commands));
        run_test_stage(&core.handle(), &level);
        assert_eq!(new_same_id.load(Ordering::SeqCst), 1);
        assert_eq!(replacement_marker.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn native_replacement_never_restores_the_old_box_or_pending_work_into_new_world() {
    for should_panic in [false, true] {
        let core = CoreRuntime::new();
        let level = test_level(RuntimeExtensionRegistry::default());
        let replacement_marker = Arc::new(AtomicUsize::new(0));
        let calls = Arc::new(AtomicUsize::new(0));
        let marker = Arc::clone(&replacement_marker);
        let calls_for_replacement = Arc::clone(&calls);
        level.with_world_mut(|world| {
            world
                .register_boxed_native_system(Box::new(WorldCallback::new(
                    SceneSystemMetadata::new("tests.replace.native", SystemStage::Update, 0),
                    move |world| {
                        let mut replacement = replacement_world(Arc::clone(&marker));
                        let calls = Arc::clone(&calls_for_replacement);
                        replacement
                            .register_native_system::<CommandsParam, _>(
                                "tests.replace.native",
                                SystemStage::Update,
                                0,
                                move |_: crate::scene::ecs::Commands<'_>| {
                                    calls.fetch_add(1, Ordering::SeqCst);
                                },
                            )
                            .unwrap();
                        *world = replacement;
                        if should_panic {
                            panic!("original native replacement panic");
                        }
                    },
                )))
                .unwrap();
        });
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            try_run_test_stage(&core.handle(), &level)
        }));
        if should_panic {
            assert_eq!(
                result.unwrap_err().downcast_ref::<&'static str>(),
                Some(&"original native replacement panic")
            );
        } else {
            assert!(result.unwrap().is_err());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert_eq!(replacement_marker.load(Ordering::SeqCst), 0);
        run_test_stage(&core.handle(), &level);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(replacement_marker.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn worker_replacement_discards_old_commands_and_preserves_original_worker_panic() {
    for should_panic in [false, true] {
        let core = CoreRuntime::new();
        let level = test_level(RuntimeExtensionRegistry::default());
        let old_commands = Arc::new(AtomicUsize::new(0));
        let fresh_marker = Arc::new(AtomicUsize::new(0));
        let fresh_calls = Arc::new(AtomicUsize::new(0));
        let owner = level.clone();
        let old_commands_for_system = Arc::clone(&old_commands);
        let fresh_marker_for_system = Arc::clone(&fresh_marker);
        let fresh_calls_for_system = Arc::clone(&fresh_calls);
        level.with_world_mut(|world| {
            world
                .register_worldless_native_system::<CommandsParam, _>(
                    "tests.replace.worker",
                    SystemStage::Update,
                    0,
                    move |mut commands: crate::scene::ecs::Commands<'_>| {
                        let old = Arc::clone(&old_commands_for_system);
                        commands.queue_fn(move |_| {
                            old.fetch_add(1, Ordering::SeqCst);
                        });
                        let mut replacement =
                            replacement_world(Arc::clone(&fresh_marker_for_system));
                        let calls = Arc::clone(&fresh_calls_for_system);
                        replacement
                            .register_worldless_native_system::<CommandsParam, _>(
                                "tests.replace.worker",
                                SystemStage::Update,
                                0,
                                move |_: crate::scene::ecs::Commands<'_>| {
                                    calls.fetch_add(1, Ordering::SeqCst);
                                },
                            )
                            .unwrap();
                        owner.replace(replacement);
                        if should_panic {
                            panic!("original worker replacement panic");
                        }
                    },
                )
                .unwrap();
        });
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            try_run_test_stage(&core.handle(), &level)
        }));
        if should_panic {
            assert_eq!(
                result.unwrap_err().downcast_ref::<&'static str>(),
                Some(&"original worker replacement panic")
            );
        } else {
            assert!(result.unwrap().is_err());
        }
        assert_eq!(old_commands.load(Ordering::SeqCst), 0);
        assert_eq!(fresh_marker.load(Ordering::SeqCst), 0);
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
        run_test_stage(&core.handle(), &level);
        assert_eq!(old_commands.load(Ordering::SeqCst), 0);
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
        assert_eq!(fresh_marker.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn worker_deferred_replacement_cancels_remaining_command_window_and_old_restore() {
    let core = CoreRuntime::new();
    let level = test_level(RuntimeExtensionRegistry::default());
    let old_commands = Arc::new(AtomicUsize::new(0));
    let fresh_marker = Arc::new(AtomicUsize::new(0));
    let marker_for_system = Arc::clone(&fresh_marker);
    let old_for_system = Arc::clone(&old_commands);
    level.with_world_mut(|world| {
        world
            .register_worldless_native_system::<CommandsParam, _>(
                "tests.replace.deferred-worker",
                SystemStage::Update,
                0,
                move |mut commands: crate::scene::ecs::Commands<'_>| {
                    let marker = Arc::clone(&marker_for_system);
                    commands.queue_fn(move |world| {
                        *world = replacement_world(marker);
                    });
                    let old = Arc::clone(&old_for_system);
                    commands.queue_fn(move |_| {
                        old.fetch_add(1, Ordering::SeqCst);
                    });
                },
            )
            .unwrap();
    });
    assert!(try_run_test_stage(&core.handle(), &level).is_err());
    assert_eq!(old_commands.load(Ordering::SeqCst), 0);
    assert_eq!(fresh_marker.load(Ordering::SeqCst), 0);
    level.with_world_mut(|world| {
        assert!(world
            .schedule_mut()
            .take_native_system("tests.replace.deferred-worker")
            .is_none());
        world.apply_deferred();
    });
    assert_eq!(fresh_marker.load(Ordering::SeqCst), 1);
}

#[test]
fn whole_tick_replacement_cancels_later_stages_from_the_original_plan() {
    use crate::engine_module::EngineModule;

    let runtime = CoreRuntime::new();
    runtime
        .register_module(crate::core::runtime::TasksModule.descriptor())
        .unwrap();
    runtime
        .register_module(crate::core::runtime::TimeModule.descriptor())
        .unwrap();
    runtime
        .register_module(crate::asset::module_descriptor())
        .unwrap();
    runtime
        .register_module(crate::scene::module_descriptor())
        .unwrap();
    runtime
        .activate_module(crate::core::framework::scene::SCENE_MODULE_NAME)
        .unwrap();
    let level = crate::scene::create_default_level(&runtime.handle()).unwrap();
    let later_calls = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&later_calls);
    level.with_world_mut(|world| {
        world
            .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                SceneSystemMetadata::new("tests.replace.first", SystemStage::First, 0),
                |context| {
                    context.level.replace(World::empty());
                    Ok(())
                },
            )))
            .unwrap();
        world
            .register_native_system::<(), _>(
                "tests.replace.post",
                SystemStage::PostUpdate,
                0,
                move |()| {
                    calls.fetch_add(1, Ordering::SeqCst);
                },
            )
            .unwrap();
    });
    assert!(level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(1), 1)
        )
        .is_err());
    assert_eq!(later_calls.load(Ordering::SeqCst), 0);
    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(1), 1),
        )
        .unwrap();
}

#[test]
fn runtime_staged_commit_keeps_original_plan_and_restores_original_runtime_callback() {
    let core = CoreRuntime::new();
    let level = test_level(RuntimeExtensionRegistry::default());
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_for_system = Arc::clone(&calls);
    level.with_world_mut(|world| {
        world
            .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                SceneSystemMetadata::new("tests.replace.retained", SystemStage::Update, 0),
                move |context| {
                    let first = calls_for_system.fetch_add(1, Ordering::SeqCst) == 0;
                    if first {
                        let original = context
                            .level
                            .with_world(|world| world.schedule().stage_plan());
                        let (staged, _) = context.level.with_world_mut(|world| {
                            world.clone_for_dynamic_scene_staging(1024 * 1024).unwrap()
                        });
                        let generation = context.level.world_generation();
                        context
                            .level
                            .replace_world_if_generation(generation, staged)
                            .unwrap();
                        assert!(context.level.with_world(|world| Arc::ptr_eq(
                            &original,
                            &world.schedule().stage_plan()
                        )));
                    }
                    Ok(())
                },
            )))
            .unwrap();
    });
    run_test_stage(&core.handle(), &level);
    run_test_stage(&core.handle(), &level);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn detached_native_world_recovers_its_original_box_after_reattach() {
    let core = CoreRuntime::new();
    let level = test_level(RuntimeExtensionRegistry::default());
    let retained = Arc::new(Mutex::new(None::<World>));
    let retained_for_callback = Arc::clone(&retained);
    let once = Arc::new(AtomicBool::new(true));
    let callback_once = Arc::clone(&once);
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_for_callback = Arc::clone(&calls);
    level.with_world_mut(|world| {
        world
            .register_boxed_native_system(Box::new(WorldCallback::new(
                SceneSystemMetadata::new("tests.detach.native-world", SystemStage::Update, 0),
                move |world| {
                    calls_for_callback.fetch_add(1, Ordering::SeqCst);
                    if callback_once.swap(false, Ordering::SeqCst) {
                        *retained_for_callback.lock().unwrap() =
                            Some(std::mem::replace(world, World::empty()));
                    }
                },
            )))
            .unwrap();
    });
    assert!(try_run_test_stage(&core.handle(), &level).is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let original = retained.lock().unwrap().take().unwrap();
    level.replace(original);
    run_test_stage(&core.handle(), &level);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(level
        .with_world_mut(|world| world.unregister_native_system("tests.detach.native-world"))
        .unwrap());
}

#[test]
fn detached_schedule_returns_native_box_publishes_validated_pending_plan_and_can_unregister_it() {
    let core = CoreRuntime::new();
    let level = test_level(RuntimeExtensionRegistry::default());
    let retained = Arc::new(Mutex::new(None::<crate::scene::ecs::Schedule>));
    let retained_for_callback = Arc::clone(&retained);
    let original = level.with_world_mut(|world| {
        world
            .register_boxed_native_system(Box::new(WorldCallback::new(
                SceneSystemMetadata::new("tests.detach.native-schedule", SystemStage::Update, 0),
                move |world| {
                    world
                        .schedule_mut()
                        .register_system(crate::scene::ecs::SceneSystemDescriptor::new(
                            "tests.detach.pending",
                            SystemStage::Update,
                            crate::scene::ecs::InternalSceneSystem::NodeCache,
                        ))
                        .unwrap();
                    *retained_for_callback.lock().unwrap() =
                        Some(std::mem::take(world.schedule_mut()));
                    panic!("original detached native panic");
                },
            )))
            .unwrap();
        world.schedule().stage_plan()
    });
    let result =
        std::panic::catch_unwind(AssertUnwindSafe(|| run_test_stage(&core.handle(), &level)));
    assert_eq!(
        result.unwrap_err().downcast_ref::<&'static str>(),
        Some(&"original detached native panic")
    );
    let schedule = retained.lock().unwrap().take().unwrap();
    assert!(!Arc::ptr_eq(&original, &schedule.stage_plan()));
    assert!(schedule
        .systems()
        .iter()
        .any(|system| system.id == "tests.detach.pending"));
    level.with_world_mut(|world| {
        *world.schedule_mut() = schedule;
    });
    assert!(level
        .with_world_mut(|world| world.unregister_native_system("tests.detach.native-schedule"))
        .unwrap());
    assert!(!level
        .with_world_mut(|world| world.unregister_native_system("tests.detach.native-schedule"))
        .unwrap());
    run_test_stage(&core.handle(), &level);
}

#[test]
fn detached_runtime_world_recovers_exact_original_error_and_original_slot() {
    let core = CoreRuntime::new();
    let level = test_level(RuntimeExtensionRegistry::default());
    let retained = Arc::new(Mutex::new(None::<World>));
    let retained_for_callback = Arc::clone(&retained);
    let once = Arc::new(AtomicBool::new(true));
    let callback_once = Arc::clone(&once);
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_for_callback = Arc::clone(&calls);
    level.with_world_mut(|world| {
        world
            .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                SceneSystemMetadata::new("tests.detach.runtime", SystemStage::Update, 0),
                move |context| {
                    calls_for_callback.fetch_add(1, Ordering::SeqCst);
                    if callback_once.swap(false, Ordering::SeqCst) {
                        context.level.with_world_mut(|world| {
                            *retained_for_callback.lock().unwrap() =
                                Some(std::mem::replace(world, World::empty()));
                        });
                        return Err(CoreError::Initialization(
                            "original detached runtime".to_string(),
                            "exact original error".to_string(),
                        ));
                    }
                    Ok(())
                },
            )))
            .unwrap();
    });
    let error = try_run_test_stage(&core.handle(), &level)
        .unwrap_err()
        .into_parts()
        .1;
    match error {
        CoreError::Initialization(owner, message) => {
            assert_eq!(owner, "original detached runtime");
            assert_eq!(message, "exact original error");
        }
        other => panic!("unexpected detached runtime error: {other}"),
    }
    let original = retained.lock().unwrap().take().unwrap();
    level.replace(original);
    run_test_stage(&core.handle(), &level);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn detached_worker_schedule_returns_its_slot_discards_old_commands_and_recovers_after_reattach() {
    let core = CoreRuntime::new();
    let level = test_level(RuntimeExtensionRegistry::default());
    let retained = Arc::new(Mutex::new(None::<crate::scene::ecs::Schedule>));
    let retained_for_callback = Arc::clone(&retained);
    let owner = level.clone();
    let once = Arc::new(AtomicBool::new(true));
    let callback_once = Arc::clone(&once);
    let commands_applied = Arc::new(AtomicUsize::new(0));
    let commands_for_callback = Arc::clone(&commands_applied);
    level.with_world_mut(|world| {
        world
            .register_worldless_native_system::<CommandsParam, _>(
                "tests.detach.worker",
                SystemStage::Update,
                0,
                move |mut commands: crate::scene::ecs::Commands<'_>| {
                    let applied = Arc::clone(&commands_for_callback);
                    commands.queue_fn(move |_| {
                        applied.fetch_add(1, Ordering::SeqCst);
                    });
                    if callback_once.swap(false, Ordering::SeqCst) {
                        owner.with_world_mut(|world| {
                            *retained_for_callback.lock().unwrap() =
                                Some(std::mem::take(world.schedule_mut()));
                        });
                    }
                },
            )
            .unwrap();
    });
    assert!(try_run_test_stage(&core.handle(), &level).is_err());
    assert_eq!(commands_applied.load(Ordering::SeqCst), 0);
    let schedule = retained.lock().unwrap().take().unwrap();
    level.with_world_mut(|world| {
        *world.schedule_mut() = schedule;
    });
    run_test_stage(&core.handle(), &level);
    assert_eq!(commands_applied.load(Ordering::SeqCst), 1);
    assert!(level
        .with_world_mut(|world| world.unregister_native_system("tests.detach.worker"))
        .unwrap());
}

#[test]
fn direct_command_window_reports_replacement_cancellation_and_leaves_new_world_commands_untouched()
{
    let mut world = World::empty();
    let canceled_command = Arc::new(AtomicUsize::new(0));
    let fresh_marker = Arc::new(AtomicUsize::new(0));
    let marker = Arc::clone(&fresh_marker);
    world.commands().queue_fn(move |world| {
        *world = replacement_world(marker);
    });
    let canceled = Arc::clone(&canceled_command);
    world.commands().queue_fn(move |_| {
        canceled.fetch_add(1, Ordering::SeqCst);
    });
    let report = world.apply_deferred();
    assert!(!report.is_success());
    assert!(report.was_canceled_by_world_replacement());
    assert_eq!(canceled_command.load(Ordering::SeqCst), 0);
    assert_eq!(fresh_marker.load(Ordering::SeqCst), 0);
    assert!(world.apply_deferred().is_success());
    assert_eq!(fresh_marker.load(Ordering::SeqCst), 1);
}

#[derive(Debug)]
struct StructuralReplacementComponent;
zircon_runtime_interface::impl_retained_owner_v1!(StructuralReplacementComponent {});
impl Component for StructuralReplacementComponent {}
#[derive(Debug)]
struct CanceledStructuralComponent;
zircon_runtime_interface::impl_retained_owner_v1!(CanceledStructuralComponent {});
impl Component for CanceledStructuralComponent {}

#[test]
fn structural_lifecycle_replacement_cancels_old_observers_rows_and_following_closure() {
    let mut world = World::empty();
    let first = world.spawn_node(crate::scene::NodeKind::Empty).unwrap();
    let second = world.spawn_node(crate::scene::NodeKind::Empty).unwrap();
    let old_observers = Arc::new(AtomicUsize::new(0));
    let old_closure = Arc::new(AtomicUsize::new(0));
    let fresh_marker = Arc::new(AtomicUsize::new(0));
    let marker = Arc::clone(&fresh_marker);
    world.observe_component_lifecycle::<StructuralReplacementComponent>(
        LifecycleEventKind::Add,
        // SAFETY: This retires the current World or schedule. Remaining work is counter-only
        // scheduling; replacement history is never adopted.
        unsafe {
            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, _| {
                *world = replacement_world(Arc::clone(&marker));
            })
        },
    );
    let observers = Arc::clone(&old_observers);
    world.observe_component_lifecycle::<StructuralReplacementComponent>(
        LifecycleEventKind::Add,
        // SAFETY: Only an external atomic counter changes; retained World payload and dynamic
        // JSON are untouched.
        unsafe {
            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_, _| {
                observers.fetch_add(1, Ordering::SeqCst);
            })
        },
    );
    world
        .commands()
        .insert(first, StructuralReplacementComponent);
    world.commands().insert(second, CanceledStructuralComponent);
    let old = Arc::clone(&old_closure);
    world.commands().queue_fn(move |_| {
        old.fetch_add(1, Ordering::SeqCst);
    });
    let report = world.apply_deferred();
    assert!(report.was_canceled_by_world_replacement());
    assert!(!report.is_success());
    assert_eq!(old_observers.load(Ordering::SeqCst), 0);
    assert_eq!(old_closure.load(Ordering::SeqCst), 0);
    assert_eq!(fresh_marker.load(Ordering::SeqCst), 0);
    assert!(world.internal_entity(first).is_none());
    assert!(world.internal_entity(second).is_none());
    assert!(world.apply_deferred().is_success());
    assert_eq!(fresh_marker.load(Ordering::SeqCst), 1);
}

struct PanicOnReplace(Arc<AtomicBool>);
zircon_runtime_interface::impl_retained_owner_v1!(PanicOnReplace { 0 });
impl Component for PanicOnReplace {}
impl Drop for PanicOnReplace {
    fn drop(&mut self) {
        if self.0.swap(false, Ordering::SeqCst) {
            panic!("original dense component drop panic");
        }
    }
}

#[test]
fn dense_replacement_drop_panic_restores_lifecycle_staging_and_preserves_the_original_panic() {
    let mut world = World::empty();
    let entity = world.spawn_node(crate::scene::NodeKind::Empty).unwrap();
    world
        .insert(entity, PanicOnReplace(Arc::new(AtomicBool::new(true))))
        .unwrap();
    let observer_calls = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&observer_calls);
    world.observe_component_lifecycle::<CanceledStructuralComponent>(
        LifecycleEventKind::Add,
        // SAFETY: Only an external atomic counter changes; retained World payload and dynamic
        // JSON are untouched.
        unsafe {
            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_, _| {
                calls.fetch_add(1, Ordering::SeqCst);
            })
        },
    );
    world
        .commands()
        .insert(entity, PanicOnReplace(Arc::new(AtomicBool::new(false))));
    let panic = std::panic::catch_unwind(AssertUnwindSafe(|| world.apply_deferred())).unwrap_err();
    assert_eq!(
        panic.downcast_ref::<&'static str>(),
        Some(&"original dense component drop panic")
    );
    world.commands().insert(entity, CanceledStructuralComponent);
    assert!(world.apply_deferred().is_success());
    assert_eq!(observer_calls.load(Ordering::SeqCst), 1);
}

struct CallbackDropProbe(Arc<AtomicUsize>);
impl CallbackDropProbe {
    fn keep(&self) {}
}
impl Drop for CallbackDropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn destroyed_original_registry_drops_native_runtime_and_worker_returned_boxes_once() {
    for lane in 0..3 {
        let core = CoreRuntime::new();
        let level = test_level(RuntimeExtensionRegistry::default());
        let drops = Arc::new(AtomicUsize::new(0));
        let probe = CallbackDropProbe(Arc::clone(&drops));
        let owner = level.clone();
        level.with_world_mut(|world| match lane {
            0 => world
                .register_boxed_native_system(Box::new(WorldCallback::new(
                    SceneSystemMetadata::new("tests.drop.native", SystemStage::Update, 0),
                    move |world| {
                        probe.keep();
                        *world = World::empty();
                    },
                )))
                .unwrap(),
            1 => world
                .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                    SceneSystemMetadata::new("tests.drop.runtime", SystemStage::Update, 0),
                    move |context| {
                        probe.keep();
                        context.level.replace(World::empty());
                        Ok(())
                    },
                )))
                .unwrap(),
            2 => world
                .register_worldless_native_system::<(), _>(
                    "tests.drop.worker",
                    SystemStage::Update,
                    0,
                    move |()| {
                        probe.keep();
                        owner.replace(World::empty());
                    },
                )
                .unwrap(),
            _ => unreachable!(),
        });
        assert!(try_run_test_stage(&core.handle(), &level).is_err());
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn full_world_replacement_cancels_the_old_tick_even_when_it_moves_the_same_original_schedule_arc() {
    let core = CoreRuntime::new();
    let level = test_level(RuntimeExtensionRegistry::default());
    let once = Arc::new(AtomicBool::new(true));
    let once_for_callback = Arc::clone(&once);
    let later = Arc::new(AtomicUsize::new(0));
    let later_for_system = Arc::clone(&later);
    level.with_world_mut(|world| {
        world
            .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                SceneSystemMetadata::new("tests.same-arc.replace", SystemStage::Update, -10),
                move |context| {
                    if once_for_callback.swap(false, Ordering::SeqCst) {
                        context.level.with_world_mut(|world| {
                            let plan = world.schedule().stage_plan();
                            let schedule = std::mem::take(world.schedule_mut());
                            let mut replacement = World::empty();
                            *replacement.schedule_mut() = schedule;
                            assert!(Arc::ptr_eq(&plan, &replacement.schedule().stage_plan()));
                            *world = replacement;
                        });
                    }
                    Ok(())
                },
            )))
            .unwrap();
        world
            .register_native_system::<(), _>(
                "tests.same-arc.late",
                SystemStage::Update,
                10,
                move |()| {
                    later_for_system.fetch_add(1, Ordering::SeqCst);
                },
            )
            .unwrap();
    });
    assert!(try_run_test_stage(&core.handle(), &level).is_err());
    assert_eq!(later.load(Ordering::SeqCst), 0);
}

fn assert_rejected_command_errors_in_order(
    errors: &[crate::scene::ecs::DeferredCommandError],
    entity: EntityId,
    missing_parent: EntityId,
) {
    assert_eq!(errors.len(), 2);
    assert_eq!(
        errors[0].operation(),
        DeferredCommandOperation::InsertBundle
    );
    assert_eq!(errors[0].target(), &DeferredCommandTarget::Resolved(entity));
    assert!(matches!(
        errors[0].error(),
        crate::scene::SceneError::DuplicateBundleComponentType
    ));
    assert_eq!(
        errors[1].operation(),
        DeferredCommandOperation::ReparentChecked
    );
    assert_eq!(errors[1].target(), &DeferredCommandTarget::Resolved(entity));
    assert!(matches!(errors[1].error(),
        crate::scene::SceneError::MissingParent { child, parent }
        if *child == entity && *parent == missing_parent));
}

#[test]
fn rejected_queue_errors_survive_whole_world_replacement_without_touching_replacement_diagnostics()
{
    for panic_after_replacement in [false, true] {
        let mut world = World::empty();
        let entity = world.spawn(()).unwrap();
        let missing_parent = entity + 100;
        let replacement_missing_parent = entity + 200;
        let canceled_calls = Arc::new(AtomicUsize::new(0));
        let fresh_calls = Arc::new(AtomicUsize::new(0));
        {
            let mut commands = world.commands();
            commands.insert_bundle(
                entity,
                (
                    StructuralReplacementComponent,
                    StructuralReplacementComponent,
                ),
            );
            // This existing opaque command reports through World's error hook.
            // Its later error must follow the rejected structural bundle once.
            commands.set_parent_checked(entity, Some(missing_parent));
            let marker = Arc::clone(&fresh_calls);
            commands.queue_fn(move |world| {
                let mut replacement = replacement_world(marker);
                assert_eq!(replacement.spawn(()).unwrap(), entity);
                replacement.record_deferred_command_error(
                    crate::scene::ecs::DeferredCommandError::new(
                        DeferredCommandOperation::ReparentChecked,
                        DeferredCommandTarget::Resolved(entity),
                        crate::scene::SceneError::MissingParent {
                            child: entity,
                            parent: replacement_missing_parent,
                        },
                    ),
                );
                *world = replacement;
                if panic_after_replacement {
                    panic!("original whole-World command replacement panic");
                }
            });
            commands.insert(entity, CanceledStructuralComponent);
            let canceled = Arc::clone(&canceled_calls);
            commands.queue_fn(move |_| {
                canceled.fetch_add(1, Ordering::SeqCst);
            });
        }

        let result = std::panic::catch_unwind(AssertUnwindSafe(|| world.apply_deferred()));
        if panic_after_replacement {
            assert_eq!(
                result.unwrap_err().downcast_ref::<&'static str>(),
                Some(&"original whole-World command replacement panic")
            );
        } else {
            let report = result.unwrap();
            assert!(report.was_canceled_by_world_replacement());
            assert!(!report.is_success());
            assert_eq!(report.applied_count(), 3);
            assert_rejected_command_errors_in_order(report.errors(), entity, missing_parent);
        }
        assert_eq!(canceled_calls.load(Ordering::SeqCst), 0);
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
        assert!(world.contains_entity(entity));
        assert!(!world.contains_component::<StructuralReplacementComponent>(entity));
        assert!(!world.contains_component::<CanceledStructuralComponent>(entity));
        assert!(world.has_deferred_commands());
        // Read the sentinel only after the old window has completed. Its exact
        // presence proves that neither the success nor panic path drained the
        // replacement World's diagnostics into the original report.
        let replacement_errors = world.take_deferred_command_errors();
        assert_eq!(replacement_errors.len(), 1);
        assert!(matches!(replacement_errors[0].error(),
            crate::scene::SceneError::MissingParent { child, parent }
            if *child == entity && *parent == replacement_missing_parent));
        assert!(world.apply_deferred().is_success());
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn rejected_queue_errors_survive_schedule_replacement_and_original_owner_panic() {
    const SYSTEM_ID: &str = "tests.queue-report.replacement-native";
    for panic_after_replacement in [false, true] {
        let mut world = World::empty();
        let entity = world.spawn(()).unwrap();
        let missing_parent = entity + 100;
        let original_plan = world.schedule().stage_plan();
        let replacement_plan = Arc::new(Mutex::new(None));
        let canceled_calls = Arc::new(AtomicUsize::new(0));
        let fresh_calls = Arc::new(AtomicUsize::new(0));
        let system_calls = Arc::new(AtomicUsize::new(0));
        {
            let mut commands = world.commands();
            commands.insert_bundle(
                entity,
                (
                    StructuralReplacementComponent,
                    StructuralReplacementComponent,
                ),
            );
            commands.set_parent_checked(entity, Some(missing_parent));
            let marker = Arc::clone(&fresh_calls);
            let plan = Arc::clone(&replacement_plan);
            let system = Arc::clone(&system_calls);
            commands.queue_fn(move |world| {
                *world.schedule_mut() = Default::default();
                world
                    .register_native_system::<(), _>(SYSTEM_ID, SystemStage::Update, 0, move |()| {
                        system.fetch_add(1, Ordering::SeqCst);
                    })
                    .unwrap();
                *plan.lock().unwrap() = Some(world.schedule().stage_plan());
                world.commands().queue_fn(move |_| {
                    marker.fetch_add(1, Ordering::SeqCst);
                });
                if panic_after_replacement {
                    panic!("original schedule command replacement panic");
                }
            });
            commands.insert(entity, CanceledStructuralComponent);
            let canceled = Arc::clone(&canceled_calls);
            commands.queue_fn(move |_| {
                canceled.fetch_add(1, Ordering::SeqCst);
            });
        }

        let result = std::panic::catch_unwind(AssertUnwindSafe(|| world.apply_deferred()));
        if panic_after_replacement {
            assert_eq!(
                result.unwrap_err().downcast_ref::<&'static str>(),
                Some(&"original schedule command replacement panic")
            );
            // Same logical World still owns these diagnostics on panic, as it
            // did before the errors moved into the command-window local Vec.
            assert_rejected_command_errors_in_order(
                &world.take_deferred_command_errors(),
                entity,
                missing_parent,
            );
        } else {
            let report = result.unwrap();
            assert!(report.was_canceled_by_world_replacement());
            assert!(!report.is_success());
            assert_eq!(report.applied_count(), 3);
            assert_rejected_command_errors_in_order(report.errors(), entity, missing_parent);
            assert!(world.take_deferred_command_errors().is_empty());
        }
        let replacement_plan = replacement_plan.lock().unwrap().take().unwrap();
        assert!(!Arc::ptr_eq(&original_plan, &replacement_plan));
        assert!(Arc::ptr_eq(
            &replacement_plan,
            &world.schedule().stage_plan()
        ));
        assert_eq!(canceled_calls.load(Ordering::SeqCst), 0);
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
        assert_eq!(system_calls.load(Ordering::SeqCst), 0);
        assert!(world.contains_entity(entity));
        assert!(!world.contains_component::<StructuralReplacementComponent>(entity));
        assert!(!world.contains_component::<CanceledStructuralComponent>(entity));
        assert!(world.has_deferred_commands());
        assert!(world.apply_deferred().is_success());
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
        assert!(world.run_native_scene_system(SYSTEM_ID));
        assert_eq!(system_calls.load(Ordering::SeqCst), 1);
        assert!(world.unregister_native_system(SYSTEM_ID).unwrap());
    }
}

#[test]
fn rejected_queue_errors_survive_lifecycle_world_replacement_at_each_structural_finish_site() {
    for before_opaque_barrier in [false, true] {
        for panic_after_replacement in [false, true] {
            let mut world = World::empty();
            let entity = world.spawn(()).unwrap();
            let stale_entity = world.spawn(()).unwrap();
            let missing_parent = entity + 100;
            let replacement_missing_parent = entity + 200;
            let canceled_calls = Arc::new(AtomicUsize::new(0));
            let fresh_calls = Arc::new(AtomicUsize::new(0));
            let marker = Arc::clone(&fresh_calls);
            world.observe_component_lifecycle::<StructuralReplacementComponent>(
                LifecycleEventKind::Add,
                // SAFETY: Only a separate replacement World is built before owner retirement;
                // no retained payload of the original owner grows or adopts it.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, _| {
                        let mut replacement = replacement_world(Arc::clone(&marker));
                        assert_eq!(replacement.spawn(()).unwrap(), entity);
                        replacement.record_deferred_command_error(
                            crate::scene::ecs::DeferredCommandError::new(
                                DeferredCommandOperation::ReparentChecked,
                                DeferredCommandTarget::Resolved(entity),
                                crate::scene::SceneError::MissingParent {
                                    child: entity,
                                    parent: replacement_missing_parent,
                                },
                            ),
                        );
                        *world = replacement;
                        if panic_after_replacement {
                            panic!("original structural lifecycle replacement panic");
                        }
                    })
                },
            );
            {
                let mut commands = world.commands();
                commands.insert_bundle(
                    entity,
                    (
                        StructuralReplacementComponent,
                        StructuralReplacementComponent,
                    ),
                );
                commands.set_parent_checked(entity, Some(missing_parent));
                // This valid row's Add observer replaces World before the later
                // row can publish. The preceding checked error must already be
                // owned by the original command window when that callback runs.
                commands.insert(entity, StructuralReplacementComponent);
                commands.insert(stale_entity, CanceledStructuralComponent);
                if before_opaque_barrier {
                    let canceled = Arc::clone(&canceled_calls);
                    commands.queue_fn(move |_| {
                        canceled.fetch_add(1, Ordering::SeqCst);
                    });
                }
            }

            let result = std::panic::catch_unwind(AssertUnwindSafe(|| world.apply_deferred()));
            if panic_after_replacement {
                assert_eq!(
                    result.unwrap_err().downcast_ref::<&'static str>(),
                    Some(&"original structural lifecycle replacement panic")
                );
            } else {
                let report = result.unwrap();
                assert!(report.was_canceled_by_world_replacement());
                assert!(!report.is_success());
                assert_eq!(
                    report.applied_count(),
                    if before_opaque_barrier { 5 } else { 4 }
                );
                assert_rejected_command_errors_in_order(report.errors(), entity, missing_parent);
            }
            assert_eq!(canceled_calls.load(Ordering::SeqCst), 0);
            assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
            assert!(world.contains_entity(entity));
            assert!(!world.contains_entity(stale_entity));
            assert!(!world.contains_component::<StructuralReplacementComponent>(entity));
            assert!(!world.contains_component::<CanceledStructuralComponent>(entity));
            assert!(world.has_deferred_commands());
            let replacement_errors = world.take_deferred_command_errors();
            assert_eq!(replacement_errors.len(), 1);
            assert!(matches!(replacement_errors[0].error(),
                crate::scene::SceneError::MissingParent { child, parent }
                if *child == entity && *parent == replacement_missing_parent));
            assert!(world.apply_deferred().is_success());
            assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
            assert_eq!(canceled_calls.load(Ordering::SeqCst), 0);
        }
    }
}

mod checked_reparent_lifecycle_owner {
    use super::*;
    use crate::scene::components::{Hierarchy, Mobility};
    use crate::scene::ecs::{DeferredCommandError, Resource};
    use crate::scene::SceneError;
    use zircon_runtime_interface::world_sync::LifecycleTargetChange;

    #[derive(Debug, PartialEq, Eq)]
    struct ReparentSentinel(u32);
    impl Resource for ReparentSentinel {}

    fn fixture() -> (World, EntityId, EntityId, EntityId) {
        let mut world = World::empty();
        let parent = world.spawn(()).unwrap();
        let child = world.spawn(()).unwrap();
        let next_parent = world.spawn(()).unwrap();
        world.set_mobility(child, Mobility::Dynamic).unwrap();
        world.set_parent_checked(child, Some(parent)).unwrap();
        (world, parent, child, next_parent)
    }

    fn assert_prior_error(error: &DeferredCommandError, child: EntityId, missing_parent: EntityId) {
        assert_eq!(error.operation(), DeferredCommandOperation::ReparentChecked);
        assert_eq!(error.target(), &DeferredCommandTarget::Resolved(child));
        assert_eq!(
            error.error(),
            &SceneError::MissingParent {
                child,
                parent: missing_parent
            }
        );
    }

    #[test]
    fn real_checked_reparent_world_replacement_does_not_append_cancellation_to_fresh_diagnostics() {
        for retain_schedule in [false, true] {
            let (mut world, parent, child, next_parent) = fixture();
            let missing_parent = next_parent + 100;
            let (mut fresh, fresh_parent, fresh_child, fresh_next) = fixture();
            assert_eq!(
                (parent, child, next_parent),
                (fresh_parent, fresh_child, fresh_next)
            );
            assert_eq!(
                world.registered_component_id::<Hierarchy>(),
                fresh.registered_component_id::<Hierarchy>()
            );
            assert!(!world.owner_identity().is_same_world(fresh.owner_identity()));
            let generation = fresh.world_generation();
            fresh.insert_resource(ReparentSentinel(51));
            fresh.record_deferred_command_error(DeferredCommandError::new(
                DeferredCommandOperation::ReparentChecked,
                DeferredCommandTarget::Resolved(child),
                SceneError::Message("fresh reparent diagnostics".into()),
            ));
            let fresh_calls = Arc::new(AtomicUsize::new(0));
            let marker = Arc::clone(&fresh_calls);
            fresh.commands().queue_fn(move |_| {
                marker.fetch_add(1, Ordering::SeqCst);
            });
            let stale_calls = Arc::new(AtomicUsize::new(0));
            let fresh = Arc::new(Mutex::new(Some(fresh)));
            let original = world.schedule().stage_plan();
            world.observe_component_lifecycle::<Hierarchy>(
                LifecycleEventKind::Replace,
                // SAFETY: This retires the current World or schedule. Remaining work is
                // counter-only scheduling; replacement history is never adopted.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                        move |world, event| {
                            assert_eq!(event.entity(), child);
                            let mut fresh = fresh.lock().unwrap().take().unwrap();
                            if retain_schedule {
                                *fresh.schedule_mut() = std::mem::take(world.schedule_mut());
                                assert!(Arc::ptr_eq(&original, &fresh.schedule().stage_plan()));
                            }
                            *world = fresh;
                        },
                    )
                },
            );
            {
                let mut commands = world.commands();
                commands.set_parent_checked(child, Some(missing_parent));
                commands.set_parent_checked(child, Some(next_parent));
                let stale = Arc::clone(&stale_calls);
                commands.queue_fn(move |_| {
                    stale.fetch_add(1, Ordering::SeqCst);
                });
            }
            let report = world.apply_deferred();
            assert!(report.was_canceled_by_world_replacement());
            assert!(!report.is_success());
            assert_eq!(report.applied_count(), 2);
            assert_eq!(
                report.errors().len(),
                1,
                "the void opaque boundary retains prior errors, not the new replaced-World error"
            );
            assert_prior_error(&report.errors()[0], child, missing_parent);
            assert_eq!(stale_calls.load(Ordering::SeqCst), 0);
            assert_eq!(world.parent_of(child), Some(parent));
            assert_eq!(
                world.get_resource::<ReparentSentinel>(),
                Some(&ReparentSentinel(51))
            );
            assert_eq!(world.world_generation(), generation);
            let fresh_errors = world.take_deferred_command_errors();
            assert_eq!(fresh_errors.len(), 1);
            assert_eq!(
                fresh_errors[0].error(),
                &SceneError::Message("fresh reparent diagnostics".into())
            );
            assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
            assert!(world.apply_deferred().is_success());
            assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
        }
    }

    #[test]
    fn real_checked_reparent_schedule_replacement_retains_typed_cancellation_once_after_prior_error(
    ) {
        let (mut world, _, child, next_parent) = fixture();
        let missing_parent = next_parent + 100;
        let original_owner = world.owner_identity().clone();
        let fresh_calls = Arc::new(AtomicUsize::new(0));
        let systems = Arc::new(AtomicUsize::new(0));
        let stale_calls = Arc::new(AtomicUsize::new(0));
        let fresh = Arc::clone(&fresh_calls);
        let system = Arc::clone(&systems);
        world.observe_component_lifecycle::<Hierarchy>(
            LifecycleEventKind::Replace,
            // SAFETY: After World or schedule retirement, only fixed resources and counter-only
            // commands change; no replacement history is adopted.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |world, event| {
                    assert_eq!(event.entity(), child);
                    *world.schedule_mut() = Default::default();
                    let system = Arc::clone(&system);
                    world
                        .register_native_system::<(), _>(
                            "tests.checked-reparent.fresh",
                            SystemStage::Update,
                            0,
                            move |()| {
                                system.fetch_add(1, Ordering::SeqCst);
                            },
                        )
                        .unwrap();
                    world.insert_resource(ReparentSentinel(52));
                    let fresh = Arc::clone(&fresh);
                    world.commands().queue_fn(move |_| {
                        fresh.fetch_add(1, Ordering::SeqCst);
                    });
                })
            },
        );
        {
            let mut commands = world.commands();
            commands.set_parent_checked(child, Some(missing_parent));
            commands.set_parent_checked(child, Some(next_parent));
            let stale = Arc::clone(&stale_calls);
            commands.queue_fn(move |_| {
                stale.fetch_add(1, Ordering::SeqCst);
            });
        }
        let report = world.apply_deferred();
        assert!(report.was_canceled_by_world_replacement());
        assert_eq!(report.errors().len(), 2);
        assert_prior_error(&report.errors()[0], child, missing_parent);
        assert_eq!(
            report.errors()[1].operation(),
            DeferredCommandOperation::ReparentChecked
        );
        assert_eq!(
            report.errors()[1].target(),
            &DeferredCommandTarget::Resolved(child)
        );
        assert_eq!(
            report.errors()[1].error(),
            &SceneError::LifecycleTargetReplaced {
                operation: "insert component",
                target: LifecycleTargetChange::Schedule,
            }
        );
        assert!(world.take_deferred_command_errors().is_empty());
        assert!(original_owner.is_same_world(world.owner_identity()));
        assert_eq!(
            world.parent_of(child),
            Some(next_parent),
            "original row publication is not rolled back"
        );
        assert_eq!(
            world.get_resource::<ReparentSentinel>(),
            Some(&ReparentSentinel(52))
        );
        assert_eq!(stale_calls.load(Ordering::SeqCst), 0);
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
        assert!(world.apply_deferred().is_success());
        assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
        assert!(world.run_native_scene_system("tests.checked-reparent.fresh"));
        assert_eq!(systems.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn real_checked_reparent_valid_registration_keeps_phases_and_command_tail_on_original_plan() {
        let (mut world, _, child, next_parent) = fixture();
        let missing_parent = next_parent + 100;
        let original = world.schedule().stage_plan();
        let callbacks = Arc::new(AtomicUsize::new(0));
        let systems = Arc::new(AtomicUsize::new(0));
        let tail_calls = Arc::new(AtomicUsize::new(0));
        for kind in [LifecycleEventKind::Replace, LifecycleEventKind::Insert] {
            let callbacks = Arc::clone(&callbacks);
            let systems = Arc::clone(&systems);
            let original = Arc::clone(&original);
            world.observe_component_lifecycle::<Hierarchy>(
                kind,
                // SAFETY: The local callback only observes and registers a system that
                // increments an external counter; no history payload grows.
                unsafe {
                    use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                    RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                        move |world, event| {
                            assert_eq!(event.entity(), child);
                            assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                            if callbacks.fetch_add(1, Ordering::SeqCst) == 0 {
                                let systems = Arc::clone(&systems);
                                world
                                    .register_native_system::<(), _>(
                                        "tests.checked-reparent.pending",
                                        SystemStage::Update,
                                        0,
                                        move |()| {
                                            systems.fetch_add(1, Ordering::SeqCst);
                                        },
                                    )
                                    .unwrap();
                            }
                            assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                        },
                    )
                },
            );
        }
        {
            let mut commands = world.commands();
            commands.set_parent_checked(child, Some(missing_parent));
            commands.set_parent_checked(child, Some(next_parent));
            let tail = Arc::clone(&tail_calls);
            commands.queue_fn(move |_| {
                tail.fetch_add(1, Ordering::SeqCst);
            });
        }
        let report = world.apply_deferred();
        assert!(!report.was_canceled_by_world_replacement());
        assert_eq!(report.errors().len(), 1);
        assert_prior_error(&report.errors()[0], child, missing_parent);
        assert_eq!(callbacks.load(Ordering::SeqCst), 2);
        assert_eq!(tail_calls.load(Ordering::SeqCst), 1);
        assert_eq!(world.parent_of(child), Some(next_parent));
        assert!(!Arc::ptr_eq(&original, &world.schedule().stage_plan()));
        assert!(world.run_native_scene_system("tests.checked-reparent.pending"));
        assert_eq!(systems.load(Ordering::SeqCst), 1);
    }
}
