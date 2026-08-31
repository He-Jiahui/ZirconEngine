use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, TryLockError};
use std::thread;
use std::time::Duration;

use crate::asset::project::{ProjectManager, ProjectManifest};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::watch::AssetChangeKind;
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator};

use super::*;

fn watcher_activation(lifecycle: ProjectWatcherLifecycle) -> ProjectWatcherActivation {
    ProjectWatcherActivation {
        state: std::sync::Mutex::new(ProjectWatcherActivationState {
            lifecycle,
            changes: Vec::new(),
            coalescible_change_indices: Default::default(),
            queued_change_bytes: 0,
            requires_reconciliation: false,
            diagnostics: Default::default(),
            errors: Default::default(),
            worker_scheduled: false,
        }),
    }
}

fn watcher_change(path: &str) -> AssetChange {
    AssetChange::new(
        AssetChangeKind::Modified,
        AssetUri::parse(path).unwrap(),
        None,
    )
}

#[test]
fn file_commit_failure_cancels_preflighted_resource_publication() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let locator = ResourceLocator::parse("res://models/preflighted-file-failure.glb").unwrap();
    let record = ResourceRecord::new(
        ResourceId::from_stable_label("preflighted-file-failure"),
        ResourceKind::Model,
        locator,
    );
    let events = resource_manager.subscribe();

    let result: Result<(), _> = manager.commit_resource_batch_after_dependencies(
        ResourceMutationBatch::new().upsert_lazy(record.clone()),
        || {
            assert!(resource_manager.registry().get(record.id).is_none());
            assert!(events.try_recv().is_err());
            Err(asset_error(crate::asset::AssetImportError::Parse(
                "injected staged-file commit failure".to_owned(),
            )))
        },
    );

    assert!(result.is_err());
    assert!(resource_manager.registry().get(record.id).is_none());
    assert!(events.try_recv().is_err());
    resource_manager
        .commit(ResourceMutationBatch::new().upsert_lazy(record.clone()))
        .expect("a cancelled outer transaction releases the Resource commit gate");
    assert_eq!(resource_manager.registry().get(record.id), Some(&record));
}

#[test]
fn dependency_state_is_visible_before_resource_publication() {
    let manager = ProjectAssetManager::default();
    let resource_manager = manager.resource_manager();
    let locator = ResourceLocator::parse("res://models/dependency-before-resource.glb").unwrap();
    let record = ResourceRecord::new(
        ResourceId::from_stable_label("dependency-before-resource"),
        ResourceKind::Model,
        locator,
    );
    let events = resource_manager.subscribe();
    let dependency_committed = Arc::new(AtomicBool::new(false));
    let observed_dependency = Arc::clone(&dependency_committed);
    let observer = thread::spawn(move || {
        events
            .recv_timeout(Duration::from_secs(2))
            .expect("resource publication must reach the observer");
        assert!(
            observed_dependency.load(Ordering::Acquire),
            "resource events must not overtake dependent project state"
        );
    });

    manager
        .commit_resource_batch_after_dependencies(
            ResourceMutationBatch::new().upsert_lazy(record.clone()),
            || {
                assert!(resource_manager.registry().get(record.id).is_none());
                dependency_committed.store(true, Ordering::Release);
                Ok(())
            },
        )
        .unwrap();

    observer.join().unwrap();
    assert_eq!(resource_manager.registry().get(record.id), Some(&record));
}

#[test]
fn watcher_activation_queues_pending_and_draining_events_in_arrival_order() {
    let activation = std::sync::Arc::new(watcher_activation(ProjectWatcherLifecycle::Pending));
    let manager = ProjectAssetManager::default();

    activation.enqueue_batch(
        &manager,
        AssetWatchBatch {
            changes: vec![watcher_change("res://first.json")],
            ..AssetWatchBatch::default()
        },
    );
    activation.begin_draining();
    activation.enqueue_batch(
        &manager,
        AssetWatchBatch {
            changes: vec![watcher_change("res://second.json")],
            ..AssetWatchBatch::default()
        },
    );

    let state = activation.lock_state();
    assert_eq!(state.lifecycle, ProjectWatcherLifecycle::Draining);
    assert_eq!(
        state
            .changes
            .iter()
            .map(|change| change.uri.to_string())
            .collect::<Vec<_>>(),
        vec!["res://first.json", "res://second.json"]
    );
}

