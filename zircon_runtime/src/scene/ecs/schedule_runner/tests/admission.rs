use super::*;
use crate::scene::ecs::{
    EventReaderParam, FunctionRuntimeSceneSystem, FunctionSceneSystem, InternalSceneSystem,
    SceneSystem, SceneSystemMetadata, ScheduleError, SystemOrderingConstraint, SystemRef,
};

pub(super) struct WorldCallback {
    pub(super) metadata: SceneSystemMetadata,
    pub(super) access: SystemParamAccess,
    pub(super) callback: Box<dyn FnMut(&mut World) + Send>,
}

impl WorldCallback {
    pub(super) fn new(
        metadata: SceneSystemMetadata,
        callback: impl FnMut(&mut World) + Send + 'static,
    ) -> Self {
        let mut access = SystemParamAccess::default();
        access.add_conservative_world_access();
        Self {
            metadata,
            access,
            callback: Box::new(callback),
        }
    }
}

impl SceneSystem for WorldCallback {
    fn metadata(&self) -> &SceneSystemMetadata {
        &self.metadata
    }
    fn access(&self) -> &SystemParamAccess {
        &self.access
    }
    fn run(&mut self, world: &mut World) {
        (self.callback)(world);
    }
}

fn descriptor(id: &str, target: &str) -> crate::scene::ecs::SceneSystemDescriptor {
    crate::scene::ecs::SceneSystemDescriptor::new(
        id,
        SystemStage::Update,
        InternalSceneSystem::NodeCache,
    )
    .after(SystemRef::System(target.to_string()))
}

