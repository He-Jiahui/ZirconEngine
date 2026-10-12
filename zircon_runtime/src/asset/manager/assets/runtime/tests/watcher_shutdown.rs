use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use super::{AssetWatcher, ProjectAssetManager};
use crate::asset::project::{ProjectManager, ProjectManifest};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::watch::AssetChangeKind;
use crate::asset::{AssetManager, AssetUri};

const SHUTDOWN_BUDGET: Duration = Duration::from_millis(25);
const SCHEDULING_TOLERANCE: Duration = Duration::from_millis(100);

#[test]
fn astra_life_a4_concurrent_watcher_shutdown_cannot_report_an_in_flight_join_as_complete() {
    let manager = ProjectAssetManager::default();
    let (stop, stopped) = crossbeam_channel::bounded(1);
    let (entered, entered_rx) = mpsc::sync_channel(1);
    let (release, release_rx) = mpsc::sync_channel(1);
    let worker = thread::spawn(move || {
        let _ = stopped.recv_timeout(Duration::from_secs(2));
        let _ = entered.send(());
        let _ = release_rx.recv_timeout(Duration::from_secs(3));
    });
    manager.install_project_watcher_for_test(AssetWatcher::from_test_worker(stop, worker));

    let first_manager = manager.clone();
    let first = thread::spawn(move || {
        first_manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(2))
    });
    let stop_entered = entered_rx.recv_timeout(Duration::from_secs(1));
    let started = Instant::now();
    let second =
        manager.shutdown_project_watchers_until(Instant::now() + Duration::from_millis(25));
    let elapsed = started.elapsed();
    let _ = release.send(());
    let first_completed = first.join().unwrap();
    let completed =
        manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(1));

    assert!(stop_entered.is_ok());
    assert!(!second);
    assert!(elapsed <= SHUTDOWN_BUDGET + SCHEDULING_TOLERANCE);
    assert!(first_completed);
    assert!(completed);
    assert!(manager.lock_watchers().is_empty());
}

#[test]
fn astra_life_a4_watcher_shutdown_returns_pending_when_owner_locks_are_contended() {
    for owner in ["activation", "watchers", "publication"] {
        let manager = ProjectAssetManager::default();
        let (stop, _stopped) = crossbeam_channel::bounded(1);
        let (worker_release, worker_release_rx) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            let _ = worker_release_rx.recv_timeout(Duration::from_secs(2));
        });
        manager.install_project_watcher_for_test(AssetWatcher::from_test_worker(stop, worker));
        let holder_manager = manager.clone();
        let (entered, entered_rx) = mpsc::sync_channel(1);
        let (release, release_rx) = mpsc::sync_channel(1);
        let holder = thread::spawn(move || match owner {
            "activation" => {
                let _guard = holder_manager.lock_watcher_activation();
                let _ = entered.send(());
                let _ = release_rx.recv_timeout(Duration::from_secs(2));
            }
            "watchers" => {
                let _guard = holder_manager.lock_watchers();
                let _ = entered.send(());
                let _ = release_rx.recv_timeout(Duration::from_secs(2));
            }
            _ => {
                let _guard = holder_manager.admit_project_watcher_publication().unwrap();
                let _ = entered.send(());
                let _ = release_rx.recv_timeout(Duration::from_secs(2));
            }
        });
        let lock_entered = entered_rx.recv_timeout(Duration::from_secs(1));
        let started = Instant::now();
        let pending = manager.shutdown_project_watchers_until(started + SHUTDOWN_BUDGET);
        let elapsed = started.elapsed();
        let _ = release.send(());
        holder.join().unwrap();
        let admission_closed = manager.begin_project_watcher_operation().is_err();
        let retained = manager.lock_watchers().len();
        let _ = worker_release.send(());
        let complete =
            manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(1));

        assert!(lock_entered.is_ok());
        assert!(!pending);
        assert!(
            elapsed <= SHUTDOWN_BUDGET + SCHEDULING_TOLERANCE,
            "{owner}: {elapsed:?}"
        );
        assert_eq!(retained, 1, "{owner}");
        assert!(admission_closed, "{owner}");
        assert!(complete);
    }
}

#[test]
fn astra_life_a4_watcher_shutdown_retains_in_flight_transitions_and_closes_admission() {
    let manager = ProjectAssetManager::default();
    let operation = manager.begin_project_watcher_operation().unwrap();

    assert!(!manager.shutdown_project_watchers_until(Instant::now()));
    assert!(manager.begin_project_watcher_operation().is_err());
    assert!(manager.admit_project_watcher_publication().is_err());

    drop(operation);
    assert!(manager.shutdown_project_watchers_until(Instant::now()));
    assert!(manager.begin_project_watcher_operation().is_err());
}

