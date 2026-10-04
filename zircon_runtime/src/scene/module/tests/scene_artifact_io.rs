use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Sender};

use crate::core::framework::scene::{SceneArtifactTerminal, SceneArtifactWaitResult};
use crate::core::runtime::{TaskPool, TaskPoolDescriptor};

use super::{SceneArtifactIo, MAX_PENDING_SCENE_ARTIFACTS};

#[test]
fn scene_artifact_io_keeps_only_the_latest_queued_generation_for_a_scene() {
    let (pool, release) = blocked_io_pool();
    let io = SceneArtifactIo::new(pool);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let first_calls = Arc::clone(&calls);
    let first = io
        .submit(
            "project://fixture/main.scene.toml".to_string(),
            Box::new(move || {
                first_calls.lock().unwrap().push(1);
                Ok(())
            }),
        )
        .unwrap();
    let second_calls = Arc::clone(&calls);
    let second = io
        .submit(
            "project://fixture/main.scene.toml".to_string(),
            Box::new(move || {
                second_calls.lock().unwrap().push(2);
                Ok(())
            }),
        )
        .unwrap();

    assert_eq!(
        first.terminal(),
        Some(SceneArtifactTerminal::Superseded {
            successor: second.generation()
        })
    );
    release.send(()).unwrap();
    assert_eq!(
        second.wait_until(Instant::now() + Duration::from_secs(10)),
        SceneArtifactWaitResult::Terminal(SceneArtifactTerminal::Succeeded)
    );
    assert_eq!(*calls.lock().unwrap(), vec![2]);
}

#[test]
fn scene_artifact_io_rejects_work_after_the_bounded_queue_is_full() {
    let (pool, release) = blocked_io_pool();
    let io = SceneArtifactIo::new(pool);
    let tickets = (0..MAX_PENDING_SCENE_ARTIFACTS)
        .map(|index| {
            io.submit(
                format!("project://fixture/{index}.scene.toml"),
                Box::new(|| Ok(())),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    let error = io
        .submit(
            "project://fixture/overflow.scene.toml".to_string(),
            Box::new(|| Ok(())),
        )
        .unwrap_err();

    assert!(error.to_string().contains("EntryCapacityExceeded"));
    release.send(()).unwrap();
    for ticket in tickets {
        assert_eq!(
            ticket.wait_until(Instant::now() + Duration::from_secs(10)),
            SceneArtifactWaitResult::Terminal(SceneArtifactTerminal::Succeeded)
        );
    }
}

fn blocked_io_pool() -> (TaskPool, Sender<()>) {
    let pool = TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1));
    let (started_tx, started_rx) = bounded(1);
    let (release_tx, release_rx) = bounded(1);
    pool.spawn(move || {
        started_tx.send(()).unwrap();
        release_rx.recv().unwrap();
    });
    started_rx.recv().unwrap();
    (pool, release_tx)
}
