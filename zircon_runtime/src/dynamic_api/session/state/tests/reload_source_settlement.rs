use std::fs;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc};
use std::time::{Duration, Instant};

use zircon_runtime_interface::project::RelPath;

use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::{
    project_asset_manager_handle, AssetEvent, AssetUri, SceneAsset, ASSET_MODULE_NAME,
};
use crate::core::framework::project::ProjectPluginManifest;
use crate::core::manager::resolve_manager_service;
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};
use crate::core::runtime::tasks::{JobHandle, JobScheduler, TaskPoolKind};
use crate::core::{CoreError, LifecycleState, TaskState};

use super::super::{
    RuntimeDynamicSession, RuntimeDynamicSessionError, RuntimeDynamicSessionProfile,
    RuntimeFrameDemand, RuntimeProjectConfig,
};

const WAIT: Duration = Duration::from_secs(5);
const BLOCK_LIMIT: Duration = Duration::from_secs(30);
const QUEUED_SCENE: &str = "res://scenes/queued-source.scene.toml";

#[test]
fn session_retains_real_source_failure_and_settles_its_original_worker_once() {
    let project = OwnedProjectFixture::new();
    let config = RuntimeProjectConfig::from_root(&project.root).expect("actual project config");
    let mut session =
        RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, Some(config))
            .expect("actual registered-source Session");

    // Blockers are owned inside this scope. Unwind releases them before the
    // outer scope invokes the real Session shutdown and Core worker joins.
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        let core = session.runtime.handle();
        let source = project_asset_manager_handle(&core).expect("canonical Asset source token");
        {
            let manager = resolve_manager_service(&core, source.clone())
                .expect("ordinary source call admission");
            assert!(manager.shutdown_project_watchers_until(Instant::now() + WAIT));
        }
        settle_initial_project_events(&mut session);
        assert_eq!(session.frame_demand(), RuntimeFrameDemand::Idle);

        let scheduler = core.scheduler().clone();
        let mut blockers = ComputeBlockers::start(&scheduler);
        let scene = project.write_queued_scene();
        let locator = AssetUri::parse(QUEUED_SCENE).expect("legitimate scene locator");
        let id = ResourceId::from_locator(&locator);
        let published = {
            let manager = resolve_manager_service(&core, source.clone())
                .expect("normal admission for real resource publication");
            manager
                .assets::<SceneAsset>()
                .insert(ResourceRecord::new(id, ResourceKind::Scene, locator), scene)
                .expect("canonical resource commit publishes its actual Added event")
        };
        assert_eq!(published.id(), id);
        session
            .tick_scene_asset_reload()
            .expect("normal Session helper admits the actual event");
        let admission = session
            .last_scene_asset_reload_report
            .as_ref()
            .expect("full admission report");
        assert_eq!(admission.events_drained(), 1);
        assert_eq!(admission.scheduled_count(), 1);
        assert_eq!(admission.pending_count(), 1);
        assert!(admission.drain.source_error.is_none());
        assert_eq!(admission.applied_count(), 0);
        let original_task_id = {
            let queue = session.scene_asset_reload_queue.as_ref().unwrap();
            assert_eq!(queue.pending_tasks().len(), 1);
            let pending = queue.pending_tasks().next().unwrap();
            assert!(matches!(pending.event(), AssetEvent::Added { .. }));
            assert_eq!(pending.event().handle().id(), id);
            assert!(!pending.is_ready());
            pending.task().status_snapshot().id
        };
        assert_eq!(session.frame_demand(), RuntimeFrameDemand::Immediate);

        // This public zero-duration API is an immediate module probe, not an
        // expired Instant deadline and not a fabricated ModuleShutdownReport.
        let probe = session
            .runtime
            .shutdown_registered_modules_with_drain_timeout(Duration::ZERO);
        let asset_state = core
            .inner
            .modules
            .lock()
            .expect("actual module ledger")
            .get(ASSET_MODULE_NAME)
            .expect("Asset remains registered")
            .lifecycle;
        assert_eq!(
            asset_state,
            LifecycleState::Unloaded,
            "real Asset retirement is a required precondition: {probe:?}"
        );
        let source_error = resolve_manager_service(&core, source.clone())
            .err()
            .expect("the original token must be refused by normal admission");

        for attempt in 0..2 {
            assert_source_failure(&mut session, &source_error);
            let report = session.last_scene_asset_reload_report.as_ref().unwrap();
            assert_eq!(report.drain.source_error.as_ref(), Some(&source_error));
            assert_eq!(report.superseded_pending_count(), usize::from(attempt == 0));
            assert_eq!(report.scheduled_count(), 0);
            assert_eq!(report.applied_count(), 0);
            assert_eq!(report.pending_count(), 1);
            assert!(!report.drain.receiver_disconnected);
            let queue = session.scene_asset_reload_queue.as_ref().unwrap();
            assert!(queue.has_pending_work());
            let pending = queue.pending_tasks().next().unwrap();
            assert_eq!(pending.task().status_snapshot().id, original_task_id);
            assert_eq!(pending.event().handle().id(), id);
            assert!(pending.task().is_cancellation_requested());
            assert!(!pending.is_ready());
        }
        blockers.assert_no_timeout();
        blockers.release_and_observe_terminal();

        // Observe the actual original TaskHandle terminal state before any
        // helper can remove its row. A cancellation request alone is not ready.
        let ready_deadline = Instant::now() + WAIT;
        loop {
            let queue = session.scene_asset_reload_queue.as_ref().unwrap();
            assert_eq!(queue.pending_tasks().len(), 1);
            let pending = queue.pending_tasks().next().unwrap();
            let status = pending.task().status_snapshot();
            assert_eq!(status.id, original_task_id);
            if pending.is_ready() {
                assert!(status.is_terminal());
                assert_eq!(status.state, TaskState::Cancelled);
                break;
            }
            assert!(
                Instant::now() < ready_deadline,
                "original worker did not settle"
            );
            std::thread::yield_now();
        }
        for _ in 0..2 {
            assert_source_failure(&mut session, &source_error);
            let report = session.last_scene_asset_reload_report.as_ref().unwrap();
            assert_eq!(report.drain.source_error.as_ref(), Some(&source_error));
            assert_eq!(report.superseded_pending_count(), 0);
            assert_eq!(report.scheduled_count(), 0);
            assert_eq!(report.applied_count(), 0);
            assert_eq!(report.pending_count(), 0);
            assert!(!report.drain.receiver_disconnected);
            let queue = session.scene_asset_reload_queue.as_ref().unwrap();
            assert_eq!(queue.pending_tasks().len(), 0);
            assert!(!queue.has_pending_work());
            assert_eq!(session.frame_demand(), RuntimeFrameDemand::Idle);
        }
    }));

    let cleanup = catch_unwind(AssertUnwindSafe(|| {
        let session_stopped = session.shutdown_before_library_unload_with_timeout(WAIT);
        let receipt = session.runtime.shutdown_until(Instant::now() + WAIT);
        assert!(session_stopped, "actual Session cleanup must finish");
        let receipt = receipt.expect("actual Core shutdown retry and worker-join receipt");
        assert!(receipt.worker_shutdown(TaskPoolKind::Compute).is_some());
        assert!(!receipt.has_in_flight_work(), "{receipt:?}");
    }));
    if let Err(payload) = outcome {
        resume_unwind(payload);
    }
    if let Err(payload) = cleanup {
        resume_unwind(payload);
    }
}