fn project_fixture(label: &str) -> (std::path::PathBuf, ProjectManager) {
    let root = unique_temp_project_root(label);
    std::fs::create_dir_all(root.join("assets")).unwrap();
    ProjectManifest::new(
        "Watcher shutdown",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();
    let project = ProjectManager::open(&root).unwrap();
    (root, project)
}

#[test]
fn astra_life_a4_project_open_cannot_install_watchers_while_shutdown_is_joining() {
    let (root, project) = project_fixture("astra_shutdown_rejects_project_open");
    let manager = ProjectAssetManager::default();
    let (stop, stopped) = crossbeam_channel::bounded(1);
    let (entered, entered_rx) = mpsc::sync_channel(1);
    let (release, release_rx) = mpsc::sync_channel(1);
    let worker = thread::spawn(move || {
        let _ = stopped.recv_timeout(Duration::from_secs(2));
        let _ = entered.send(());
        let _ = release_rx.recv_timeout(Duration::from_secs(2));
    });
    manager.install_project_watcher_for_test(AssetWatcher::from_test_worker(stop, worker));
    let closing_manager = manager.clone();
    let closing = thread::spawn(move || {
        closing_manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(2))
    });
    let entered = entered_rx.recv_timeout(Duration::from_secs(1));
    let started = Instant::now();
    let opened = manager.open_prepared_project(project);
    let elapsed = started.elapsed();
    let _ = release.send(());
    let completed = closing.join().unwrap();
    let has_project = manager.project_read().is_some();
    let has_activation = manager.lock_watcher_activation().is_some();
    let retained = manager.lock_watchers().len();
    let cleanup = manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(1));
    let _ = std::fs::remove_dir_all(root);

    assert!(entered.is_ok());
    assert!(opened.is_err());
    assert!(elapsed <= SCHEDULING_TOLERANCE);
    assert!(completed);
    assert!(cleanup);
    assert!(!has_project);
    assert!(!has_activation);
    assert_eq!(retained, 0);
}

#[test]
fn astra_life_a4_project_open_preparation_remains_owned_until_shutdown_retry() {
    let (root, project) = project_fixture("astra_shutdown_pending_project_open");
    let manager = ProjectAssetManager::default();
    let generation = manager.project_generation_write();
    let epoch = manager.current_project_preparation_epoch();
    let opening_manager = manager.clone();
    let opening = thread::spawn(move || opening_manager.open_prepared_project(project));
    let admission_deadline = Instant::now() + Duration::from_secs(2);
    while manager.current_project_preparation_epoch() == epoch
        && Instant::now() < admission_deadline
    {
        thread::sleep(Duration::from_millis(1));
    }
    let was_admitted = manager.current_project_preparation_epoch() != epoch;
    let started = Instant::now();
    let early_completion = manager.shutdown_project_watchers_until(started + SHUTDOWN_BUDGET);
    let elapsed = started.elapsed();
    drop(generation);
    let opened = opening.join().unwrap();
    let completed =
        manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(1));
    let has_project = manager.project_read().is_some();
    let retained = manager.lock_watchers().len();
    let cleanup = manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(1));
    let _ = std::fs::remove_dir_all(root);

    assert!(was_admitted);
    assert!(!early_completion);
    assert!(elapsed <= SHUTDOWN_BUDGET + SCHEDULING_TOLERANCE);
    assert!(opened.is_err());
    assert!(completed);
    assert!(cleanup);
    assert!(!has_project);
    assert_eq!(retained, 0);
}

#[test]
fn astra_life_a4_project_open_wake_can_request_shutdown_without_waiting_for_itself() {
    assert_publication_wake_shutdown_is_retryable(false);
}

#[test]
fn astra_life_a4_project_close_wake_can_request_shutdown_without_waiting_for_itself() {
    assert_publication_wake_shutdown_is_retryable(true);
}

fn assert_publication_wake_shutdown_is_retryable(close: bool) {
    let (root, project) = project_fixture(if close {
        "astra_close_wake_shutdown"
    } else {
        "astra_open_wake_shutdown"
    });
    std::fs::write(root.join("assets/value.json"), r#"{"value": 1}"#).unwrap();
    let manager = Arc::new(ProjectAssetManager::default());
    let pending_project = if close {
        manager.open_prepared_project(project).unwrap();
        None
    } else {
        Some(project)
    };
    let callback_manager = Arc::downgrade(&manager);
    let (callback_result, callback_results) = mpsc::sync_channel(1);
    let changes = AssetManager::subscribe_asset_changes_with_wake(
        manager.as_ref(),
        Arc::new(move || {
            let manager = callback_manager.upgrade().unwrap();
            let started = Instant::now();
            let complete = manager.shutdown_project_watchers_until(started + SHUTDOWN_BUDGET);
            let _ = callback_result.try_send((complete, started.elapsed()));
        }),
    );
    let operation_manager = Arc::clone(&manager);
    let (operation_result, operation_results) = mpsc::sync_channel(1);
    let operation = thread::spawn(move || {
        let result = match pending_project {
            Some(project) => operation_manager.open_prepared_project(project).map(|_| ()),
            None => operation_manager.close_project().map(|_| ()),
        };
        let _ = operation_result.send(result);
    });

    // Bound the harness wait as well so reentrant waiting fails instead of hanging the suite.
    operation_results
        .recv_timeout(Duration::from_secs(3))
        .expect("project publication did not return from its synchronous wake")
        .unwrap();
    operation.join().unwrap();
    let (early_completion, elapsed) = callback_results
        .recv_timeout(Duration::from_secs(1))
        .expect("project publication did not invoke the subscribed wake");
    let change = changes.recv_timeout(Duration::from_secs(1)).unwrap();
    let completed =
        manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(1));

    assert!(!early_completion);
    assert!(elapsed <= SHUTDOWN_BUDGET + SCHEDULING_TOLERANCE);
    assert_eq!(
        change.kind,
        if close {
            AssetChangeKind::Removed
        } else {
            AssetChangeKind::Added
        }
    );
    assert!(completed);
    assert!(manager.lock_watchers().is_empty());
    drop(changes);
    drop(manager);
    let _ = std::fs::remove_dir_all(root);
}
