use std::sync::{Arc, Mutex};

use crate::core::{CoreRuntime, JobScheduler, TaskPools};
use crate::scene::World;
use crate::scene::ecs::{
    Component, QueryState, ResMutParam, ResParam, Resource, SCHEDULE_PARALLEL_BATCHES_DIAGNOSTIC,
    SCHEDULE_SERIAL_FALLBACKS_DIAGNOSTIC, ScheduleConflictGraph, ScheduleConflictNode,
    ScheduleParallelBatch, ScheduleParallelExecutor, ScheduleParallelExecutorError,
    ScheduleParallelTaskRegistry, SystemParamAccess, SystemStage, SystemState,
};

#[derive(Debug, PartialEq, Eq)]
struct ScheduleHealth(u32);

impl Component for ScheduleHealth {}

#[derive(Debug, PartialEq, Eq)]
struct SchedulePlayer;

impl Component for SchedulePlayer {}

#[derive(Debug, PartialEq, Eq)]
struct ScheduleFrameCounter(u32);

impl Resource for ScheduleFrameCounter {}

fn test_job_scheduler() -> JobScheduler {
    JobScheduler::from_pool(TaskPools::process_default().compute().clone())
}

#[test]
fn schedule_parallel_executor_runs_registered_batches_through_job_scheduler() {
    let mut world = World::empty();
    world.spawn((ScheduleHealth(1), SchedulePlayer)).unwrap();
    world.insert_resource(ScheduleFrameCounter(0));
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let read_counter = SystemState::<ResParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::Update,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "read.counter",
            SystemStage::Update,
            read_counter.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::Update,
            write_health.access().clone(),
        ),
    ]);
    let batches = graph.conservative_parallel_batches();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    for system_id in ["read.health", "read.counter", "write.health"] {
        let observed = observed.clone();
        registry.register(system_id, move || {
            observed.lock().unwrap().push(system_id);
            Ok(())
        });
    }
    let executor = ScheduleParallelExecutor::new(test_job_scheduler().with_diagnostics());

    let report = executor
        .run_batches_with_report(&batches, &registry)
        .unwrap();

    assert!(executor.scheduler().parallelism() >= 1);
    assert!(executor.parallel_enabled());
    assert_eq!(report.parallel_batches(), 1);
    assert_eq!(report.serial_batches(), 1);
    assert_eq!(report.serial_fallbacks(), 0);
    assert_eq!(report.executed_systems(), 3);
    assert!(registry.contains("write.health"));
    let observed = observed.lock().unwrap();
    assert_eq!(observed.len(), 3);
    let mut first_batch = observed[..2].to_vec();
    first_batch.sort_unstable();
    assert_eq!(first_batch, vec!["read.counter", "read.health"]);
    assert_eq!(observed[2], "write.health");
}

#[test]
fn schedule_parallel_executor_can_run_parallel_batches_serially_with_report() {
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
        ScheduleConflictNode::new(
            "read.counter",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::PostUpdate,
            SystemParamAccess::default(),
        ),
    ]);
    let batches = graph.conservative_parallel_batches();
    assert_eq!(batches.len(), 2);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    for system_id in ["read.health", "read.counter", "write.health"] {
        let observed = observed.clone();
        registry.register(system_id, move || {
            observed.lock().unwrap().push(system_id);
            Ok(())
        });
    }
    let executor = ScheduleParallelExecutor::new(test_job_scheduler()).with_parallel_enabled(false);

    let report = executor
        .run_batches_with_report(&batches, &registry)
        .unwrap();

    assert!(!executor.parallel_enabled());
    assert_eq!(report.parallel_batches(), 0);
    assert_eq!(report.serial_batches(), 2);
    assert_eq!(report.serial_fallbacks(), 1);
    assert_eq!(report.executed_systems(), 3);
    assert_eq!(
        *observed.lock().unwrap(),
        vec!["read.health", "read.counter", "write.health"]
    );
}

#[test]
fn schedule_parallel_execution_report_records_diagnostic_counts() {
    let runtime = CoreRuntime::new();
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("a", SystemStage::Update, SystemParamAccess::default()),
        ScheduleConflictNode::new("b", SystemStage::Update, SystemParamAccess::default()),
    ]);
    let batches = graph.conservative_parallel_batches();
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    registry.register("a", || Ok(()));
    registry.register("b", || Ok(()));
    let executor = ScheduleParallelExecutor::new(test_job_scheduler()).with_parallel_enabled(false);
    let report = executor
        .run_batches_with_report(&batches, &registry)
        .unwrap();

    report.record_diagnostics(&runtime.handle(), 42);

    let snapshot = runtime.diagnostic_store_snapshot();
    assert_eq!(
        diagnostic_current_value(&snapshot, SCHEDULE_PARALLEL_BATCHES_DIAGNOSTIC),
        Some(0.0)
    );
    assert_eq!(
        diagnostic_current_value(&snapshot, SCHEDULE_SERIAL_FALLBACKS_DIAGNOSTIC),
        Some(1.0)
    );
}

