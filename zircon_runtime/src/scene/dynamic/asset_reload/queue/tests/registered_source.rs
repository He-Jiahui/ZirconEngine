use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use zircon_runtime_interface::project::RelPath;

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{AssetEvent, AssetManager, AssetUri, Handle, ProjectAssetManager, SceneAsset};
use crate::core::manager::{
    manager_service_handle, resolve_manager_service, ManagerServiceHandle, RegisteredManagerService,
};
use crate::core::resource::ResourceId;
use crate::core::runtime::tasks::{EngineTaskGraphOptions, TaskPoolKind};
use crate::core::runtime::ServiceObject;
use crate::core::{
    CoreError, CoreHandle, CoreRuntime, ManagerDescriptor, ModuleDescriptor, RegistryName,
    ServiceKind, StartupMode,
};

use super::super::reports::DynamicSceneAssetReloadDrainReport;
use super::DynamicSceneAssetReloadQueue;

const MODULE: &str = "RegisteredReloadSourceProbe";
const WAIT: Duration = Duration::from_secs(5);

#[test]
fn registered_reload_source_keeps_its_generation_and_terminal_error_after_reactivation() {
    let (_project_root, runtime, core, handle, project) = registered_source_fixture("generation");
    let scheduler = runtime.task_graph().scheduler(TaskPoolKind::Compute);
    let mut queue =
        DynamicSceneAssetReloadQueue::from_project_asset_manager(project, &core, handle.clone())
            .expect("the real registered provider should construct the queue");

    runtime
        .deactivate_module(MODULE)
        .expect("queue must not retain a call guard");
    let first = queue.drain_events(&scheduler);
    let error = first
        .source_error
        .expect("the retired generation must be rejected");
    assert!(matches!(error, CoreError::StaleServiceHandle { .. }));
    assert_eq!(first.scheduled, 0);
    assert!(!first.receiver_disconnected);
    assert!(!queue.has_pending_work());

    runtime.activate_module(MODULE).unwrap();
    let fresh: ManagerServiceHandle<ProjectAssetManager> =
        manager_service_handle(&core, manager_name().as_str()).unwrap();
    assert_ne!(fresh, handle);
    assert!(resolve_manager_service(&core, fresh).is_ok());
    let second = queue.drain_events(&scheduler);
    assert_eq!(second.source_error, Some(error));
    assert_eq!(second.scheduled, 0);
    assert!(second.superseded_pending.is_empty());
    assert!(!queue.has_pending_work());
    runtime.deactivate_module(MODULE).unwrap();
}

#[test]
fn registered_reload_source_reports_wake_install_failure_after_its_runtime_is_gone() {
    let (_project_root, runtime, core, handle, project) = registered_source_fixture("runtime");
    let scheduler = runtime.task_graph().scheduler(TaskPoolKind::Compute);
    let mut queue =
        DynamicSceneAssetReloadQueue::from_project_asset_manager(project, &core, handle).unwrap();
    queue.install_runtime_frame_wake(Arc::new(|| {})).unwrap();
    drop(core);
    drop(runtime);

    let error = queue
        .install_runtime_frame_wake(Arc::new(|| {}))
        .unwrap_err();
    assert_eq!(error, CoreError::RuntimeUnavailable);
    assert!(queue.runtime_frame_wake_token.is_none());
    assert!(!queue.has_pending_work());
    for _ in 0..2 {
        let report = queue.drain_events(&scheduler);
        assert_eq!(report.source_error, Some(error.clone()));
        assert_eq!(report.scheduled, 0);
        assert!(!report.receiver_disconnected);
        assert!(!queue.has_pending_work());
    }
}

