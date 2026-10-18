use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, TryLockError};
use std::thread;
use std::time::{Duration, Instant};

use crate::asset::project::{ProjectManager, ProjectManifest};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::watch::AssetChangeKind;
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator, ResourceRecord};

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
            ingress_epoch: Arc::new(()),
            retry_timer: None,
            retry_work: None,
            retry_ready: false,
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
        let _guard = manager.change_events.lock_for_test();
        panic!("poison change subscribers lock");
    }))
    .is_err());
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.watch_errors.lock_for_test();
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
    assert!(manager.change_events.subscriber_count_for_test() == 0);
    assert!(manager.watch_errors.subscriber_count_for_test() == 0);
    assert!(manager.lock_watcher_activation().is_none());
    drop(manager.lock_watch_refresh());
    assert!(manager.lock_watchers().is_empty());
}

#[test]
fn astra_life_a4_project_watcher_shutdown_retains_join_for_retry() {
    let manager = ProjectAssetManager::default();
    let (stop_tx, _stop_rx) = crossbeam_channel::bounded(1);
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let worker = thread::spawn(move || {
        started_tx.send(()).unwrap();
        release_rx.recv().unwrap();
    });
    started_rx.recv().unwrap();
    manager.install_project_watcher_for_test(AssetWatcher::from_test_worker(stop_tx, worker));

    assert!(!manager.shutdown_project_watchers_until(Instant::now()));
    assert_eq!(manager.lock_watchers().len(), 1);

    release_tx.send(()).unwrap();
    assert!(manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(1)));
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
    let subscribers = manager.change_events.lock_for_test();
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
    let runtime = include_str!("../../runtime.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("runtime production source must precede its test module");
    let close = include_str!("../../close_project.rs");
    let open = include_str!("../../open_project.rs");
    let contract =
        include_str!("../../../../../manager/service_contracts/asset_manager_contract.rs");

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
    let runtime = include_str!("../../runtime.rs")
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
        .find("self.collect_generation_wakes()")
        .expect("generation wake");

    assert!(install < broadcast);
    assert!(broadcast < wake);
    let unlock = publish
        .find("drop(generation);")
        .expect("release generation fence");
    let invoke = publish.find("wake();").expect("invoke callbacks");
    assert!(wake < unlock && unlock < invoke);
}