#[test]
fn native_callback_rejects_inflight_cycle_rolls_back_and_runs_valid_additions_later() {
    let mut world = World::empty();
    let once = Arc::new(AtomicBool::new(true));
    let new_calls = Arc::new(AtomicUsize::new(0));
    let once_for_callback = Arc::clone(&once);
    let new_calls_for_callback = Arc::clone(&new_calls);
    world
        .register_boxed_native_system(Box::new(WorldCallback::new(
            SceneSystemMetadata::new("tests.admission.native", SystemStage::Update, 0),
            move |world| {
                if !once_for_callback.swap(false, Ordering::SeqCst) {
                    return;
                }
                let original = world.schedule().stage_plan();
                let receipt = world.schedule().build_receipt();
                world
                    .schedule_mut()
                    .register_system(descriptor("tests.admission.a", "tests.admission.b"))
                    .unwrap();
                let error = world
                    .schedule_mut()
                    .register_system(descriptor("tests.admission.b", "tests.admission.a"))
                    .unwrap_err();
                assert!(matches!(error, ScheduleError::OrderingCycle { .. }));
                assert!(!world
                    .schedule()
                    .systems()
                    .iter()
                    .any(|s| s.id == "tests.admission.b"));
                assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                assert_eq!(receipt, world.schedule().build_receipt());
                let calls = Arc::clone(&new_calls_for_callback);
                world
                    .register_native_system::<(), _>(
                        "tests.admission.b",
                        SystemStage::Update,
                        1,
                        move |()| {
                            calls.fetch_add(1, Ordering::SeqCst);
                        },
                    )
                    .unwrap();
                assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
            },
        )))
        .unwrap();
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    assert_eq!(new_calls.load(Ordering::SeqCst), 0);
    assert!(world
        .schedule()
        .systems()
        .iter()
        .any(|s| s.id == "tests.admission.a"));
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    assert_eq!(new_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn taken_native_constraints_still_reject_cycle_and_cross_stage_additions() {
    let mut world = World::empty();
    let once = Arc::new(AtomicBool::new(true));
    let callback_once = Arc::clone(&once);
    world
        .register_boxed_native_system(Box::new(WorldCallback::new(
            SceneSystemMetadata::new("tests.admission.taken-native", SystemStage::Update, 0)
                .with_constraints([SystemOrderingConstraint::After(SystemRef::System(
                    "tests.admission.cycle-target".to_string(),
                ))]),
            move |world| {
                if !callback_once.swap(false, Ordering::SeqCst) {
                    return;
                }
                assert!(matches!(
                    world.schedule_mut().register_system(descriptor(
                        "tests.admission.cycle-target",
                        "tests.admission.taken-native",
                    )),
                    Err(ScheduleError::OrderingCycle { .. })
                ));
                let cross = crate::scene::ecs::SceneSystemDescriptor::new(
                    "tests.admission.cross-stage",
                    SystemStage::PostUpdate,
                    InternalSceneSystem::NodeCache,
                )
                .after(SystemRef::System(
                    "tests.admission.taken-native".to_string(),
                ));
                assert!(matches!(
                    world.schedule_mut().register_system(cross),
                    Err(ScheduleError::CrossStageConstraint { .. })
                ));
                world
                    .register_native_system::<(), _>(
                        "tests.admission.cycle-target",
                        SystemStage::Update,
                        0,
                        |()| {},
                    )
                    .unwrap();
            },
        )))
        .unwrap();
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    assert!(world
        .unregister_native_system("tests.admission.cycle-target")
        .unwrap());
}

#[derive(Debug)]
struct AdmissionEvent;

#[test]
fn runtime_callback_validates_taken_definition_and_releases_rejected_native_reader() {
    let level = test_level(RuntimeExtensionRegistry::default());
    let once = Arc::new(AtomicBool::new(true));
    let callback_once = Arc::clone(&once);
    let valid_calls = Arc::new(AtomicUsize::new(0));
    let callback_calls = Arc::clone(&valid_calls);
    level.with_world_mut(|world| {
        world
            .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                SceneSystemMetadata::new("tests.admission.runtime", SystemStage::Update, 0)
                    .with_constraints([SystemOrderingConstraint::After(SystemRef::System(
                        "tests.admission.reader".to_string(),
                    ))]),
                move |context| {
                    if !callback_once.swap(false, Ordering::SeqCst) {
                        return Ok(());
                    }
                    context.level.with_world_mut(|world| {
                        let original = world.schedule().stage_plan();
                        world
                            .schedule_mut()
                            .register_system(descriptor(
                                "tests.admission.runtime-a",
                                "tests.admission.runtime-b",
                            ))
                            .unwrap();
                        assert!(matches!(
                            world.schedule_mut().register_system(descriptor(
                                "tests.admission.runtime-b",
                                "tests.admission.runtime-a"
                            )),
                            Err(ScheduleError::OrderingCycle { .. })
                        ));
                        let reader =
                            FunctionSceneSystem::<EventReaderParam<AdmissionEvent>, _>::new(
                                SceneSystemMetadata::new(
                                    "tests.admission.reader",
                                    SystemStage::Update,
                                    1,
                                )
                                .with_constraints([
                                    SystemOrderingConstraint::After(SystemRef::System(
                                        "tests.admission.runtime".to_string(),
                                    )),
                                ]),
                                world,
                                |_: crate::scene::ecs::EventReader<'_, AdmissionEvent>| {},
                            )
                            .unwrap();
                        world.register_event::<AdmissionEvent>();
                        let event_id = world
                            .event_type_id::<AdmissionEvent>()
                            .expect("admission event should be registered");
                        assert_eq!(world.event_reader_count(event_id), Some(1));
                        assert!(matches!(
                            world.register_boxed_native_system(Box::new(reader)),
                            Err(ScheduleError::OrderingCycle { .. })
                        ));
                        assert_eq!(world.event_reader_count(event_id), Some(0));
                        let calls = Arc::clone(&callback_calls);
                        world
                            .register_native_system::<(), _>(
                                "tests.admission.reader",
                                SystemStage::Update,
                                1,
                                move |()| {
                                    calls.fetch_add(1, Ordering::SeqCst);
                                },
                            )
                            .unwrap();
                        world
                            .schedule_mut()
                            .register_system(crate::scene::ecs::SceneSystemDescriptor::new(
                                "tests.admission.runtime-b",
                                SystemStage::Update,
                                InternalSceneSystem::NodeCache,
                            ))
                            .unwrap();
                        assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                    });
                    Ok(())
                },
            )))
            .unwrap();
    });
    let core = CoreRuntime::new();
    run_test_stage(&core.handle(), &level);
    assert_eq!(valid_calls.load(Ordering::SeqCst), 0);
    run_test_stage(&core.handle(), &level);
    assert_eq!(valid_calls.load(Ordering::SeqCst), 1);
}