#[test]
fn registered_reload_source_failure_keeps_admitted_work_until_physical_settlement() {
    let (_project_root, runtime, core, handle, project) = registered_source_fixture("settlement");
    let scheduler = runtime.task_graph().scheduler(TaskPoolKind::Compute);
    assert_eq!(
        scheduler.parallelism(),
        1,
        "the real worker budget must serialize the blocker"
    );
    let mut queue =
        DynamicSceneAssetReloadQueue::from_project_asset_manager(project, &core, handle).unwrap();
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    let mut release = ReleaseWorker(Some(release_tx));
    let blocker = scheduler.schedule(move || {
        started_tx.send(()).unwrap();
        release_rx
            .recv_timeout(WAIT)
            .expect("the test must release the real worker");
    });
    started_rx
        .recv_timeout(WAIT)
        .expect("the real compute worker must start");

    let mut admission = DynamicSceneAssetReloadDrainReport::default();
    queue.handle_event(
        &scheduler,
        AssetEvent::Modified {
            handle: Handle::<SceneAsset>::new(ResourceId::from_stable_label(
                "registered-source-work",
            )),
            locator: Some(AssetUri::parse("res://scenes/pending.scene.toml").unwrap()),
            revision: 1,
        },
        &mut admission,
    );
    assert_eq!(admission.scheduled, 1);
    assert!(admission.failed.is_empty());
    assert_eq!(queue.pending_count(), 1);
    assert!(!queue.pending_tasks().next().unwrap().is_ready());

    runtime.deactivate_module(MODULE).unwrap();
    let error = queue
        .install_runtime_frame_wake(Arc::new(|| {}))
        .unwrap_err();
    assert_eq!(queue.pending_count(), 1);
    assert!(queue.has_pending_work());
    let first = queue.drain_events(&scheduler);
    assert_eq!(first.source_error, Some(error.clone()));
    assert_eq!(first.superseded_pending.len(), 1);
    assert_eq!(first.pending_count, 1);
    assert!(queue.has_pending_work());
    assert!(!queue.pending_tasks().next().unwrap().is_ready());

    release.release();
    blocker.wait();
    let deadline = Instant::now() + WAIT;
    while queue.has_pending_work() {
        let report = queue.drain_events(&scheduler);
        assert_eq!(report.source_error, Some(error.clone()));
        assert!(report.superseded_pending.is_empty());
        assert_eq!(report.scheduled, 0);
        assert!(
            Instant::now() < deadline,
            "cancelled work must physically settle"
        );
        std::thread::yield_now();
    }
    assert_eq!(queue.pending_count(), 0);
    queue.assert_no_admitted_reload_storage();
    let terminal = queue.drain_events(&scheduler);
    assert_eq!(terminal.source_error, Some(error));
    assert!(!queue.has_pending_work());
}

fn manager_name() -> RegistryName {
    RegistryName::from_parts(MODULE, ServiceKind::Manager, "ProjectAssetManager")
}

fn registered_source_fixture(
    label: &str,
) -> (
    TemporaryProject,
    CoreRuntime,
    CoreHandle,
    ManagerServiceHandle<ProjectAssetManager>,
    ProjectManager,
) {
    let project_root = TemporaryProject::new(label);
    let runtime =
        CoreRuntime::try_with_task_graph_options(EngineTaskGraphOptions::with_worker_threads(3))
            .unwrap();
    let io = runtime.task_graph().task_pool(TaskPoolKind::Io).clone();
    runtime
        .register_module(
            ModuleDescriptor::new(MODULE, "real registered reload-source ownership probe")
                .with_manager(ManagerDescriptor::new(
                    manager_name(),
                    StartupMode::Immediate,
                    Vec::new(),
                    Arc::new(move |_| {
                        Ok(Arc::new(RegisteredManagerService::new(Arc::new(
                            ProjectAssetManager::new(io.clone()),
                        ))) as ServiceObject)
                    }),
                )),
        )
        .unwrap();
    runtime.activate_module(MODULE).unwrap();
    let core = runtime.handle();
    let handle =
        manager_service_handle::<ProjectAssetManager>(&core, manager_name().as_str()).unwrap();
    let project = {
        let manager =
            resolve_manager_service::<ProjectAssetManager>(&core, handle.clone()).unwrap();
        AssetManager::open_project(&*manager, project_root.root.to_string_lossy().as_ref())
            .unwrap();
        assert!(manager.shutdown_project_watchers_until(Instant::now() + WAIT));
        manager.current_project_manager().unwrap()
    };
    (project_root, runtime, core, handle, project)
}