#[test]
fn representative_schedule_produces_multi_system_parallel_batches() {
    let batches = representative_parallel_batches();

    assert_eq!(batches.len(), 3);
    assert!(batches.iter().all(|batch| batch.system_ids().len() == 2));
    assert_eq!(
        batches
            .iter()
            .map(|batch| {
                batch
                    .system_ids()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        vec![
            vec!["read.health", "read.counter"],
            vec!["write.health", "write.counter"],
            vec!["post.extract.a", "post.extract.b"]
        ]
    );
}

#[test]
fn parallel_and_serial_execution_reach_identical_world_state() {
    let batches = representative_parallel_batches();
    let parallel_state = Arc::new(Mutex::new(RepresentativeScheduleWorld::default()));
    let serial_state = Arc::new(Mutex::new(RepresentativeScheduleWorld::default()));
    let parallel_registry = representative_schedule_registry(parallel_state.clone());
    let serial_registry = representative_schedule_registry(serial_state.clone());
    let parallel_executor = ScheduleParallelExecutor::new(test_job_scheduler());
    let serial_executor =
        ScheduleParallelExecutor::new(test_job_scheduler()).with_parallel_enabled(false);

    let parallel_report = parallel_executor
        .run_batches_with_report(&batches, &parallel_registry)
        .unwrap();
    let serial_report = serial_executor
        .run_batches_with_report(&batches, &serial_registry)
        .unwrap();

    assert_eq!(
        *parallel_state.lock().unwrap(),
        *serial_state.lock().unwrap()
    );
    assert_eq!(parallel_report.parallel_batches(), 3);
    assert_eq!(parallel_report.serial_batches(), 0);
    assert_eq!(parallel_report.serial_fallbacks(), 0);
    assert_eq!(parallel_report.executed_systems(), 6);
    assert_eq!(serial_report.parallel_batches(), 0);
    assert_eq!(serial_report.serial_batches(), 3);
    assert_eq!(serial_report.serial_fallbacks(), 3);
    assert_eq!(serial_report.executed_systems(), 6);
}

#[test]
fn executor_batches_are_chained_through_job_dependencies() {
    let batches = representative_parallel_batches();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    for system_id in [
        "read.health",
        "read.counter",
        "write.health",
        "write.counter",
        "post.extract.a",
        "post.extract.b",
    ] {
        let observed = observed.clone();
        registry.register(system_id, move || {
            let mut observed = observed.lock().unwrap();
            if system_id.starts_with("write.") {
                let read_batch_complete =
                    observed.contains(&"read.health") && observed.contains(&"read.counter");
                if !read_batch_complete {
                    return Err("write.started.before.read.batch.completed");
                }
            }
            if system_id.starts_with("post.") {
                let write_batch_complete =
                    observed.contains(&"write.health") && observed.contains(&"write.counter");
                if !write_batch_complete {
                    return Err("post.started.before.write.batch.completed");
                }
            }
            observed.push(system_id);
            Ok(())
        });
    }
    let executor = ScheduleParallelExecutor::new(test_job_scheduler().with_diagnostics());

    let report = executor
        .run_batches_with_report(&batches, &registry)
        .unwrap();

    let scheduler_report = executor.scheduler().diagnostic_report();
    assert_eq!(report.parallel_batches(), 3);
    assert_eq!(scheduler_report.scheduled, batches.len() as u64);
    assert_eq!(scheduler_report.completed, batches.len() as u64);
    assert_eq!(observed.lock().unwrap().len(), 6);
}

#[test]
fn schedule_parallel_executor_reports_missing_tasks_before_running_batch() {
    let graph = ScheduleConflictGraph::from_nodes([ScheduleConflictNode::new(
        "missing.task",
        SystemStage::Update,
        SystemParamAccess::default(),
    )]);
    let batches = graph.conservative_parallel_batches();
    let registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    let executor = ScheduleParallelExecutor::new(test_job_scheduler());

    let error = executor.run_batches(&batches, &registry).unwrap_err();

    assert_eq!(
        error,
        ScheduleParallelExecutorError::MissingTask {
            system_id: "missing.task".to_string(),
        }
    );
}

fn diagnostic_current_value(
    snapshot: &crate::core::diagnostics::DiagnosticStoreSnapshot,
    path: &str,
) -> Option<f64> {
    snapshot
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .and_then(|series| series.current)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct RepresentativeScheduleWorld {
    health_reads: u32,
    counter_reads: u32,
    health_writes: u32,
    counter_writes: u32,
    post_extracts: u32,
}

fn representative_parallel_batches() -> Vec<ScheduleParallelBatch> {
    let mut world = World::empty();
    world.spawn((ScheduleHealth(1), SchedulePlayer)).unwrap();
    world.insert_resource(ScheduleFrameCounter(0));
    let read_health = SystemState::<QueryState<&'static ScheduleHealth>>::new(&mut world).unwrap();
    let read_counter = SystemState::<ResParam<ScheduleFrameCounter>>::new(&mut world).unwrap();
    let write_health =
        SystemState::<QueryState<&'static mut ScheduleHealth>>::new(&mut world).unwrap();
    let write_counter = SystemState::<ResMutParam<ScheduleFrameCounter>>::new(&mut world).unwrap();

    ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new(
            "read.health",
            SystemStage::Update,
            read_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "read.counter",
            SystemStage::Update,
            read_counter.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.health",
            SystemStage::Update,
            write_health.access().clone(),
        ),
        ScheduleConflictNode::new(
            "write.counter",
            SystemStage::Update,
            write_counter.access().clone(),
        ),
        ScheduleConflictNode::new(
            "post.extract.a",
            SystemStage::PostUpdate,
            SystemParamAccess::default(),
        ),
        ScheduleConflictNode::new(
            "post.extract.b",
            SystemStage::PostUpdate,
            SystemParamAccess::default(),
        ),
    ])
    .conservative_parallel_batches()
}

fn representative_schedule_registry(
    state: Arc<Mutex<RepresentativeScheduleWorld>>,
) -> ScheduleParallelTaskRegistry<&'static str> {
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    register_representative_task(&mut registry, state.clone(), "read.health", |world| {
        world.health_reads += 1;
    });
    register_representative_task(&mut registry, state.clone(), "read.counter", |world| {
        world.counter_reads += 1;
    });
    register_representative_task(&mut registry, state.clone(), "write.health", |world| {
        world.health_writes += 1;
    });
    register_representative_task(&mut registry, state.clone(), "write.counter", |world| {
        world.counter_writes += 1;
    });
    register_representative_task(&mut registry, state.clone(), "post.extract.a", |world| {
        world.post_extracts += 1;
    });
    register_representative_task(&mut registry, state, "post.extract.b", |world| {
        world.post_extracts += 1;
    });
    registry
}