#[test]
fn original_native_panic_survives_rejected_cycle_and_schedule_is_reusable() {
    let mut world = World::empty();
    let once = Arc::new(AtomicBool::new(true));
    let callback_once = Arc::clone(&once);
    world
        .register_boxed_native_system(Box::new(WorldCallback::new(
            SceneSystemMetadata::new("tests.admission.panic", SystemStage::Update, 0),
            move |world| {
                if !callback_once.swap(false, Ordering::SeqCst) {
                    return;
                }
                world
                    .schedule_mut()
                    .register_system(descriptor(
                        "tests.admission.panic-a",
                        "tests.admission.panic-b",
                    ))
                    .unwrap();
                assert!(matches!(
                    world.schedule_mut().register_system(descriptor(
                        "tests.admission.panic-b",
                        "tests.admission.panic-a"
                    )),
                    Err(ScheduleError::OrderingCycle { .. })
                ));
                panic!("original native admission panic");
            },
        )))
        .unwrap();
    let payload = std::panic::catch_unwind(AssertUnwindSafe(|| {
        world.run_native_scene_systems_for_stage(SystemStage::Update);
    }))
    .unwrap_err();
    assert_eq!(
        payload.downcast_ref::<&'static str>(),
        Some(&"original native admission panic")
    );
    world.run_native_scene_systems_for_stage(SystemStage::Update);
    assert!(world
        .unregister_native_system("tests.admission.panic")
        .unwrap());
}

#[test]
fn whole_tick_pins_later_stage_callbacks_and_releases_after_success() {
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
    let late_calls = Arc::new(AtomicUsize::new(0));
    let fresh_calls = Arc::new(AtomicUsize::new(0));
    let once = Arc::new(AtomicBool::new(true));
    let late_calls_for_system = Arc::clone(&late_calls);
    let fresh_calls_for_callback = Arc::clone(&fresh_calls);
    level.with_world_mut(|world| {
        world
            .register_native_system::<(), _>("tests.tick.late", SystemStage::Update, 0, move |()| {
                late_calls_for_system.fetch_add(1, Ordering::SeqCst);
            })
            .unwrap();
        world
            .register_boxed_runtime_scene_system(Box::new(FunctionRuntimeSceneSystem::new(
                SceneSystemMetadata::new("tests.tick.first", SystemStage::First, 0),
                move |context| {
                    if once.swap(false, Ordering::SeqCst) {
                        context.level.with_world_mut(|world| {
                            assert_eq!(
                                world.unregister_native_system("tests.tick.late"),
                                Err(ScheduleError::SystemInFlight("tests.tick.late".to_string()))
                            );
                            let original = world.schedule().stage_plan();
                            let calls = Arc::clone(&fresh_calls_for_callback);
                            world
                                .register_native_system::<(), _>(
                                    "tests.tick.fresh",
                                    SystemStage::Update,
                                    1,
                                    move |()| {
                                        calls.fetch_add(1, Ordering::SeqCst);
                                    },
                                )
                                .unwrap();
                            assert!(Arc::ptr_eq(&original, &world.schedule().stage_plan()));
                        });
                    }
                    Ok(())
                },
            )))
            .unwrap();
    });
    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(1), 1),
        )
        .unwrap();
    assert_eq!(late_calls.load(Ordering::SeqCst), 1);
    assert_eq!(fresh_calls.load(Ordering::SeqCst), 0);
    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(1), 1),
        )
        .unwrap();
    assert_eq!(fresh_calls.load(Ordering::SeqCst), 1);
    assert!(level
        .with_world_mut(|world| world.unregister_native_system("tests.tick.late"))
        .unwrap());
}

#[test]
fn execution_guard_releases_original_schedule_after_its_world_moves() {
    let mut world = World::empty();
    world
        .register_native_system::<(), _>("tests.tick.moved", SystemStage::Update, 0, |()| {})
        .unwrap();
    let original = world.schedule().stage_plan();
    let execution = world.begin_schedule_execution();
    world
        .schedule_mut()
        .register_system(crate::scene::ecs::SceneSystemDescriptor::new(
            "tests.tick.pending",
            SystemStage::Update,
            InternalSceneSystem::NodeCache,
        ))
        .unwrap();
    let mut moved = world;
    assert!(Arc::ptr_eq(&original, &moved.schedule().stage_plan()));
    assert!(matches!(
        moved.unregister_native_system("tests.tick.moved"),
        Err(ScheduleError::SystemInFlight(_))
    ));
    drop(execution);
    assert!(!Arc::ptr_eq(&original, &moved.schedule().stage_plan()));
    assert!(moved.unregister_native_system("tests.tick.moved").unwrap());
}