struct TemporaryProject {
    root: PathBuf,
    intended_parent: PathBuf,
}

impl TemporaryProject {
    fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let executable = std::env::current_exe().unwrap().canonicalize().unwrap();
        let intended_parent = executable
            .parent()
            .unwrap()
            .join("registered-reload-source-fixtures");
        std::fs::create_dir_all(&intended_parent).unwrap();
        assert_eq!(
            intended_parent.canonicalize().unwrap(),
            intended_parent,
            "the fixture parent must remain below the actual executable directory"
        );
        let root = (0..128)
            .find_map(|_| {
                let candidate = intended_parent.join(format!(
                    "{label}-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                match std::fs::create_dir(&candidate) {
                    Ok(()) => Some(candidate),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
                    Err(error) => panic!("exclusive fixture directory creation failed: {error}"),
                }
            })
            .expect("an exclusive fixture leaf must be available without adopting existing paths");
        let resolved_root = root.canonicalize().unwrap();
        assert!(
            resolved_root.is_absolute()
                && resolved_root == root
                && resolved_root.parent() == Some(intended_parent.as_path()),
            "the exclusively created fixture must remain a direct child of its intended parent"
        );
        let fixture = Self {
            root,
            intended_parent,
        };
        let paths = ProjectPaths::from_root(&fixture.root).unwrap();
        paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
        ProjectManifest::new(
            "Registered Reload Source",
            AssetUri::parse("res://scenes/pending.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        fixture
    }
}

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let Ok(resolved_parent) = self.intended_parent.canonicalize() else {
            return;
        };
        let Ok(resolved_root) = self.root.canonicalize() else {
            return;
        };
        if !resolved_root.is_absolute()
            || resolved_parent != self.intended_parent
            || resolved_root != self.root
            || resolved_root.parent() != Some(resolved_parent.as_path())
        {
            return;
        }
        let _ = std::fs::remove_dir_all(resolved_root);
    }
}

struct ReleaseWorker(Option<mpsc::SyncSender<()>>);

