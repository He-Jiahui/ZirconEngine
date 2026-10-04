use std::sync::mpsc;
use std::{thread, time::Duration};

use crate::core::runtime::tasks::{
    EngineTaskGraph, EngineTaskGraphOptions, TaskGraphScopeDescriptor, TaskPools,
};
use crate::core::TaskState;

use super::*;

fn test_job_scheduler() -> JobScheduler {
    JobScheduler::from_pool(TaskPools::process_default().compute().clone())
}

#[test]
fn dynamic_scene_asset_reload_cancellation_prevents_running_loader_publication() {
    let scheduler = test_job_scheduler();
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let task = DynamicSceneSpawnTask::schedule_with_loader(
        &scheduler,
        "cancelled-running-loader",
        usize::MAX,
        move || {
            started_tx
                .send(())
                .expect("test should observe loader start");
            release_rx.recv().expect("test should release loader");
            Ok(DynamicScene::empty())
        },
    )
    .expect("running scheduler should admit loader");

    started_rx.recv().expect("loader should start");
    assert!(task.request_cancel());
    assert_eq!(task.status_snapshot().state, TaskState::Running);
    assert!(!task.is_ready());
    release_tx.send(()).expect("loader should resume");
    task.wait();

    assert_eq!(task.status_snapshot().state, TaskState::Cancelled);
    assert!(matches!(
        task.take_ready(),
        Some(Err(DynamicSceneError::SpawnTaskCancelled { .. }))
    ));
}

#[test]
fn dynamic_scene_prepare_worker_delay_matrix_completes_without_duplicate_publication() {
    for delay_ms in [0u64, 10, 1_000] {
        let scheduler = test_job_scheduler();
        let task = DynamicSceneSpawnTask::schedule_with_loader(
            &scheduler,
            format!("prepare-delay-{delay_ms}ms"),
            usize::MAX,
            move || {
                thread::sleep(Duration::from_millis(delay_ms));
                Ok(DynamicScene::empty())
            },
        )
        .expect("running scheduler should admit loader");
        task.wait();

        assert_eq!(task.status_snapshot().state, TaskState::Completed);
        assert!(task.take_ready().is_some_and(|result| result.is_ok()));
        assert!(task.take_ready().is_some_and(|result| matches!(
            result,
            Err(DynamicSceneError::SpawnTaskResultUnavailable { .. })
        )));
    }
}

#[test]
fn scoped_dynamic_scene_prepare_cancels_before_a_queued_loader_starts() {
    let runtime = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(3))
        .expect("explicit runtime should create its worker budget");
    let scope = runtime
        .create_scope(TaskGraphScopeDescriptor::new("dynamic-scene"))
        .expect("running runtime should admit its scene scope");
    let scheduler = JobScheduler::from_pool(runtime.worker_pool().clone());
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    scheduler.schedule(move || {
        started_tx.send(()).expect("worker blocker should start");
        release_rx.recv().expect("worker blocker should release");
    });
    started_rx
        .recv()
        .expect("compute worker should be occupied");

    let (loader_tx, loader_rx) = mpsc::sync_channel(1);
    let task = DynamicSceneSpawnTask::schedule_with_loader_in_scope(
        &scheduler,
        &scope,
        "scope-cancelled-loader",
        usize::MAX,
        move || {
            loader_tx.send(()).expect("cancelled loader must not run");
            Ok(DynamicScene::empty())
        },
    )
    .expect("scope should admit the queued scene loader");

    scope.close_admission();
    release_tx.send(()).expect("worker blocker should release");
    task.wait();
    runtime
        .shutdown(Duration::from_secs(1))
        .expect("scoped task should drain before runtime shutdown");

    assert_eq!(task.status_snapshot().state, TaskState::Cancelled);
    assert!(loader_rx.try_recv().is_err());
    assert!(matches!(
        task.take_ready(),
        Some(Err(DynamicSceneError::SpawnTaskCancelled { .. }))
    ));
}