fn assert_source_failure(session: &mut RuntimeDynamicSession, expected: &CoreError) {
    match session.tick_scene_asset_reload() {
        Err(RuntimeDynamicSessionError::CoreStep { step, source }) => {
            assert_eq!(step, "drain scene asset reload source");
            assert_eq!(&source, expected);
        }
        result => panic!("expected the real retained source failure, got {result:?}"),
    }
}

fn settle_initial_project_events(session: &mut RuntimeDynamicSession) {
    let deadline = Instant::now() + WAIT;
    loop {
        session
            .tick_scene_asset_reload()
            .expect("normal initial project event settlement");
        let queue = session
            .scene_asset_reload_queue
            .as_ref()
            .expect("prepared project owns its real queue");
        if !queue.has_pending_work() {
            assert_eq!(queue.pending_count(), 0);
            return;
        }
        assert!(
            Instant::now() < deadline,
            "initial project work did not settle"
        );
        std::thread::yield_now();
    }
}

struct ComputeBlockers {
    releases: Vec<mpsc::Sender<()>>,
    jobs: Vec<JobHandle>,
    timed_out: Arc<AtomicBool>,
}

impl ComputeBlockers {
    fn start(scheduler: &JobScheduler) -> Self {
        let count = scheduler.parallelism();
        assert!(count > 0);
        let (started_tx, started_rx) = mpsc::channel();
        let mut owned = Self {
            releases: Vec::with_capacity(count),
            jobs: Vec::with_capacity(count),
            timed_out: Arc::new(AtomicBool::new(false)),
        };
        for _ in 0..count {
            let (release_tx, release_rx) = mpsc::channel();
            owned.releases.push(release_tx);
            let started = started_tx.clone();
            let timed_out = Arc::clone(&owned.timed_out);
            owned.jobs.push(scheduler.schedule(move || {
                let _ = started.send(());
                if matches!(
                    release_rx.recv_timeout(BLOCK_LIMIT),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ) {
                    timed_out.store(true, Ordering::Release);
                }
            }));
        }
        drop(started_tx);
        let deadline = Instant::now() + WAIT;
        for _ in 0..count {
            started_rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("all actual Compute workers must acknowledge their blockers");
        }
        owned
    }

    fn release(&mut self) {
        for sender in self.releases.drain(..) {
            let _ = sender.send(());
        }
    }

    fn assert_no_timeout(&self) {
        assert!(
            !self.timed_out.load(Ordering::Acquire),
            "blocker auto-release invalidated the probe"
        );
    }

    fn release_and_observe_terminal(&mut self) {
        self.release();
        let deadline = Instant::now() + WAIT;
        while self.jobs.iter().any(|job| !job.is_complete()) {
            assert!(
                Instant::now() < deadline,
                "released blocker jobs did not settle"
            );
            std::thread::yield_now();
        }
        self.assert_no_timeout();
    }
}

impl Drop for ComputeBlockers {
    fn drop(&mut self) {
        self.release();
    }
}

struct OwnedProjectFixture {
    root: PathBuf,
    parent: PathBuf,
}