impl ReleaseWorker {
    fn release(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

impl Drop for ReleaseWorker {
    fn drop(&mut self) {
        self.release();
    }
}

mod lifecycle_interruption {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;

    use zircon_runtime_interface::world_sync::LifecycleTargetChange;

    use crate::core::framework::scene::WorldHandle;
    use crate::core::resource::{ResourceKind, ResourceRecord};
    use crate::core::JobHandle;
    use crate::scene::components::Name;
    use crate::scene::ecs::{LifecycleEventKind, SystemStage};
    use crate::scene::{
        DynamicSceneAssetReloadLimits, DynamicSceneError, LevelMetadata, LevelSystem, NodeKind,
        World,
    };

    struct Fixture {
        project_root: TemporaryProject,
        runtime: CoreRuntime,
        core: CoreHandle,
        source: ManagerServiceHandle<ProjectAssetManager>,
        level: LevelSystem,
        queue: DynamicSceneAssetReloadQueue,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let (project_root, runtime, core, source, project) = registered_source_fixture(label);
            let limits = DynamicSceneAssetReloadLimits {
                max_apply_per_tick: 2,
                max_ready_per_tick: 3,
                event_time_budget: Duration::MAX,
                ready_time_budget: Duration::MAX,
                apply_time_budget: Duration::MAX,
                ..Default::default()
            };
            let queue = DynamicSceneAssetReloadQueue::from_project_asset_manager_with_limits(
                project,
                &core,
                source.clone(),
                limits,
            )
            .unwrap();
            let level = LevelSystem::new(
                WorldHandle::new(901),
                Arc::new(Mutex::new(World::empty())),
                LevelMetadata::default(),
            );
            Self {
                project_root,
                runtime,
                core,
                source,
                level,
                queue,
            }
        }

        fn publish(&self, label: &str) -> ResourceId {
            let scene: SceneAsset = serde_json::from_value(serde_json::json!({
                "entities": [{
                    "entity": 1, "name": label, "parent": null,
                    "transform": {
                        "translation": [0.0, 0.0, 0.0],
                        "rotation": [0.0, 0.0, 0.0, 1.0],
                        "scale": [1.0, 1.0, 1.0]
                    },
                    "camera": null, "mesh": null, "directional_light": null
                }]
            }))
            .unwrap();
            let paths = ProjectPaths::from_root(&self.project_root.root).unwrap();
            let scenes = paths.asset_root(&RelPath::project_assets()).join("scenes");
            std::fs::create_dir_all(&scenes).unwrap();
            let document = scene
                .to_project_toml_string(|_| {
                    unreachable!("the node fixture has no asset references")
                })
                .unwrap();
            std::fs::write(scenes.join(format!("{label}.scene.toml")), document).unwrap();
            let locator = AssetUri::parse(&format!("res://scenes/{label}.scene.toml")).unwrap();
            let id = ResourceId::from_locator(&locator);
            let manager = resolve_manager_service(&self.core, self.source.clone()).unwrap();
            let published = manager
                .assets::<SceneAsset>()
                .insert(ResourceRecord::new(id, ResourceKind::Scene, locator), scene)
                .unwrap();
            assert_eq!(published.id(), id);
            id
        }

        fn stage_three(&mut self) -> [ResourceId; 3] {
            let scheduler = self.core.scheduler().clone();
            assert_eq!(scheduler.parallelism(), 1);
            let (mut release, blocker) = block_compute(&scheduler);
            let ids = [
                self.publish("lifecycle-first"),
                self.publish("lifecycle-tail"),
                self.publish("lifecycle-ready"),
            ];
            let admission = self.queue.tick_into_level(&scheduler, &self.level);
            assert_eq!(admission.events_drained(), 3);
            assert_eq!(admission.scheduled_count(), 3);
            assert_eq!(self.queue.pending_tasks().len(), 3);
            assert!(self
                .queue
                .pending_tasks()
                .all(|task| !task.task().is_ready()));
            release.release();
            blocker.wait();
            wait_until(|| {
                self.queue
                    .pending_tasks()
                    .all(|task| task.task().is_ready())
            });

            let (mut release, blocker) = block_compute(&scheduler);
            let staged = self.queue.tick_into_level(&scheduler, &self.level);
            assert_eq!(staged.apply.target_stages_scheduled, 2);
            assert_eq!(self.queue.target_staging_count(), 2);
            assert!(self.queue.ready.contains_key(&ids[2]));
            assert_eq!(self.queue.pending_count(), 3);
            release.release();
            blocker.wait();
            wait_until(|| self.queue.target_stages_ready());
            ids
        }
    }

    fn block_compute(scheduler: &crate::core::JobScheduler) -> (ReleaseWorker, JobHandle) {
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let release = ReleaseWorker(Some(release_tx));
        let job = scheduler.schedule(move || {
            started_tx.send(()).unwrap();
            release_rx
                .recv_timeout(WAIT)
                .expect("owned blocker release");
        });
        started_rx.recv_timeout(WAIT).expect("real worker started");
        (release, job)
    }

    fn wait_until(mut ready: impl FnMut() -> bool) {
        let deadline = Instant::now() + WAIT;
        while !ready() {
            assert!(Instant::now() < deadline, "real worker settlement");
            std::thread::yield_now();
        }
    }

    #[test]
    fn live_commit_interruption_freezes_repeated_ticks_and_public_drain() {
        for target in [
            LifecycleTargetChange::World,
            LifecycleTargetChange::Schedule,
        ] {
            let mut fixture = Fixture::new("live-cancel");
            let first_calls = Arc::new(AtomicUsize::new(0));
            let stale_calls = Arc::new(AtomicUsize::new(0));
            let fresh_updates = Arc::new(AtomicUsize::new(0));
            let retained_original = Arc::new(Mutex::new(None));
            let first = Arc::clone(&first_calls);
            let stale = Arc::clone(&stale_calls);
            let updates = Arc::clone(&fresh_updates);
            let retained = Arc::downgrade(&retained_original);
            fixture.level.with_world_mut(|world| {
                world.observe_component_lifecycle::<Name>(
                    LifecycleEventKind::Add,
                    move |world, _| {
                        assert_eq!(first.fetch_add(1, Ordering::SeqCst), 0);
                        match target {
                            LifecycleTargetChange::World => {
                                let mut replacement = World::empty();
                                replacement.spawn_node(NodeKind::Cube).unwrap();
                                let counter = Arc::clone(&updates);
                                replacement
                                    .register_native_system::<(), _>(
                                        "tests.asset-reload.fresh",
                                        SystemStage::Update,
                                        0,
                                        move |()| {
                                            counter.fetch_add(1, Ordering::SeqCst);
                                        },
                                    )
                                    .unwrap();
                                let owner =
                                    retained.upgrade().expect("original owner remains alive");
                                *owner.lock().unwrap() =
                                    Some(std::mem::replace(world, replacement));
                            }
                            LifecycleTargetChange::Schedule => {
                                *world.schedule_mut() = Default::default();
                                let counter = Arc::clone(&updates);
                                world
                                    .register_native_system::<(), _>(
                                        "tests.asset-reload.fresh",
                                        SystemStage::Update,
                                        0,
                                        move |()| {
                                            counter.fetch_add(1, Ordering::SeqCst);
                                        },
                                    )
                                    .unwrap();
                            }
                        }
                    },
                );
                world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |_, _| {
                    stale.fetch_add(1, Ordering::SeqCst);
                });
            });
            let ids = fixture.stage_three();
            let scheduler = fixture.core.scheduler().clone();
            let first_report = fixture.queue.tick_into_level(&scheduler, &fixture.level);
            assert_eq!(first_report.failed_count(), 1);
            assert_eq!(first_report.apply.failed[0].event().handle().id(), ids[0]);
            let cause = first_report.lifecycle_interruption().unwrap().clone();
            assert_eq!(cause.lifecycle_target_change(), Some(target));
            assert_eq!(fixture.queue.lifecycle_interruption(), Some(&cause));
            assert_eq!(first_report.apply.target_stages_scheduled, 0);
            assert_eq!(first_calls.load(Ordering::SeqCst), 1);
            assert_eq!(stale_calls.load(Ordering::SeqCst), 0);
            assert_eq!(fresh_updates.load(Ordering::SeqCst), 0);
            assert_eq!(fixture.queue.target_staging_count(), 1);
            assert!(fixture.queue.target_staging.contains_key(&ids[1]));
            assert!(fixture.queue.ready.contains_key(&ids[2]));
            let metadata_bytes = fixture.queue.pending_metadata_bytes;
            let ready_bytes = fixture.queue.ready_result_bytes;
            let scheduled = fixture.queue.diagnostics().tasks_scheduled;

            let new_id = fixture.publish("lifecycle-after-cancel");
            assert!(!fixture.queue.events.is_empty());
            for _ in 0..3 {
                let report = fixture.queue.tick_into_level(&scheduler, &fixture.level);
                assert_eq!(report.lifecycle_interruption(), Some(&cause));
                assert_eq!(report.events_drained(), 0);
                assert_eq!(report.scheduled_count(), 0);
                assert_eq!(report.failed_count(), 0);
                assert_eq!(report.applied_count(), 0);
                assert_eq!(report.apply.target_stages_scheduled, 0);
                assert_eq!(report.pending_count(), 2);
                assert_eq!(report.apply.deferred_count, 1);
                let drain = fixture.queue.drain_events(&scheduler);
                assert_eq!(drain.lifecycle_interruption(), Some(&cause));
                assert_eq!(drain.events_drained, 0);
                assert_eq!(drain.scheduled, 0);
                assert_eq!(drain.pending_count, 2);
                assert_eq!(drain.pending_metadata_bytes, metadata_bytes);
                assert_eq!(fixture.queue.pending_metadata_bytes, metadata_bytes);
                assert_eq!(fixture.queue.ready_result_bytes, ready_bytes);
                assert_eq!(fixture.queue.diagnostics().tasks_scheduled, scheduled);
                assert!(!fixture.queue.latest_revisions.contains_key(&new_id));
                assert!(!fixture.queue.events.is_empty());
                assert!(fixture.queue.target_staging.contains_key(&ids[1]));
                assert!(fixture.queue.ready.contains_key(&ids[2]));
                assert!(!fixture.queue.has_pending_work());
            }
            let direct_commit = fixture.queue.commit_staged_into_level(&fixture.level);
            assert_eq!(direct_commit.lifecycle_interruption(), Some(&cause));
            assert_eq!(direct_commit.failed_count(), 0);
            let mut isolated = World::empty();
            let world_tick = fixture.queue.tick_into(&scheduler, &mut isolated);
            assert_eq!(world_tick.lifecycle_interruption(), Some(&cause));
            assert_eq!(isolated.node_records().len(), 0);
            fixture.runtime.deactivate_module(MODULE).unwrap();
            fixture
                .queue
                .install_runtime_frame_wake(Arc::new(|| {}))
                .expect("terminal wake rebinding must leave retained rows alone");
            assert_eq!(fixture.queue.source_error, None);
            assert!(fixture.queue.ready.contains_key(&ids[2]));
            assert!(fixture.queue.target_staging.contains_key(&ids[1]));
            assert_eq!(fixture.queue.pending_metadata_bytes, metadata_bytes);
            assert_eq!(fixture.queue.lifecycle_interruption(), Some(&cause));
            assert_eq!(first_calls.load(Ordering::SeqCst), 1);
            assert_eq!(stale_calls.load(Ordering::SeqCst), 0);
            assert_eq!(fresh_updates.load(Ordering::SeqCst), 0);
        }
    }