#[test]
fn watcher_activation_rechecks_retirement_after_initial_active_admission() {
    let activation = watcher_activation(ProjectWatcherLifecycle::Active);

    activation.retire();

    assert!(!activation.is_active());
}

#[test]
fn project_asset_manager_runtime_accessors_recover_poisoned_locks() {
    let manager = ProjectAssetManager::default();

    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.project_generation_gate.write().unwrap();
        panic!("poison project generation gate");
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.project.write().unwrap();
        panic!("poison project lock");
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.project_source_paths.write().unwrap();
        panic!("poison project source paths lock");
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.asset_importers.write().unwrap();
        panic!("poison importer registry lock");
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.change_subscribers.lock().unwrap();
        panic!("poison change subscribers lock");
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.watch_error_subscribers.lock().unwrap();
        panic!("poison watch error subscribers lock");
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.watcher_activation.lock().unwrap();
        panic!("poison watcher activation lock");
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.watch_refresh_gate.lock().unwrap();
        panic!("poison watch refresh gate");
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.watchers.lock().unwrap();
        panic!("poison watchers lock");
    }))
    .is_err());

    drop(manager.project_generation_read());
    assert!(manager.project_read().is_none());
    assert!(manager.project_source_paths_read().is_empty());
    assert!(manager.importer_registry_read().importers().is_empty());
    assert!(manager.lock_change_subscribers().is_empty());
    assert!(manager.lock_watch_error_subscribers().is_empty());
    assert!(manager.lock_watcher_activation().is_none());
    drop(manager.lock_watch_refresh());
    assert!(manager.lock_watchers().is_empty());
}

#[test]
fn only_the_latest_project_preparation_epoch_can_publish() {
    let manager = ProjectAssetManager::default();

    let older = manager.begin_project_preparation();
    let newer = manager.begin_project_preparation();

    assert!(!manager.is_latest_project_preparation(older));
    assert!(manager.is_latest_project_preparation(newer));
}