fn register_representative_task(
    registry: &mut ScheduleParallelTaskRegistry<&'static str>,
    state: Arc<Mutex<RepresentativeScheduleWorld>>,
    system_id: &'static str,
    task: impl Fn(&mut RepresentativeScheduleWorld) + Send + Sync + 'static,
) {
    registry.register(system_id, move || {
        task(&mut state.lock().unwrap());
        Ok(())
    });
}

#[test]
fn schedule_parallel_executor_reports_task_failure_by_batch_order() {
    let graph = ScheduleConflictGraph::from_nodes([
        ScheduleConflictNode::new("ok.task", SystemStage::Update, SystemParamAccess::default()),
        ScheduleConflictNode::new(
            "fail.task",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
        ScheduleConflictNode::new(
            "also.ok.task",
            SystemStage::Update,
            SystemParamAccess::default(),
        ),
    ]);
    let batches = graph.conservative_parallel_batches();
    assert_eq!(batches.len(), 1);
    assert_eq!(
        batches[0]
            .system_ids()
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["ok.task", "fail.task", "also.ok.task"]
    );
    let mut registry = ScheduleParallelTaskRegistry::<&'static str>::new();
    registry.register("ok.task", || Ok(()));
    registry.register("fail.task", || Err("boom"));
    registry.register("also.ok.task", || Ok(()));
    let executor = ScheduleParallelExecutor::new(test_job_scheduler());

    let error = executor.run_batches(&batches, &registry).unwrap_err();

    assert_eq!(
        error,
        ScheduleParallelExecutorError::TaskFailed {
            system_id: "fail.task".to_string(),
            error: "boom",
        }
    );
}