    #[test]
    fn preflight_target_change_is_recoverable_without_terminal_interruption() {
        let mut fixture = Fixture::new("preflight-recovery");
        let ids = fixture.stage_three();
        fixture.level.with_world_mut(|world| {
            world.spawn_node(NodeKind::Cube).unwrap();
        });
        let scheduler = fixture.core.scheduler().clone();
        let rejected = fixture.queue.tick_into_level(&scheduler, &fixture.level);
        assert_eq!(rejected.failed_count(), 2);
        assert!(rejected.apply.failed.iter().all(|failure| matches!(
            failure.error(),
            DynamicSceneError::TargetWorldChanged { .. }
        )));
        assert!(rejected.lifecycle_interruption().is_none());
        assert!(fixture.queue.lifecycle_interruption().is_none());
        assert_eq!(rejected.apply.target_stages_scheduled, 1);
        assert!(!fixture.queue.ready.contains_key(&ids[2]));
        assert_eq!(fixture.queue.target_staging_count(), 1);
        wait_until(|| fixture.queue.target_stages_ready());
        let recovered = fixture.queue.tick_into_level(&scheduler, &fixture.level);
        assert_eq!(recovered.applied_count(), 1);
        assert_eq!(recovered.apply.applied[0].event().handle().id(), ids[2]);
        assert!(recovered.lifecycle_interruption().is_none());
        assert_eq!(fixture.queue.pending_count(), 0);
    }
}
