use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::World;
use crate::scene::ecs::{
    CommandQueue, DeferredSystemKey, Resource, SceneSystemThreadAffinity, SystemParamAccess,
    SystemStage, WorkerCommandBuffer,
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
fn worker_command_buffers_use_rebound_topological_key_not_registration_order() {
    let mut buffers = [
        WorkerCommandBuffer::with_capacity(-100, "tests.worker.consumer", 1),
        WorkerCommandBuffer::with_capacity(100, "tests.worker.producer", 1),
    ];
    buffers[0].bind_compiled_key(DeferredSystemKey::compiled(
        SystemStage::Update.rank(),
        1,
        "tests.worker.consumer",
    ));
    buffers[1].bind_compiled_key(DeferredSystemKey::compiled(
        SystemStage::Update.rank(),
        0,
        "tests.worker.producer",
    ));
    buffers[0].push(|world: &mut World| {
        world
            .get_resource_mut::<WorkerMergeOrder>()
            .expect("compiled producer must publish before consumer")
            .0
            .push(2);
    });
    buffers[1].push(|world: &mut World| {
        world.insert_resource(WorkerMergeOrder(vec![1]));
    });

    let mut queue = CommandQueue::default();
    queue
        .merge_worker_buffers(&mut buffers)
        .expect("unique compiled keys must merge");
    let mut world = World::empty();
    let report = queue.apply(&mut world);

    assert_eq!(report.applied_count(), 2);
    assert_eq!(
        world.get_resource::<WorkerMergeOrder>(),
        Some(&WorkerMergeOrder(vec![1, 2]))
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

    assert_eq!(error.plan_order(), 10);
    assert_eq!(error.system_id(), "tests.worker.duplicate");
    assert!(queue.is_empty());
    assert!(buffers.iter().all(|buffer| !buffer.is_empty()));
}

#[test]
fn worker_command_buffer_reclaims_drained_packed_blocks_before_requeue() {
    let mut worker = WorkerCommandBuffer::with_capacity(10, "tests.worker.reclaim", 1);
    worker.push(|_: &mut World| {});

    let mut destination = CommandQueue::default();
    destination
        .merge_worker_buffers(std::slice::from_mut(&mut worker))
        .expect("one worker buffer must merge");
    destination.apply(&mut World::empty());
    worker.reclaim_after_apply(&mut destination);

    worker.push(|_: &mut World| {});

    assert_eq!(worker.metrics().inline_block_storage_growths(), 0);
    destination
        .merge_worker_buffers(std::slice::from_mut(&mut worker))
        .expect("reclaimed worker buffer must merge again");
    let report = destination.apply(&mut World::empty());
    worker.reclaim_after_apply(&mut destination);

    assert_eq!(report.applied_count(), 1);
}

#[test]
fn worker_command_buffer_defers_reclaim_until_its_queued_work_is_applied() {
    let mut worker = WorkerCommandBuffer::with_capacity(10, "tests.worker.defer_reclaim", 1);
    worker.push(|_: &mut World| {});

    let mut destination = CommandQueue::default();
    destination
        .merge_worker_buffers(std::slice::from_mut(&mut worker))
        .expect("one worker buffer must merge");
    worker.reclaim_after_apply(&mut destination);

    worker.push(|_: &mut World| {});
    destination
        .merge_worker_buffers(std::slice::from_mut(&mut worker))
        .expect("worker work must remain mergeable before its barrier");
    let report = destination.apply(&mut World::empty());
    worker.reclaim_after_apply(&mut destination);

    assert_eq!(report.applied_count(), 2);
}

#[test]
fn command_queue_append_preserves_worker_arena_payloads_from_nested_work() {
    let mut worker = WorkerCommandBuffer::with_capacity(10, "tests.worker.nested", 1);
    worker.push(|world: &mut World| {
        world.insert_resource(WorkerMergeOrder(vec![9]));
    });

    let mut nested = CommandQueue::default();
    nested
        .merge_worker_buffers(std::slice::from_mut(&mut worker))
        .expect("one worker buffer must merge into nested work");
    let mut destination = CommandQueue::default();
    destination.append(&mut nested);

    let mut world = World::empty();
    let report = destination.apply(&mut world);

    assert_eq!(report.applied_count(), 1);
    assert_eq!(
        world.get_resource::<WorkerMergeOrder>(),
        Some(&WorkerMergeOrder(vec![9]))
    );
}

#[test]
fn command_queue_append_coalesces_matching_worker_arena_provenance() {
    let mut first_worker = WorkerCommandBuffer::with_capacity(10, "tests.worker.shared", 1);
    first_worker.push(|world: &mut World| {
        world.insert_resource(WorkerMergeOrder(vec![1]));
    });
    let mut destination = CommandQueue::default();
    destination
        .merge_worker_buffers(std::slice::from_mut(&mut first_worker))
        .expect("first worker buffer must merge");

    let mut second_worker = WorkerCommandBuffer::with_capacity(10, "tests.worker.shared", 1);
    second_worker.push(|world: &mut World| {
        world
            .get_resource_mut::<WorkerMergeOrder>()
            .expect("first worker command must run first")
            .0
            .push(2);
    });
    let mut nested = CommandQueue::default();
    nested
        .merge_worker_buffers(std::slice::from_mut(&mut second_worker))
        .expect("second worker buffer must merge into nested work");

    destination.append(&mut nested);
    let mut world = World::empty();
    let report = destination.apply(&mut world);

    assert_eq!(report.applied_count(), 2);
    assert_eq!(
        world.get_resource::<WorkerMergeOrder>(),
        Some(&WorkerMergeOrder(vec![1, 2]))
    );
}

#[test]
fn direct_native_callback_panic_discards_local_commands_before_retry() {
    let panic_once = Arc::new(AtomicBool::new(true));
    let published = Arc::new(AtomicUsize::new(0));
    let panic_once_for_system = Arc::clone(&panic_once);
    let published_for_system = Arc::clone(&published);
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("tests.worker_commands")
        .unwrap();
    registry
        .register_external_native_command_system(
            owner,
            "tests.worker_commands.direct_callback_panic",
            SystemStage::Update,
            SceneSystemThreadAffinity::MainThreadOnly,
            |_world| Ok(SystemParamAccess::default()),
            move || {
                let panic_once = Arc::clone(&panic_once_for_system);
                let published = Arc::clone(&published_for_system);
                move |commands| {
                    let published = Arc::clone(&published);
                    commands.push(move |_world: &mut World| {
                        published.fetch_add(1, Ordering::SeqCst);
                    });
                    if panic_once.swap(false, Ordering::SeqCst) {
                        panic!("intentional direct callback panic after enqueue");
                    }
                }
            },
        )
        .register()
        .unwrap();
    let mut world = World::empty();
    registry.apply_to_world(&mut world).unwrap();

    let first = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        world.run_native_scene_system("tests.worker_commands.direct_callback_panic")
    }));
    assert!(first.is_err());
    assert_eq!(published.load(Ordering::SeqCst), 0);

    assert!(world.run_native_scene_system("tests.worker_commands.direct_callback_panic"));
    let report = world.apply_deferred();

    assert_eq!(report.applied_count(), 1);
    assert_eq!(published.load(Ordering::SeqCst), 1);
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
            || {
                |commands| {
                    commands.push(|world: &mut World| {
                        world.insert_resource(WorkerMergeOrder(vec![7]));
                    });
                }
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
