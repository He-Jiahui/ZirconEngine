use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, TryLockError, mpsc};
use std::thread;
use std::time::Duration;

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

    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.project_generation_gate.write().unwrap();
            panic!("poison project generation gate");
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.project.write().unwrap();
            panic!("poison project lock");
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.project_source_paths.write().unwrap();
            panic!("poison project source paths lock");
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.asset_importers.write().unwrap();
            panic!("poison importer registry lock");
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.change_subscribers.lock().unwrap();
            panic!("poison change subscribers lock");
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.watch_error_subscribers.lock().unwrap();
            panic!("poison watch error subscribers lock");
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.watcher_activation.lock().unwrap();
            panic!("poison watcher activation lock");
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.watch_refresh_gate.lock().unwrap();
            panic!("poison watch refresh gate");
        }))
        .is_err()
    );
    assert!(
        catch_unwind(AssertUnwindSafe(|| {
            let _guard = manager.watchers.lock().unwrap();
            panic!("poison watchers lock");
        }))
        .is_err()
    );

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
        2,
        "targeted import and full reimport must share the fenced owner"
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