fn install_generation_fixture(manager: &ProjectAssetManager, case: &str) -> std::path::PathBuf {
    let root = unique_temp_project_root(case);
    fs::create_dir_all(root.join("assets")).unwrap();
    ProjectManifest::new(
        "Generation Commit Fixture",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();
    let project = ProjectManager::open(&root).unwrap();
    manager.begin_project_preparation();
    let generation = manager.project_generation_write();
    *manager.project_write() = Some(project);
    drop(generation);
    root
}

#[test]
fn project_generation_conditional_commit_holds_the_read_fence_through_the_callback() {
    let manager = Arc::new(ProjectAssetManager::default());
    let root = install_generation_fixture(&manager, "conditional_commit_fence");
    let snapshot = manager.current_project_generation_snapshot().unwrap();
    let (_, token) = snapshot.into_parts();
    let (entered_sender, entered_receiver) = mpsc::sync_channel(0);
    let (release_sender, release_receiver) = mpsc::sync_channel(0);
    let commit_manager = Arc::clone(&manager);
    let commit = thread::spawn(move || {
        commit_manager.commit_if_project_generation(&token, || {
            entered_sender.send(()).unwrap();
            release_receiver.recv().unwrap();
            41
        })
    });

    entered_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("conditional commit callback must begin");
    assert!(matches!(
        manager.project_generation_gate.try_write(),
        Err(TryLockError::WouldBlock)
    ));
    release_sender.send(()).unwrap();
    assert_eq!(
        commit.join().unwrap(),
        ProjectGenerationCommitOutcome::Committed(41)
    );

    drop(manager);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn superseded_project_generation_never_invokes_the_commit_callback() {
    let manager = ProjectAssetManager::default();
    let root = install_generation_fixture(&manager, "conditional_commit_superseded");
    let snapshot = manager.current_project_generation_snapshot().unwrap();
    let (_, token) = snapshot.into_parts();
    let newer_project = ProjectManager::open(&root).unwrap();
    manager.begin_project_preparation();
    let generation = manager.project_generation_write();
    *manager.project_write() = Some(newer_project);
    drop(generation);
    let invoked = AtomicBool::new(false);

    let outcome = manager.commit_if_project_generation(&token, || {
        invoked.store(true, Ordering::Release);
    });

    assert_eq!(
        outcome,
        ProjectGenerationCommitOutcome::Superseded {
            newer_same_project_generation: true,
        }
    );
    assert!(!invoked.load(Ordering::Acquire));

    drop(manager);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn unpublished_preparation_does_not_invalidate_the_active_generation() {
    let manager = ProjectAssetManager::default();
    let root = install_generation_fixture(&manager, "conditional_commit_pending_preparation");
    let snapshot = manager.current_project_generation_snapshot().unwrap();
    let (_, token) = snapshot.into_parts();
    manager.begin_project_preparation();

    assert_eq!(
        manager.commit_if_project_generation(&token, || 17),
        ProjectGenerationCommitOutcome::Committed(17)
    );

    drop(manager);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_generation_precheck_distinguishes_current_newer_and_closed_projects() {
    let manager = ProjectAssetManager::default();
    let root = install_generation_fixture(&manager, "generation_precheck");
    let snapshot = manager.current_project_generation_snapshot().unwrap();
    let (_, token) = snapshot.into_parts();

    assert_eq!(
        manager.check_project_generation(&token),
        ProjectGenerationMatch::Current
    );

    let newer_project = ProjectManager::open(&root).unwrap();
    let generation = manager.project_generation_write();
    *manager.project_write() = Some(newer_project);
    drop(generation);
    assert_eq!(
        manager.check_project_generation(&token),
        ProjectGenerationMatch::Superseded {
            newer_same_project_generation: true,
        }
    );

    let generation = manager.project_generation_write();
    *manager.project_write() = None;
    drop(generation);
    assert_eq!(
        manager.check_project_generation(&token),
        ProjectGenerationMatch::Superseded {
            newer_same_project_generation: false,
        }
    );

    drop(manager);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_generation_publication_holds_the_write_fence_through_broadcast() {
    let manager = Arc::new(ProjectAssetManager::default());
    let subscribers = manager.lock_change_subscribers();
    let (generation_acquired, generation_ready) = mpsc::sync_channel(0);
    let publishing_manager = Arc::clone(&manager);
    let publication = thread::spawn(move || {
        let generation = publishing_manager.project_generation_write();
        generation_acquired.send(()).unwrap();
        publishing_manager.publish_project_generation(
            generation,
            vec![watcher_change("res://generation-fence.json")],
        );
    });

    generation_ready
        .recv_timeout(Duration::from_secs(2))
        .expect("publication worker must acquire the generation fence");
    assert!(matches!(
        manager.project_generation_gate.try_read(),
        Err(TryLockError::WouldBlock)
    ));
    assert!(matches!(
        manager.project_generation_gate.try_write(),
        Err(TryLockError::WouldBlock)
    ));

    drop(subscribers);
    publication.join().unwrap();
}

#[test]
fn generation_publication_callers_share_the_fenced_runtime_owner() {
    let runtime = include_str!("../runtime.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("runtime production source must precede its test module");
    let close = include_str!("../close_project.rs");
    let open = include_str!("../open_project.rs");
    let contract = include_str!("../../service_contracts/asset_manager_contract.rs");

    assert_eq!(
        open.matches("self.publish_project_generation(").count(),
        1,
        "open must publish through the generation-fenced owner"
    );
    assert_eq!(
        close.matches("self.publish_project_generation(").count(),
        1,
        "close must publish through the generation-fenced owner"
    );
    assert_eq!(
        contract.matches("self.publish_project_generation(").count(),
        3,
        "model import, targeted import, and full reimport must share the fenced owner"
    );
    assert_eq!(
        runtime.matches("self.publish_project_generation(").count(),
        1,
        "watcher commit must publish through the same fenced owner"
    );
    assert!(!open.contains("drop(generation);"));
    assert!(!contract.contains("drop(_generation);"));
    assert!(!close.contains("drop(generation);"));
}

#[test]
fn project_generation_publication_installs_asset_management_before_observers_run() {
    let runtime = include_str!("../runtime.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("runtime production source must precede its test module");
    let publish = runtime
        .split("fn publish_project_generation")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(in crate::asset::pipeline::manager) fn broadcast_watch_error")
                .next()
        })
        .expect("read project generation publication owner");

    let install = publish
        .find("self.refresh_asset_management_generation();")
        .expect("asset management generation refresh");
    let broadcast = publish
        .find("self.broadcast(changes);")
        .expect("change broadcast");
    let wake = publish
        .find("self.publish_generation_wake();")
        .expect("generation wake");

    assert!(install < broadcast);
    assert!(broadcast < wake);
}