impl OwnedProjectFixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let executable = std::env::current_exe().unwrap().canonicalize().unwrap();
        let parent = executable
            .parent()
            .unwrap()
            .join("session-reload-source-fixtures");
        fs::create_dir_all(&parent).expect("executable fixture parent");
        assert_eq!(
            parent.canonicalize().unwrap(),
            parent,
            "fixture parent alias"
        );
        for _ in 0..128 {
            let root = parent.join(format!(
                "settlement-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&root) {
                Ok(()) => {
                    let owned = Self { root, parent };
                    let resolved = owned.root.canonicalize().unwrap();
                    assert_eq!(resolved, owned.root);
                    assert_eq!(resolved.parent(), Some(owned.parent.as_path()));
                    assert!(resolved.is_absolute());
                    owned.write_project();
                    return owned;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("exclusive project fixture: {error}"),
            }
        }
        panic!("exclusive fixture attempts exhausted; existing paths preserved");
    }

    fn write_project(&self) {
        let paths = ProjectPaths::from_root(&self.root).expect("owned project paths");
        paths
            .ensure_layout(&[RelPath::project_assets()])
            .expect("actual project layout");
        let mut manifest = ProjectManifest::new(
            "session-reload-source",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        );
        manifest.plugins = ProjectPluginManifest::default();
        manifest
            .save(paths.manifest_path())
            .expect("actual project manifest");
        let scenes = paths.asset_root(&RelPath::project_assets()).join("scenes");
        fs::create_dir_all(&scenes).expect("owned scene input directory");
        fs::write(
            scenes.join("main.scene.toml"),
            SceneAsset {
                entities: Vec::new(),
            }
            .to_toml_string()
            .unwrap(),
        )
        .expect("actual empty default scene");
    }

    fn write_queued_scene(&self) -> SceneAsset {
        let paths = ProjectPaths::from_root(&self.root).unwrap();
        let scene = SceneAsset {
            entities: Vec::new(),
        };
        fs::write(
            paths
                .asset_root(&RelPath::project_assets())
                .join("scenes/queued-source.scene.toml"),
            scene.to_toml_string().unwrap(),
        )
        .expect("legitimate additional scene input");
        scene
    }
}

impl Drop for OwnedProjectFixture {
    fn drop(&mut self) {
        let (Ok(parent), Ok(root)) = (self.parent.canonicalize(), self.root.canonicalize()) else {
            return;
        };
        if !root.is_absolute()
            || parent != self.parent
            || root != self.root
            || root.parent() != Some(parent.as_path())
        {
            return;
        }
        let _ = fs::remove_dir_all(root);
    }
}

mod lifecycle_interruption {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;

    use zircon_runtime_interface::world_sync::{LifecycleTargetChange, WorldFact};

    use crate::asset::AssetManager;
    use crate::scene::components::Name;
    use crate::scene::ecs::{LifecycleEventKind, SystemStage};
    use crate::scene::{
        DynamicSceneAssetReloadLimits, DynamicSceneAssetReloadQueue, DynamicSceneError,
        LevelMetadata, LevelSystem, NodeKind, World,
    };

    fn publish_scene(session: &RuntimeDynamicSession, root: &std::path::Path, label: &str) {
        publish_scene_fixture(session, root, label, false);
    }

