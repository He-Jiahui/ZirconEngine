use super::*;
use crate::core::{
    EngineTaskGraph, EngineTaskGraphOptions, TaskGraphScopeDescriptor, TaskPoolKind,
};
use std::time::Duration;

#[test]
fn scene_loader_ids_and_domains_remain_distinct_in_one_foreign_scope() {
    let original =
        EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
    let target = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
    let scope = target
        .create_scope(TaskGraphScopeDescriptor::new("scene-id-domain"))
        .unwrap();
    let io = original.scheduler(TaskPoolKind::Io);
    let compute = original.scheduler(TaskPoolKind::Compute);
    let io_task = DynamicSceneSpawnTask::schedule_with_loader_in_scope(
        &io,
        &scope,
        "scene-io",
        usize::MAX,
        || Ok(DynamicScene::empty()),
    )
    .unwrap();
    let compute_task = DynamicSceneSpawnTask::schedule_with_loader_in_scope(
        &compute,
        &scope,
        "scene-compute",
        usize::MAX,
        || Ok(DynamicScene::empty()),
    )
    .unwrap();
    assert_ne!(io_task.descriptor().id, compute_task.descriptor().id);
    assert_eq!(io_task.descriptor().kind, TaskPoolKind::Io);
    assert_eq!(compute_task.descriptor().kind, TaskPoolKind::Compute);
    io_task.wait();
    compute_task.wait();
    assert!(io_task.take_ready().unwrap().is_ok());
    assert!(compute_task.take_ready().unwrap().is_ok());
    assert!(scope.wait_until_quiescent(Duration::from_secs(1)));
    assert_eq!(scope.census().submitted, 2);
    original.shutdown(Duration::from_secs(1)).unwrap();
    target.shutdown(Duration::from_secs(1)).unwrap();
}

#[test]
fn detached_scene_loader_reports_original_runtime_refusal() {
    let original =
        EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3)).unwrap();
    let scheduler = original.scheduler(TaskPoolKind::Io);
    original.shutdown(Duration::from_secs(1)).unwrap();
    assert!(matches!(
        DynamicSceneSpawnTask::schedule_scene(&scheduler, DynamicScene::empty(), "stopped-scene"),
        Err(TaskGraphAdmissionError::RuntimeStopped)
    ));
}
