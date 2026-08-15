use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkerCommandCallbackOrder(Vec<u8>);

impl Resource for WorkerCommandCallbackOrder {}

#[test]
fn worker_command_callbacks_merge_once_in_compiled_order_before_apply() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry.intern_plugin_module("tests.runtime").unwrap();
    registry
        .register_external_native_command_system(
            owner,
            "tests.worker_command.z",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            || {
                |commands| {
                    commands.push(|world: &mut World| {
                        world
                            .get_resource_mut::<WorkerCommandCallbackOrder>()
                            .expect("earlier worker command callbacks should apply first")
                            .0
                            .push(3);
                    });
                }
            },
        )
        .with_order(30)
        .with_command_capacity(1)
        .register()
        .unwrap();
    registry
        .register_external_native_command_system(
            owner,
            "tests.worker_command.b",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            || {
                |commands| {
                    commands.push(|world: &mut World| {
                        world
                            .get_resource_mut::<WorkerCommandCallbackOrder>()
                            .expect("first worker command callback should apply first")
                            .0
                            .push(2);
                    });
                }
            },
        )
        .with_order(10)
        .with_command_capacity(1)
        .register()
        .unwrap();
    registry
        .register_external_native_command_system(
            owner,
            "tests.worker_command.a",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            || {
                |commands| {
                    commands.push(|world: &mut World| {
                        world.insert_resource(WorkerCommandCallbackOrder(vec![1]));
                    });
                }
            },
        )
        .with_order(10)
        .with_command_capacity(1)
        .register()
        .unwrap();
    let core = CoreRuntime::new();
    let level = test_level(registry);

    run_test_stage(&core.handle(), &level);

    assert_eq!(
        level.with_world(|world| world.get_resource::<WorkerCommandCallbackOrder>().cloned()),
        Some(WorkerCommandCallbackOrder(vec![1, 2, 3]))
    );
    let diagnostics = level.with_world(|world| {
        world
            .ecs_frame_performance_diagnostics()
            .native_system_schedule
    });
    assert_eq!(diagnostics.worker_batch_count(), 1);
    assert_eq!(diagnostics.callback_count(), 3);
    assert_eq!(diagnostics.temporary_control_buffer_count(), 3);
    let two_buffer_payload_bytes = 3usize
        .saturating_mul(std::mem::size_of::<BoxedSceneSystem>())
        .saturating_add(3usize.saturating_mul(std::mem::size_of::<NativeSystemCallbackTiming>()))
        as u64;
    assert!(
        diagnostics.temporary_control_buffer_bytes() > two_buffer_payload_bytes,
        "the command-buffer reference container must contribute to the capacity proxy"
    );
}
