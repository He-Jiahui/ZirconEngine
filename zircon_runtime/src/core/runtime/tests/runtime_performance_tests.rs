use super::{
    CoreRuntime, EngineTaskGraphOptions, TaskGraphAdmissionError, TaskGraphScopeDescriptor,
};
use std::time::Duration;

#[test]
fn runtime_facade_reuses_its_owned_handle() {
    let source = include_str!("../runtime.rs");
    let end = source
        .find("mod performance_tests {")
        .expect("performance test module");
    let implementation = &source[..end];

    assert!(implementation.contains("handle: CoreHandle,"));
    assert!(implementation.contains("self.handle.clone()"));
    assert!(!implementation.contains("self.handle()"));
    assert!(implementation.contains("try_with_task_graph_options"));
    assert!(!implementation.contains("TaskPools::default()"));
    assert!(!implementation.contains("task_pools()"));
}

#[test]
fn core_runtime_routes_scope_shutdown_through_its_execution_owner() {
    let runtime =
        CoreRuntime::try_with_task_graph_options(EngineTaskGraphOptions::with_worker_threads(3))
            .expect("task graph owner should initialize");
    let scope = runtime
        .create_task_graph_scope(TaskGraphScopeDescriptor::new("runtime-test"))
        .expect("running core runtime should create a scope");

    let inventory = runtime.task_graph_worker_inventory();
    assert_eq!(inventory.worker_set_count(), 3);
    assert_eq!(inventory.worker_count(), 3);

    let report = runtime
        .shutdown_task_graph(Duration::from_secs(2))
        .expect("idle worker domains should terminate and join within the deadline");

    assert_eq!(report.scopes.len(), 1);
    assert_eq!(report.worker_shutdowns.len(), 3);
    assert!(report
        .worker_shutdowns
        .iter()
        .all(|domain| domain.all_joined()));
    assert!(matches!(
        runtime.create_task_graph_scope(TaskGraphScopeDescriptor::new("late")),
        Err(TaskGraphAdmissionError::RuntimeStopped)
    ));
    drop(scope);
}
