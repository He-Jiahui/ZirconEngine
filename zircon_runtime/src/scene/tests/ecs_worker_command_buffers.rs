use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::World;
use crate::scene::ecs::{
    CommandQueue, Resource, SceneSystemThreadAffinity, SystemParamAccess, SystemStage,
    WorkerCommandBuffer,
};

#[derive(Debug, PartialEq, Eq)]
struct WorkerMergeOrder(Vec<u8>);

impl Resource for WorkerMergeOrder {}

#[test]
fn worker_command_buffers_merge_by_compiled_order_not_worker_completion_order() {
    let mut buffers = [
        WorkerCommandBuffer::with_capacity(30, "tests.worker.z", 1),
        WorkerCommandBuffer::with_capacity(10, "tests.worker.b", 1),
        WorkerCommandBuffer::with_capacity(10, "tests.worker.a", 1),
    ];
    buffers[0].push(|world: &mut World| {
        world
            .get_resource_mut::<WorkerMergeOrder>()
            .expect("all earlier local buffers should merge first")
            .0
            .push(3);
    });
    buffers[1].push(|world: &mut World| {
        world
            .get_resource_mut::<WorkerMergeOrder>()
            .expect("first local buffer should merge first")
            .0
            .push(2);
    });
    buffers[2].push(|world: &mut World| {
        world.insert_resource(WorkerMergeOrder(vec![1]));
    });

    let mut queue = CommandQueue::default();
    queue
        .merge_worker_buffers(&mut buffers)
        .expect("unique compiled system keys must merge");
    let mut world = World::empty();
    let report = queue.apply(&mut world);

    assert_eq!(report.applied_count(), 3);
    assert!(buffers.iter().all(WorkerCommandBuffer::is_empty));
    assert_eq!(
        world.get_resource::<WorkerMergeOrder>(),
        Some(&WorkerMergeOrder(vec![1, 2, 3]))
    );
}

#[test]
fn duplicate_worker_command_buffer_keys_fail_before_consuming_work() {
    let mut buffers = [
        WorkerCommandBuffer::with_capacity(10, "tests.worker.duplicate", 1),
        WorkerCommandBuffer::with_capacity(10, "tests.worker.duplicate", 1),
    ];
    buffers[0].push(|_: &mut World| {});
    buffers[1].push(|_: &mut World| {});

    let mut queue = CommandQueue::default();
    let error = queue
        .merge_worker_buffers(&mut buffers)
        .expect_err("duplicate compiled system keys must fail closed");

    assert_eq!(error.system_order(), 10);
    assert_eq!(error.system_id(), "tests.worker.duplicate");
    assert!(queue.is_empty());
    assert!(buffers.iter().all(|buffer| !buffer.is_empty()));
}

#[test]
fn direct_native_stage_helper_flushes_worker_command_callbacks_at_stage_end() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("tests.worker_commands")
        .unwrap();
    registry
        .register_external_native_command_system(
            owner,
            "tests.worker_commands.stage_end",
            SystemStage::Update,
            SceneSystemThreadAffinity::WorkerSafe,
            |_world| Ok(SystemParamAccess::default()),
            |commands| {
                commands.push(|world: &mut World| {
                    world.insert_resource(WorkerMergeOrder(vec![7]));
                });
            },
        )
        .register()
        .unwrap();
    let mut world = World::empty();
    registry.apply_to_world(&mut world).unwrap();

    world.run_native_scene_systems_for_stage(SystemStage::Update);

    assert_eq!(
        world.get_resource::<WorkerMergeOrder>(),
        Some(&WorkerMergeOrder(vec![7]))
    );
}