    fn publish_scene_fixture(
        session: &RuntimeDynamicSession,
        root: &std::path::Path,
        label: &str,
        with_camera: bool,
    ) {
        let mut scene: SceneAsset = serde_json::from_value(serde_json::json!({
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
        if with_camera {
            scene.entities[0].camera = Some(Default::default());
            scene.entities[0].transform.translation = [3.0, 2.0, 5.0];
        }
        let paths = ProjectPaths::from_root(root).unwrap();
        let document = scene
            .to_project_toml_string(|_| unreachable!("the node fixture has no asset references"))
            .unwrap();
        fs::write(
            paths
                .asset_root(&RelPath::project_assets())
                .join(format!("scenes/{label}.scene.toml")),
            document,
        )
        .unwrap();
        let locator = AssetUri::parse(&format!("res://scenes/{label}.scene.toml")).unwrap();
        let id = ResourceId::from_locator(&locator);
        let core = session.runtime.handle();
        let source = project_asset_manager_handle(&core).unwrap();
        let manager = resolve_manager_service(&core, source).unwrap();
        assert_eq!(
            manager
                .assets::<SceneAsset>()
                .insert(ResourceRecord::new(id, ResourceKind::Scene, locator), scene)
                .unwrap()
                .id(),
            id
        );
    }

    fn prepare_three(session: &mut RuntimeDynamicSession, project: &OwnedProjectFixture) {
        prepare_three_fixture(session, project, false);
    }

    fn prepare_three_fixture(
        session: &mut RuntimeDynamicSession,
        project: &OwnedProjectFixture,
        with_camera: bool,
    ) {
        let core = session.runtime.handle();
        let source = project_asset_manager_handle(&core).unwrap();
        let manager = resolve_manager_service(&core, source.clone()).unwrap();
        assert!(manager.shutdown_project_watchers_until(Instant::now() + WAIT));
        let project_manager = manager.current_project_manager().unwrap();
        drop(manager);
        settle_initial_project_events(session);
        let limits = DynamicSceneAssetReloadLimits {
            max_apply_per_tick: 2,
            max_ready_per_tick: 3,
            event_time_budget: Duration::MAX,
            ready_time_budget: Duration::MAX,
            apply_time_budget: Duration::MAX,
            ..Default::default()
        };
        session.scene_asset_reload_queue = Some(
            DynamicSceneAssetReloadQueue::from_project_asset_manager_with_limits(
                project_manager,
                &core,
                source,
                limits,
            )
            .unwrap()
            .with_task_graph_scope(session.task_graph_scope.clone()),
        );
        let scheduler = core.scheduler().clone();
        let mut blockers = ComputeBlockers::start(&scheduler);
        for label in ["lifecycle-first", "lifecycle-tail", "lifecycle-ready"] {
            publish_scene_fixture(session, &project.root, label, with_camera);
        }
        session.tick_scene_asset_reload().unwrap();
        assert_eq!(
            session
                .last_scene_asset_reload_report
                .as_ref()
                .unwrap()
                .scheduled_count(),
            3
        );
        assert_eq!(
            session
                .scene_asset_reload_queue
                .as_ref()
                .unwrap()
                .pending_tasks()
                .len(),
            3
        );
        blockers.release_and_observe_terminal();
        wait_until(|| {
            session
                .scene_asset_reload_queue
                .as_ref()
                .unwrap()
                .pending_tasks()
                .all(|task| task.task().is_ready())
        });
        let mut blockers = ComputeBlockers::start(&scheduler);
        session.tick_scene_asset_reload().unwrap();
        let report = session.last_scene_asset_reload_report.as_ref().unwrap();
        assert_eq!(report.apply.target_stages_scheduled, 2);
        let queue = session.scene_asset_reload_queue.as_ref().unwrap();
        assert_eq!(queue.target_staging_count(), 2);
        assert_eq!(queue.pending_count(), 3);
        blockers.release_and_observe_terminal();
        wait_until(|| {
            session
                .scene_asset_reload_queue
                .as_ref()
                .unwrap()
                .target_stages_ready()
        });
        session.level.drain_world_invalidations();
    }

    fn wait_until(mut ready: impl FnMut() -> bool) {
        let deadline = Instant::now() + WAIT;
        while !ready() {
            assert!(Instant::now() < deadline, "real reload worker settlement");
            std::thread::yield_now();
        }
    }

    fn is_reload_fact(fact: &WorldFact) -> bool {
        matches!(fact, WorldFact::AssetReloadApplied(_))
    }

    fn assert_interrupted(
        result: Result<(), RuntimeDynamicSessionError>,
        target: LifecycleTargetChange,
    ) {
        match result {
            Err(RuntimeDynamicSessionError::SceneAssetReloadInterrupted { source }) => {
                assert_eq!(source.lifecycle_target_change(), Some(target));
            }
            other => panic!("typed live lifecycle cancellation required: {other:?}"),
        }
    }

    fn finish_session(session: &mut RuntimeDynamicSession) {
        // Explicit disposal releases the frozen target-bound rows before Core shutdown.
        drop(session.scene_asset_reload_queue.take());
        assert!(session.shutdown_before_library_unload_with_timeout(WAIT));
        let receipt = session
            .runtime
            .shutdown_until(Instant::now() + WAIT)
            .unwrap();
        assert!(!receipt.has_in_flight_work(), "{receipt:?}");
    }

    #[test]
    fn session_live_reload_interruption_stops_world_facts_and_later_frames() {
        for target in [
            LifecycleTargetChange::World,
            LifecycleTargetChange::Schedule,
        ] {
            let project = OwnedProjectFixture::new();
            let config = RuntimeProjectConfig::from_root(&project.root).unwrap();
            let mut session =
                RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, Some(config))
                    .expect("real project Session");
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let first_calls = Arc::new(AtomicUsize::new(0));
                let stale_calls = Arc::new(AtomicUsize::new(0));
                let fresh_updates = Arc::new(AtomicUsize::new(0));
                let original_owner = Arc::new(Mutex::new(None));

                let mut fresh_world = World::empty();
                fresh_world.spawn_node(NodeKind::Cube).unwrap();
                let updates = Arc::clone(&fresh_updates);
                fresh_world
                    .register_native_system::<(), _>(
                        "tests.session.asset-reload.fresh",
                        SystemStage::Update,
                        0,
                        move |()| {
                            updates.fetch_add(1, Ordering::SeqCst);
                        },
                    )
                    .unwrap();
                // This actual Level attaches a live sink. Keep its owner while its World is moved.
                let replacement_fact_owner = LevelSystem::new(
                    crate::core::framework::scene::WorldHandle::new(902),
                    Arc::new(Mutex::new(fresh_world)),
                    LevelMetadata::default(),
                );
                replacement_fact_owner
                    .record_world_fact(WorldFact::AssetReloadApplied(Default::default()));
                assert!(
                    replacement_fact_owner
                        .drain_world_invalidations()
                        .iter()
                        .flat_map(|batch| batch.facts.iter())
                        .any(is_reload_fact),
                    "the replacement sink must really observe reload facts"
                );
                let replacement = replacement_fact_owner
                    .with_world_mut(|world| std::mem::replace(world, World::empty()));
                let replacement = Arc::new(Mutex::new(Some(replacement)));
                let first = Arc::clone(&first_calls);
                let stale = Arc::clone(&stale_calls);
                let retained = Arc::downgrade(&original_owner);
                let fresh = Arc::clone(&fresh_updates);
                session.level.with_world_mut(|world| {
                    world.observe_component_lifecycle::<Name>(
                        LifecycleEventKind::Add,
                        // SAFETY: This retires the current World or schedule. Remaining work is
                        // counter-only scheduling; replacement history is never adopted.
                        unsafe {
                            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                move |world, _| {
                                    assert_eq!(first.fetch_add(1, Ordering::SeqCst), 0);
                                    match target {
                                        LifecycleTargetChange::World => {
                                            let fresh_world = replacement.lock().unwrap().take().unwrap();
                                            let owner =
                                                retained.upgrade().expect("original owner remains alive");
                                            *owner.lock().unwrap() =
                                                Some(std::mem::replace(world, fresh_world));
                                        }
                                        LifecycleTargetChange::Schedule => {
                                            *world.schedule_mut() = Default::default();
                                            let updates = Arc::clone(&fresh);
                                            world
                                                .register_native_system::<(), _>(
                                                    "tests.session.asset-reload.fresh",
                                                    SystemStage::Update,
                                                    0,
                                                    move |()| {
                                                        updates.fetch_add(1, Ordering::SeqCst);
                                                    },
                                                )
                                                .unwrap();
                                        }
                                    }
                                },
                            )
                        },
                    );
                    world.observe_component_lifecycle::<Name>(
                        LifecycleEventKind::Add,
                        // SAFETY: Only an external atomic counter changes; retained World
                        // payload and dynamic JSON are untouched.
                        unsafe {
                            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                move |_, _| {
                                    stale.fetch_add(1, Ordering::SeqCst);
                                },
                            )
                        },
                    );
                });
                prepare_three(&mut session, &project);
                // Calibrate the original sink too, before the canceling live commit.
                session
                    .level
                    .record_world_fact(WorldFact::AssetReloadApplied(Default::default()));
                assert!(session
                    .level
                    .drain_world_invalidations()
                    .iter()
                    .flat_map(|batch| batch.facts.iter())
                    .any(is_reload_fact));
                #[cfg(feature = "animation")]
                session
                    .level
                    .record_animation_requires_continuous_frame(true);
                assert_interrupted(session.tick_frame(), target);
                let report = session.last_scene_asset_reload_report.as_ref().unwrap();
                assert_eq!(report.failed_count(), 1);
                assert_eq!(
                    report
                        .lifecycle_interruption()
                        .unwrap()
                        .lifecycle_target_change(),
                    Some(target)
                );
                assert!(super::super::super::asset_reload_world_fact(report).is_none());
                assert_eq!(report.apply.target_stages_scheduled, 0);
                assert_eq!(
                    session
                        .scene_asset_reload_queue
                        .as_ref()
                        .unwrap()
                        .pending_count(),
                    2
                );
                assert_eq!(first_calls.load(Ordering::SeqCst), 1);
                assert_eq!(stale_calls.load(Ordering::SeqCst), 0);
                assert_eq!(fresh_updates.load(Ordering::SeqCst), 0);
                let original_report = report.clone();
                let timing = session.last_render_frame_timing;
                assert!(!session
                    .level
                    .drain_world_invalidations()
                    .iter()
                    .flat_map(|batch| batch.facts.iter())
                    .any(is_reload_fact));
                assert!(!replacement_fact_owner
                    .drain_world_invalidations()
                    .iter()
                    .flat_map(|batch| batch.facts.iter())
                    .any(is_reload_fact));
                publish_scene(&session, &project.root, "lifecycle-after-cancel");
                for _ in 0..3 {
                    assert_interrupted(session.tick_frame(), target);
                    assert_interrupted(session.tick_scene_asset_reload(), target);
                    session.reset_frame_demand_after_failed_tick();
                    #[cfg(feature = "animation")]
                    assert!(session.level.animation_requires_continuous_frame());
                    assert_eq!(session.frame_demand(), RuntimeFrameDemand::Idle);
                    assert_eq!(session.last_render_frame_timing, timing);
                    assert_eq!(
                        session.last_scene_asset_reload_report.as_ref(),
                        Some(&original_report)
                    );
                    let scheduler = session.runtime.handle().scheduler().clone();
                    let frame = session
                        .scene_asset_reload_queue
                        .as_mut()
                        .unwrap()
                        .tick_into_level(&scheduler, &session.level);
                    assert_eq!(
                        frame
                            .lifecycle_interruption()
                            .unwrap()
                            .lifecycle_target_change(),
                        Some(target)
                    );
                    assert_eq!(frame.events_drained(), 0);
                    assert_eq!(frame.applied_count(), 0);
                    assert_eq!(frame.apply.target_stages_scheduled, 0);
                    assert_eq!(frame.pending_count(), 2);
                    assert!(!session
                        .level
                        .drain_world_invalidations()
                        .iter()
                        .flat_map(|batch| batch.facts.iter())
                        .any(is_reload_fact));
                    assert!(!replacement_fact_owner
                        .drain_world_invalidations()
                        .iter()
                        .flat_map(|batch| batch.facts.iter())
                        .any(is_reload_fact));
                }
                assert_eq!(first_calls.load(Ordering::SeqCst), 1);
                assert_eq!(stale_calls.load(Ordering::SeqCst), 0);
                assert_eq!(fresh_updates.load(Ordering::SeqCst), 0);
            }));
            let cleanup = catch_unwind(AssertUnwindSafe(|| finish_session(&mut session)));
            if let Err(payload) = outcome {
                resume_unwind(payload);
            }
            if let Err(payload) = cleanup {
                resume_unwind(payload);
            }
        }
    }

    #[test]
    fn session_preflight_failure_keeps_ordinary_frame_progress_and_fact_publication() {
        let project = OwnedProjectFixture::new();
        let config = RuntimeProjectConfig::from_root(&project.root).unwrap();
        let mut session =
            RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, Some(config))
                .unwrap();
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            prepare_three(&mut session, &project);
            session.level.with_world_mut(|world| {
                world.spawn_node(NodeKind::Cube).unwrap();
            });
            session.level.drain_world_invalidations();
            session.tick_scene_asset_reload().unwrap();
            let rejected = session.last_scene_asset_reload_report.as_ref().unwrap();
            assert_eq!(rejected.failed_count(), 2);
            assert!(rejected.apply.failed.iter().all(|failure| matches!(
                failure.error(),
                DynamicSceneError::TargetWorldChanged { .. }
            )));
            assert!(rejected.lifecycle_interruption().is_none());
            assert_eq!(rejected.apply.target_stages_scheduled, 1);
            assert!(session
                .level
                .drain_world_invalidations()
                .iter()
                .flat_map(|batch| batch.facts.iter())
                .any(is_reload_fact));
            wait_until(|| {
                session
                    .scene_asset_reload_queue
                    .as_ref()
                    .unwrap()
                    .target_stages_ready()
            });
            session
                .tick_frame()
                .expect("ordinary preflight failure must allow a later fresh commit and frame");
            let recovered = session.last_scene_asset_reload_report.as_ref().unwrap();
            assert_eq!(recovered.applied_count(), 1);
            assert!(recovered.lifecycle_interruption().is_none());
            assert_eq!(
                session
                    .scene_asset_reload_queue
                    .as_ref()
                    .unwrap()
                    .pending_count(),
                0
            );
        }));
        let cleanup = catch_unwind(AssertUnwindSafe(|| finish_session(&mut session)));
        if let Err(payload) = outcome {
            resume_unwind(payload);
        }
        if let Err(payload) = cleanup {
            resume_unwind(payload);
        }
    }

    fn owned_session_call<T: Send>(
        handle: zircon_runtime_interface::ZrRuntimeSessionHandle,
        action: impl FnOnce(&mut RuntimeDynamicSession) -> T + Send,
    ) -> T {
        let mut returned = None;
        let status = crate::dynamic_api::session::registry::with_session(handle, |session| {
            returned = Some(action(session));
            zircon_runtime_interface::ZrStatus::ok()
        });
        assert_eq!(
            status.status_code(),
            zircon_runtime_interface::ZrStatusCode::Ok
        );
        returned.expect("the real owner must execute the scoped observation")
    }

    fn register_transform_probe(world: &mut World, calls: &Arc<AtomicUsize>) {
        let calls = Arc::clone(calls);
        world.observe_component_lifecycle::<crate::scene::components::LocalTransform>(
            LifecycleEventKind::Insert,
            // SAFETY: Only an external atomic counter changes; retained World payload and
            // dynamic JSON are untouched.
            unsafe {
                use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_, _| {
                    calls.fetch_add(1, Ordering::SeqCst);
                })
            },
        );
    }

    fn valid_owned_apply(
        session: &mut RuntimeDynamicSession,
    ) -> zircon_runtime_interface::ZrRuntimeEditorTransformWriteV1 {
        use zircon_runtime_interface::{
            ZrRuntimeEditorTransformPhaseV1, ZrRuntimeEditorTransformWriteV1,
        };
        session
            .level
            .with_world_and_replacement_epoch(|world, epoch| {
                let entity = world.active_camera();
                let expected = world
                    .local_transform(entity)
                    .expect("the admitted scene fixture has a real active camera");
                let target = crate::core::math::Transform {
                    translation: expected.translation + crate::core::math::Vec3::new(4.0, 0.0, 0.0),
                    ..expected
                };
                assert_ne!(expected, target);
                ZrRuntimeEditorTransformWriteV1::new(
                    entity,
                    901,
                    1,
                    epoch,
                    ZrRuntimeEditorTransformPhaseV1::Apply,
                    expected,
                    target,
                )
            })
    }

    fn retained_camera_transform(
        owner: &Mutex<Option<World>>,
    ) -> Option<(crate::scene::EntityId, crate::core::math::Transform)> {
        owner.lock().unwrap().as_ref().map(|world| {
            let entity = world.active_camera();
            (
                entity,
                world
                    .local_transform(entity)
                    .expect("the original staged camera stays owned"),
            )
        })
    }

    fn assert_owned_interrupted(status: zircon_runtime_interface::ZrStatus, expected: &str) {
        assert_eq!(
            status.status_code(),
            zircon_runtime_interface::ZrStatusCode::Error
        );
        assert_eq!(
            unsafe { status.diagnostics.checked_slice(4096) }.unwrap(),
            expected.as_bytes()
        );
    }

    fn dispose_owned_or_local(
        local: &mut Option<RuntimeDynamicSession>,
        owned: Option<zircon_runtime_interface::ZrRuntimeSessionHandle>,
    ) {
        if let Some(handle) = owned {
            // Release only the queue through the real owner, then run the public teardown route.
            let disposal = catch_unwind(AssertUnwindSafe(|| {
                owned_session_call(handle, |session| {
                    drop(session.scene_asset_reload_queue.take());
                });
            }));
            let status = unsafe { crate::dynamic_api::session::ffi::destroy_session(handle) };
            if let Err(payload) = disposal {
                resume_unwind(payload);
            }
            assert_eq!(
                status.status_code(),
                zircon_runtime_interface::ZrStatusCode::Ok
            );
        } else if let Some(session) = local.as_mut() {
            finish_session(session);
        }
    }

    #[test]
    fn owned_events_reject_scene_writes_after_real_lifecycle_interruption() {
        use crate::dynamic_api::session::{ffi, registry::insert_session};
        use zircon_runtime_interface::{
            ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatusCode,
            ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
            ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1, ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
            ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1, ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1,
        };

        for target in [
            LifecycleTargetChange::Schedule,
            LifecycleTargetChange::World,
        ] {
            let project = OwnedProjectFixture::new();
            let config = RuntimeProjectConfig::from_root(&project.root).unwrap();
            let mut local = Some(
                RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, Some(config))
                    .unwrap(),
            );
            let mut owned = None;
            let calls = Arc::new(AtomicUsize::new(0));
            let first_calls = Arc::new(AtomicUsize::new(0));
            let stale_calls = Arc::new(AtomicUsize::new(0));
            let original_owner = Arc::new(Mutex::new(None::<World>));
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                let session = local.as_mut().unwrap();
                assert!(
                    session.runtime_ui.is_empty(),
                    "the wheel must reach the actual camera path without UI swallowing"
                );
                let mut fresh_world = World::empty();
                let camera = fresh_world.spawn_node(NodeKind::Camera).unwrap();
                fresh_world.set_active_camera(camera);
                fresh_world
                    .update_transform(
                        camera,
                        crate::core::math::Transform::looking_at(
                            crate::core::math::Vec3::new(3.0, 2.0, 5.0),
                            crate::core::math::Vec3::ZERO,
                            crate::core::math::Vec3::Y,
                        ),
                    )
                    .unwrap();
                register_transform_probe(&mut fresh_world, &calls);
                let replacement = Arc::new(Mutex::new(Some(fresh_world)));
                let retained = Arc::downgrade(&original_owner);
                let first = Arc::clone(&first_calls);
                let stale = Arc::clone(&stale_calls);
                let probe = Arc::clone(&calls);
                session.level.with_world_mut(|world| {
                    register_transform_probe(world, &calls);
                    world.observe_component_lifecycle::<Name>(
                        LifecycleEventKind::Add,
                        // SAFETY: This retires the current World or schedule. Remaining work is
                        // counter-only scheduling; replacement history is never adopted.
                        unsafe {
                            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                move |world, _| {
                                    assert_eq!(first.fetch_add(1, Ordering::SeqCst), 0);
                                    match target {
                                        LifecycleTargetChange::World => {
                                            let replacement = replacement.lock().unwrap().take().unwrap();
                                            *retained
                                                .upgrade()
                                                .expect("old World owner remains alive")
                                                .lock()
                                                .unwrap() = Some(std::mem::replace(world, replacement));
                                        }
                                        LifecycleTargetChange::Schedule => {
                                            *world.schedule_mut() = Default::default();
                                            register_transform_probe(world, &probe);
                                        }
                                    }
                                },
                            )
                        },
                    );
                    world.observe_component_lifecycle::<Name>(
                        LifecycleEventKind::Add,
                        // SAFETY: Only an external atomic counter changes; retained World
                        // payload and dynamic JSON are untouched.
                        unsafe {
                            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
                            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(
                                move |_, _| {
                                    stale.fetch_add(1, Ordering::SeqCst);
                                },
                            )
                        },
                    );
                });
                prepare_three_fixture(session, &project, true);
                let before_epoch = session.level.capture_world_replacement_epoch();
                assert_interrupted(session.tick_frame(), target);
                let first_report = session
                    .last_scene_asset_reload_report
                    .as_ref()
                    .unwrap()
                    .clone();
                let cause = first_report.lifecycle_interruption().unwrap().clone();
                assert_eq!(cause.lifecycle_target_change(), Some(target));
                assert_eq!(first_report.failed_count(), 1);
                assert_eq!(first_calls.load(Ordering::SeqCst), 1);
                assert_eq!(stale_calls.load(Ordering::SeqCst), 0);
                let pending = session
                    .scene_asset_reload_queue
                    .as_ref()
                    .unwrap()
                    .pending_count();
                let staging = session
                    .scene_asset_reload_queue
                    .as_ref()
                    .unwrap()
                    .target_staging_count();
                let scheduled = session
                    .scene_asset_reload_queue
                    .as_ref()
                    .unwrap()
                    .diagnostics()
                    .tasks_scheduled;
                assert_eq!(pending, 2);
                assert_eq!(staging, 1);
                let timing = session.last_render_frame_timing;
                let retained_transform = retained_camera_transform(&original_owner);
                publish_scene(session, &project.root, "owned-event-after-cancel");
                let handle = insert_session(local.take().unwrap());
                owned = Some(handle);
                let request = owned_session_call(handle, valid_owned_apply);
                assert!(request.validate_editor_transform_write());
                if target == LifecycleTargetChange::Schedule {
                    assert_eq!(
                        request.world_replacement_epoch, before_epoch,
                        "Schedule cancellation must reject a same-epoch legal write"
                    );
                }
                // Synchronize the Level replacement bookkeeping before the no-new-facts baseline.
                owned_session_call(handle, |session| {
                    session.level.drain_world_invalidations();
                });
                let observer_count = calls.load(Ordering::SeqCst);
                let diagnostic = RuntimeDynamicSessionError::SceneAssetReloadInterrupted {
                    source: cause.clone(),
                }
                .to_string();
                for _ in 0..3 {
                    let apply = unsafe {
                        ffi::handle_event(
                            handle,
                            ZrRuntimeEventV1::editor_transform_write(
                                ZIRCON_RUNTIME_ABI_VERSION_V1,
                                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                                &request,
                            ),
                        )
                    };
                    assert_owned_interrupted(apply, &diagnostic);
                    let wheel = unsafe {
                        ffi::handle_event(
                            handle,
                            ZrRuntimeEventV1::mouse_wheel(
                                ZIRCON_RUNTIME_ABI_VERSION_V1,
                                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                                1.0,
                            ),
                        )
                    };
                    assert_owned_interrupted(wheel, &diagnostic);
                    let (
                        result,
                        transform,
                        report,
                        first_cause,
                        retained_pending,
                        retained_staging,
                        retained_scheduled,
                        after_timing,
                        facts,
                    ) = owned_session_call(handle, |session| {
                        let result = session.tick_frame();
                        session.reset_frame_demand_after_failed_tick();
                        let queue = session.scene_asset_reload_queue.as_ref().unwrap();
                        (
                            result,
                            session
                                .level
                                .with_world(|world| world.local_transform(request.entity).unwrap()),
                            session.last_scene_asset_reload_report.clone(),
                            queue.lifecycle_interruption().cloned(),
                            queue.pending_count(),
                            queue.target_staging_count(),
                            queue.diagnostics().tasks_scheduled,
                            session.last_render_frame_timing,
                            session.level.drain_world_invalidations(),
                        )
                    });
                    assert_interrupted(result, target);
                    assert_eq!(transform, request.expected_transform());
                    assert_eq!(report.as_ref(), Some(&first_report));
                    assert_eq!(first_cause.as_ref(), Some(&cause));
                    assert_eq!(
                        (retained_pending, retained_staging, retained_scheduled),
                        (pending, staging, scheduled)
                    );
                    assert_eq!(after_timing, timing);
                    assert!(
                        facts.is_empty(),
                        "no post-cancel World facts may be emitted"
                    );
                    assert_eq!(calls.load(Ordering::SeqCst), observer_count);
                    assert_eq!(first_calls.load(Ordering::SeqCst), 1);
                    assert_eq!(stale_calls.load(Ordering::SeqCst), 0);
                    assert_eq!(
                        retained_camera_transform(&original_owner),
                        retained_transform
                    );
                }
                // Non-scene lifecycle cleanup and resize remain usable on the actual owned route.
                let mut lifecycle = ZrRuntimeEventV1::new(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
                    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                );
                lifecycle.state = ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1;
                let mut window = ZrRuntimeEventV1::new(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    ZR_RUNTIME_EVENT_KIND_WINDOW_STATUS_V1,
                    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                );
                window.state = ZR_RUNTIME_WINDOW_STATUS_DESTROYED_V1;
                for event in [
                    lifecycle,
                    window,
                    ZrRuntimeEventV1::viewport_resized(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                        ZrRuntimeViewportSizeV1::new(640, 480),
                    ),
                ] {
                    assert_eq!(
                        unsafe { ffi::handle_event(handle, event) }.status_code(),
                        ZrStatusCode::Ok
                    );
                }
                let mut wrong_viewport = ZrRuntimeEventV1::editor_transform_write(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                    &request,
                );
                wrong_viewport.viewport = ZrRuntimeViewportHandle::new(991);
                assert_eq!(
                    unsafe { ffi::handle_event(handle, wrong_viewport) }.status_code(),
                    ZrStatusCode::NotFound
                );
                let rejected_again = unsafe {
                    ffi::handle_event(
                        handle,
                        ZrRuntimeEventV1::editor_transform_write(
                            ZIRCON_RUNTIME_ABI_VERSION_V1,
                            ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                            &request,
                        ),
                    )
                };
                assert_owned_interrupted(rejected_again, &diagnostic);
                let (report, first_cause, transform, facts) =
                    owned_session_call(handle, |session| {
                        (
                            session.last_scene_asset_reload_report.clone(),
                            session.scene_asset_reload_interruption().cloned(),
                            session
                                .level
                                .with_world(|world| world.local_transform(request.entity).unwrap()),
                            session.level.drain_world_invalidations(),
                        )
                    });
                assert_eq!(report.as_ref(), Some(&first_report));
                assert_eq!(first_cause.as_ref(), Some(&cause));
                assert_eq!(transform, request.expected_transform());
                assert!(facts.is_empty());
                assert_eq!(calls.load(Ordering::SeqCst), observer_count);
            }));
            let cleanup = catch_unwind(AssertUnwindSafe(|| {
                dispose_owned_or_local(&mut local, owned)
            }));
            if let Err(payload) = outcome {
                resume_unwind(payload);
            }
            if let Err(payload) = cleanup {
                resume_unwind(payload);
            }
        }
    }

    #[test]
    fn owned_scene_writes_remain_live_after_ordinary_preflight_recovery() {
        use crate::dynamic_api::session::{ffi, registry::insert_session};
        use zircon_runtime_interface::{
            ZrRuntimeEventV1, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
        };
        let project = OwnedProjectFixture::new();
        let config = RuntimeProjectConfig::from_root(&project.root).unwrap();
        let mut local = Some(
            RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, Some(config))
                .unwrap(),
        );
        let mut owned = None;
        let calls = Arc::new(AtomicUsize::new(0));
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let session = local.as_mut().unwrap();
            assert!(session.runtime_ui.is_empty());
            prepare_three_fixture(session, &project, true);
            session.level.with_world_mut(|world| {
                world.spawn_node(NodeKind::Cube).unwrap();
            });
            session.level.drain_world_invalidations();
            session.tick_scene_asset_reload().unwrap();
            let rejected = session.last_scene_asset_reload_report.as_ref().unwrap();
            assert_eq!(rejected.failed_count(), 2);
            assert!(rejected.apply.failed.iter().all(|failure| matches!(
                failure.error(),
                DynamicSceneError::TargetWorldChanged { .. }
            )));
            assert!(session.scene_asset_reload_interruption().is_none());
            wait_until(|| {
                session
                    .scene_asset_reload_queue
                    .as_ref()
                    .unwrap()
                    .target_stages_ready()
            });
            session
                .tick_frame()
                .expect("the ordinary rejection must permit a later admitted commit");
            assert_eq!(
                session
                    .last_scene_asset_reload_report
                    .as_ref()
                    .unwrap()
                    .applied_count(),
                1
            );
            session.level.with_world_mut(|world| {
                register_transform_probe(world, &calls);
            });
            let handle = insert_session(local.take().unwrap());
            owned = Some(handle);
            let request = owned_session_call(handle, valid_owned_apply);
            assert!(request.validate_editor_transform_write());
            let apply = unsafe {
                ffi::handle_event(
                    handle,
                    ZrRuntimeEventV1::editor_transform_write(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                        &request,
                    ),
                )
            };
            assert_eq!(apply.status_code(), ZrStatusCode::Ok);
            let after_apply = owned_session_call(handle, |session| {
                session
                    .level
                    .with_world(|world| world.local_transform(request.entity).unwrap())
            });
            assert_eq!(after_apply, request.target_transform());
            let after_apply_observers = calls.load(Ordering::SeqCst);
            assert!(
                after_apply_observers > 0,
                "the native Insert observer must witness the legal Apply"
            );
            let wheel = unsafe {
                ffi::handle_event(
                    handle,
                    ZrRuntimeEventV1::mouse_wheel(
                        ZIRCON_RUNTIME_ABI_VERSION_V1,
                        ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                        1.0,
                    ),
                )
            };
            assert_eq!(wheel.status_code(), ZrStatusCode::Ok);
            let (after_wheel, cause, report) = owned_session_call(handle, |session| {
                (
                    session
                        .level
                        .with_world(|world| world.local_transform(request.entity).unwrap()),
                    session.scene_asset_reload_interruption().cloned(),
                    session.last_scene_asset_reload_report.clone(),
                )
            });
            assert_ne!(
                after_wheel, after_apply,
                "the nonzero wheel must really mutate the active camera without the terminal gate"
            );
            assert!(calls.load(Ordering::SeqCst) > after_apply_observers);
            assert!(cause.is_none());
            assert!(report.as_ref().unwrap().lifecycle_interruption().is_none());
        }));
        let cleanup = catch_unwind(AssertUnwindSafe(|| {
            dispose_owned_or_local(&mut local, owned)
        }));
        if let Err(payload) = outcome {
            resume_unwind(payload);
        }
        if let Err(payload) = cleanup {
            resume_unwind(payload);
        }
    }
}
