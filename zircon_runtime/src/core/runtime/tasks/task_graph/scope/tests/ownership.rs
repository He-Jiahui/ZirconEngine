use super::*;

#[test]
fn canonical_dependency_rejects_prerequisite_from_another_task_graph() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("primary graph should create its worker budget");
    let foreign_runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("foreign graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("primary"))
        .expect("primary graph should admit a scope");
    let foreign_scope = foreign_runtime
        .create_scope(TaskGraphScopeDescriptor::new("foreign"))
        .expect("foreign graph should admit a scope");
    let prerequisite = foreign_scope
        .submit(
            descriptor(10, TaskCancellationPolicy::FinishOnShutdown),
            |_| {},
        )
        .expect("foreign prerequisite should be admitted");
    prerequisite.wait();

    assert!(matches!(
        scope.submit_after(
            &[prerequisite],
            descriptor(11, TaskCancellationPolicy::FinishOnShutdown),
            |_| {},
        ),
        Err(TaskGraphAdmissionError::DependencyOwnerMismatch { .. })
    ));
    let census = scope.census();
    assert_eq!(census.submitted, 0);
    assert_eq!(census.queued, 0);
    assert_eq!(census.running, 0);

    scope.close_admission();
    foreign_scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("primary graph should shut down after rejected admission");
    foreign_runtime
        .shutdown(Duration::from_secs(1))
        .expect("foreign graph should shut down after completed prerequisite");
}

#[test]
fn canonical_dependency_rejects_detached_completed_handle() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("detached-prerequisite"))
        .expect("running graph should admit a scope");
    let prerequisite =
        TaskHandle::completed(descriptor(13, TaskCancellationPolicy::FinishOnShutdown));

    assert!(matches!(
        scope.submit_after(
            &[prerequisite],
            descriptor(14, TaskCancellationPolicy::FinishOnShutdown),
            |_| {},
        ),
        Err(TaskGraphAdmissionError::DependencyOwnerMismatch { .. })
    ));
    let census = scope.census();
    assert_eq!(census.submitted, 0);
    assert_eq!(census.queued, 0);
    assert_eq!(census.running, 0);

    scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("rejected detached dependency should leave the graph idle");
}

#[test]
fn canonical_dependency_rejects_handle_after_source_graph_is_dropped() {
    let prerequisite = {
        let source_runtime =
            EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
                .expect("source graph should create its worker budget");
        let source_scope = source_runtime
            .create_scope(TaskGraphScopeDescriptor::new("dropped-source"))
            .expect("source graph should admit a scope");
        let prerequisite = source_scope
            .submit(
                descriptor(15, TaskCancellationPolicy::FinishOnShutdown),
                |_| {},
            )
            .expect("source graph should admit its prerequisite");
        prerequisite.wait();
        source_scope.close_admission();
        source_runtime
            .shutdown(Duration::from_secs(1))
            .expect("source graph should join before its owner is dropped");
        prerequisite
    };

    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("target graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("target"))
        .expect("target graph should admit a scope");
    assert!(matches!(
        scope.submit_after(
            &[prerequisite],
            descriptor(16, TaskCancellationPolicy::FinishOnShutdown),
            |_| {},
        ),
        Err(TaskGraphAdmissionError::DependencyOwnerMismatch { .. })
    ));
    let census = scope.census();
    assert_eq!(census.submitted, 0);
    assert_eq!(census.queued, 0);
    assert_eq!(census.running, 0);

    scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("target graph should remain idle after stale dependency rejection");
}

#[test]
fn cloning_canonical_handles_does_not_duplicate_graph_owner_reference() {
    const CLONE_COUNT: usize = 64;

    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1))
        .expect("task graph should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("handle-owner-projection"))
        .expect("running graph should admit a scope");
    let task = scope
        .submit(
            descriptor(12, TaskCancellationPolicy::FinishOnShutdown),
            |_| {},
        )
        .expect("scope should admit descriptor-led work");
    let owner_weak_count = task.graph_owner_weak_count();

    let clones = (0..CLONE_COUNT).map(|_| task.clone()).collect::<Vec<_>>();

    assert_eq!(
        task.graph_owner_weak_count(),
        owner_weak_count,
        "public handle projections must not duplicate graph-owner references"
    );
    drop(clones);
    task.wait();
    scope.close_admission();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("completed projection probe should drain");
}