// The production dependency/file boundary must reject an obsolete active capture before its callback.
#[test]
fn m6_watch_commit_rejects_stale_active_before_dependencies_and_preserves_failure_state() {
    use crate::core::resource::{
        ModelMarker, ResourceHandle, ResourceSourceMember, ResourceSourceOutcome,
        ResourceSourcePublication, ResourceState,
    };
    #[derive(Debug)]
    struct Payload(&'static str);
    let manager = ProjectAssetManager::default();
    let resources = manager.resource_manager();
    let root = ResourceRecord::new(
        ResourceId::from_stable_label("m6-watch-boundary"),
        ResourceKind::Model,
        ResourceLocator::parse("res://models/m6-watch-boundary.glb").unwrap(),
    )
    .with_state(ResourceState::Ready)
    .with_source_hash("a");
    resources
        .register_ready(root.clone(), Payload("a"))
        .unwrap();
    let mut failed_record = root.clone().with_state(ResourceState::Error);
    failed_record.diagnostics = vec![crate::core::resource::ResourceDiagnostic::error(
        "failed candidate",
    )];
    let capture_failure = || {
        ResourceSourcePublication::capture(
            root.id,
            vec![root.id],
            vec![ResourceSourceMember {
                record: failed_record.clone(),
                payload: None,
            }],
            ResourceSourceOutcome::Failed,
            resources.projection_snapshot().active(),
        )
    };
    let obsolete = capture_failure();
    resources
        .register_ready(root.clone().with_source_hash("b"), Payload("b"))
        .unwrap();
    let before = resources.projection_snapshot();
    let invoked = AtomicBool::new(false);
    let stale = manager.commit_resource_batch_after_dependencies(
        ResourceMutationBatch::new().publish_source(obsolete),
        || {
            invoked.store(true, Ordering::Release);
            Ok(())
        },
    );
    assert!(stale.is_err());
    assert!(!invoked.load(Ordering::Acquire));
    assert_eq!(
        resources.projection_snapshot().active().key(root.id),
        before.active().key(root.id)
    );
    let abandoned: Result<(), _> = manager.commit_resource_batch_after_dependencies(
        ResourceMutationBatch::new().publish_source(capture_failure()),
        || {
            Err(asset_error(crate::asset::AssetImportError::Parse(
                "file commit failed".into(),
            )))
        },
    );
    assert!(abandoned.is_err());
    assert_eq!(
        resources.registry().get(root.id).unwrap().state,
        ResourceState::Ready
    );
    assert_eq!(
        resources.projection_snapshot().active().key(root.id),
        before.active().key(root.id)
    );
    manager
        .commit_resource_batch_after_dependencies(
            ResourceMutationBatch::new().publish_source(capture_failure()),
            || Ok(()),
        )
        .expect("watch's successful file/dependency boundary still publishes an Error candidate");
    assert_eq!(
        resources.registry().get(root.id).unwrap().state,
        ResourceState::Error
    );
    let handle = ResourceHandle::<ModelMarker>::new(root.id);
    assert_eq!(
        resources
            .snapshot_expected::<_, Payload>(handle, &before.active().key(root.id).unwrap(),)
            .unwrap()
            .resource()
            .0,
        "b"
    );
    assert_eq!(resources.ref_count(root.id), Some(0));
}

#[test]
fn m6_gltf_watch_failure_keeps_last_good_and_recovery_replaces_resident_group() {
    use crate::asset::{AssetManager, MeshAsset, ModelAsset};
    use crate::core::resource::{
        MeshMarker, ModelMarker, ResourceHandle, ResourceState, UntypedResourceHandle,
    };
    use crate::graphics::scene::render_scene::RenderSceneResourceReferenceDelta;
    // Use the test facade while renderer resource ownership stays inside graphics.
    use crate::graphics::{
        RenderAssetDemandGeneration, RenderAssetDeviceEpoch, RenderAssetResidencyManager,
        RenderAssetResidencyState, RenderAssetResidencyTransitionError,
    };
    use zr_rhi::{DeviceGeneration, DeviceId};

    let root = unique_temp_project_root("m6_production_gltf_watch");
    fs::create_dir_all(root.join("assets/warm")).unwrap();
    fs::create_dir_all(root.join("assets/cold")).unwrap();
    ProjectManifest::new(
        "M6 Production Watch",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();
    let model_bytes = br#"{"asset":{"version":"2.0"},"buffers":[{"uri":"aaa.bin","byteLength":36}],"bufferViews":[{"buffer":0,"byteLength":36}],"accessors":[{"bufferView":0,"componentType":5126,"count":3,"type":"VEC3","min":[0,0,0],"max":[10,1,0]}],"meshes":[{"primitives":[{"attributes":{"POSITION":0}}]}]}"#;
    let vertices = |x: f32| {
        [x, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>()
    };
    let warm_uri = AssetUri::parse("res://warm/model.gltf").unwrap();
    let cold_uri = AssetUri::parse("res://cold/model.gltf").unwrap();
    for directory in ["warm", "cold"] {
        fs::write(
            root.join("assets").join(directory).join("model.gltf"),
            model_bytes,
        )
        .unwrap();
        fs::write(
            root.join("assets").join(directory).join("aaa.bin"),
            vertices(0.25),
        )
        .unwrap();
    }
    // Persist sidecars before installing PAM, as in the existing production activation fixture.
    let mut project = ProjectManager::open(&root).unwrap();
    project
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    project.scan_and_import().unwrap();
    let initial_warm = project.source_resource_records(&warm_uri);
    let initial_cold = project.source_resource_records(&cold_uri);
    let warm_model = initial_warm
        .iter()
        .find(|record| record.kind == ResourceKind::Model)
        .unwrap()
        .id;
    let warm_mesh = initial_warm
        .iter()
        .find(|record| record.kind == ResourceKind::Mesh)
        .unwrap()
        .id;
    let cold_model = initial_cold
        .iter()
        .find(|record| record.kind == ResourceKind::Model)
        .unwrap()
        .id;
    let manager = ProjectAssetManager::default();
    manager
        .register_first_wave_plugin_fixture_importers_for_test()
        .unwrap();
    AssetManager::open_prepared_project(&manager, project).unwrap();
    // Drive the same production batch method deterministically, without filesystem watcher races.
    assert!(manager.shutdown_project_watchers_until(Instant::now() + Duration::from_secs(2)));
    let resources = manager.resource_manager();
    let model_lease = manager.acquire_model_asset(warm_model).unwrap();
    let mesh_lease = manager.acquire_mesh_asset(warm_mesh).unwrap();
    let before = resources.projection_snapshot();
    let model_key = before.active().key(warm_model).unwrap();
    let mesh_key = before.active().key(warm_mesh).unwrap();
    assert!(initial_cold
        .iter()
        .filter(|record| matches!(record.kind, ResourceKind::Model | ResourceKind::Mesh))
        .all(|record| before.active().row(record.id).is_none()));

    let full_batch = || AssetWatchBatch {
        changes: vec![AssetChange::new(
            AssetChangeKind::Modified,
            cold_uri.clone(),
            None,
        )],
        requires_reconciliation: true,
        ..AssetWatchBatch::default()
    };
    fs::write(root.join("assets/cold/aaa.bin"), vertices(0.5)).unwrap();
    let cold_work = manager
        .prepare_watch_publication(full_batch())
        .expect("real full importer output");
    let cold_source = cold_work
        .sources
        .iter()
        .find(|source| source.owner == cold_model)
        .unwrap();
    assert!(
        cold_source
            .members
            .iter()
            .filter(|member| matches!(member.record.kind, ResourceKind::Model | ResourceKind::Mesh))
            .all(|member| member.payload.is_none()),
        "full preparation must immediately release cold owned geometry"
    );
    assert!(manager.commit_watch_preparation(cold_work, None).is_none());
    assert!(initial_cold
        .iter()
        .filter(|record| matches!(record.kind, ResourceKind::Model | ResourceKind::Mesh))
        .all(|record| resources
            .projection_snapshot()
            .active()
            .row(record.id)
            .is_none()));

    // Reproduce demand arriving after preparation: the missing owned output cannot be retried
    // forever or converted into an import failure. Its durable project generation stays unchanged.
    fs::write(root.join("assets/cold/aaa.bin"), vertices(0.625)).unwrap();
    let late_work = manager.prepare_watch_publication(full_batch()).unwrap();
    let sequence = manager
        .current_project_generation_snapshot()
        .unwrap()
        .project()
        .catalog_input_generation()
        .sequence();
    let late_lease = manager.acquire_model_asset(cold_model).unwrap();
    let retry = manager.commit_watch_preparation(late_work, None);
    let Some(WatchRetryWork::Reprepare(batch)) = retry else {
        panic!("late cold demand must retain a real reprepare responsibility before file commit");
    };
    assert_eq!(
        manager
            .current_project_generation_snapshot()
            .unwrap()
            .project()
            .catalog_input_generation()
            .sequence(),
        sequence
    );
    let recaptured = manager.prepare_watch_publication(batch).unwrap();
    assert!(recaptured
        .sources
        .iter()
        .find(|source| source.owner == cold_model)
        .unwrap()
        .members
        .iter()
        .any(|member| member.record.id == cold_model && member.payload.is_some()));
    assert!(manager.commit_watch_preparation(recaptured, None).is_none());
    assert!(resources
        .projection_snapshot()
        .active()
        .row(cold_model)
        .is_some());
    drop(late_lease);

    let watch_warm = || AssetWatchBatch {
        changes: vec![AssetChange::new(
            AssetChangeKind::Modified,
            warm_uri.clone(),
            None,
        )],
        ..AssetWatchBatch::default()
    };
    fs::remove_file(root.join("assets/warm/aaa.bin")).unwrap();
    manager.process_watch_batch_in_generation(watch_warm());
    let failed_project = manager.current_project_generation_snapshot().unwrap();
    let failed_root = failed_project.project().registry().get(warm_model).unwrap();
    assert_eq!(failed_root.state, ResourceState::Error);
    assert!(failed_root.artifact_locator.is_none());
    assert!(failed_project.project().load_artifact(&warm_uri).is_err());
    let failed = resources.projection_snapshot();
    assert_eq!(failed.active().key(warm_model).as_ref(), Some(&model_key));
    assert_eq!(failed.active().key(warm_mesh).as_ref(), Some(&mesh_key));
    assert!(Arc::ptr_eq(
        resources
            .snapshot_expected::<_, ModelAsset>(
                ResourceHandle::<ModelMarker>::new(warm_model),
                &model_key,
            )
            .unwrap()
            .resource(),
        model_lease.resource()
    ));
    assert!(Arc::ptr_eq(
        resources
            .snapshot_expected::<_, MeshAsset>(
                ResourceHandle::<MeshMarker>::new(warm_mesh),
                &mesh_key,
            )
            .unwrap()
            .resource(),
        mesh_lease.resource()
    ));

    // Both tickets are first issued after the actual failed watch import.
    let device = RenderAssetDeviceEpoch::new(DeviceId::new(19_407), DeviceGeneration::new(1));
    let demand = RenderAssetDemandGeneration::new(1).unwrap();
    let mut residency = RenderAssetResidencyManager::new();
    let model_resource = UntypedResourceHandle::new(warm_model, ResourceKind::Model);
    let mesh_resource = UntypedResourceHandle::new(warm_mesh, ResourceKind::Mesh);
    let mutation = residency
        .apply_scene_reference_deltas(
            &[
                RenderSceneResourceReferenceDelta::acquire(model_resource, 1),
                RenderSceneResourceReferenceDelta::acquire(mesh_resource, 1),
            ],
            &failed,
            device,
            demand,
        )
        .unwrap();
    assert_eq!(mutation.requests().len(), 2);
    let tickets = mutation.requests().to_vec();
    for ticket in &tickets {
        residency
            .advance(ticket, RenderAssetResidencyState::Reading)
            .unwrap();
    }
    manager.process_watch_batch_in_generation(watch_warm());
    assert_eq!(
        resources
            .projection_snapshot()
            .active()
            .key(warm_model)
            .as_ref(),
        Some(&model_key)
    );
    assert_eq!(
        resources
            .projection_snapshot()
            .active()
            .key(warm_mesh)
            .as_ref(),
        Some(&mesh_key)
    );

    fs::write(root.join("assets/warm/aaa.bin"), vertices(0.75)).unwrap();
    manager.process_watch_batch_in_generation(watch_warm());
    let recovered_project = manager.current_project_generation_snapshot().unwrap();
    assert_eq!(
        recovered_project
            .project()
            .registry()
            .get(warm_model)
            .unwrap()
            .state,
        ResourceState::Ready
    );
    assert!(recovered_project.project().load_artifact(&warm_uri).is_ok());
    let recovered = resources.projection_snapshot();
    assert_ne!(
        recovered.active().key(warm_model).as_ref(),
        Some(&model_key)
    );
    assert_ne!(recovered.active().key(warm_mesh).as_ref(), Some(&mesh_key));
    let new_model = resources
        .snapshot_expected::<_, ModelAsset>(
            ResourceHandle::<ModelMarker>::new(warm_model),
            &recovered.active().key(warm_model).unwrap(),
        )
        .unwrap();
    let new_mesh = resources
        .snapshot_expected::<_, MeshAsset>(
            ResourceHandle::<MeshMarker>::new(warm_mesh),
            &recovered.active().key(warm_mesh).unwrap(),
        )
        .unwrap();
    assert!(!model_lease.primitives.is_empty());
    assert!(!new_model.resource().primitives.is_empty());
    assert_eq!(
        mesh_lease.to_model_primitive().unwrap().vertices[0].position[0],
        0.25
    );
    assert_eq!(
        new_mesh.resource().to_model_primitive().unwrap().vertices[0].position[0],
        0.75
    );
    assert!(!Arc::ptr_eq(new_mesh.resource(), mesh_lease.resource()));
    let replacement = residency
        .reconcile_changed_resources(&[model_resource, mesh_resource], &recovered, device, demand)
        .unwrap();
    assert_eq!(replacement.requests().len(), 2);
    for ticket in &tickets {
        assert!(matches!(
            residency.advance(ticket, RenderAssetResidencyState::Decoding),
            Err(RenderAssetResidencyTransitionError::StaleTicket { .. })
        ));
    }
    drop(model_lease);
    drop(mesh_lease);
    assert!(
        resources
            .projection_snapshot()
            .active()
            .row(warm_model)
            .is_some(),
        "old-generation lease release cannot evict the new active payload"
    );
    drop(new_model);
    drop(new_mesh);
    drop(recovered_project);
    drop(failed_project);
    drop(manager);
    drop(resources);
    fs::remove_dir_all(root).unwrap();
}
