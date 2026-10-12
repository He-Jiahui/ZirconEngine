//! Real registered Runtime imports through the normal project-bound Editor flow.
use super::*;
use crate::core::jobs::{EditorJob, EditorJobSpec, JobContext, JobError};
use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::asset::AssetImporterHandler;

#[derive(Clone)]
struct ImportStop {
    path: PathBuf,
    gate: Arc<PostCommitGate>,
    started: mpsc::Sender<()>,
    deadline: Instant,
}

struct GatedImporter {
    delegate: FunctionAssetImporter,
    stop: Arc<Mutex<Option<ImportStop>>>,
    calls: Arc<AtomicUsize>,
}

impl std::fmt::Debug for GatedImporter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GatedImporter")
            .field("descriptor", self.delegate.descriptor())
            .finish_non_exhaustive()
    }
}

impl AssetImporterHandler for GatedImporter {
    fn descriptor(&self) -> &AssetImporterDescriptor {
        self.delegate.descriptor()
    }

    fn import(&self, context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let stop = {
            let mut pending = self.stop.lock().unwrap_or_else(|p| p.into_inner());
            if pending
                .as_ref()
                .is_some_and(|stop| stop.path == context.source_path)
            {
                pending.take()
            } else {
                None
            }
        };
        if let Some(stop) = stop {
            stop.started.send(()).unwrap();
            assert!(
                stop.gate.wait_until(stop.deadline),
                "real importer gate timed out"
            );
        }
        self.delegate.import(context)
    }
}

struct QueueBlocker {
    gate: Arc<PostCommitGate>,
    started: mpsc::Sender<()>,
    deadline: Instant,
}

impl EditorJob for QueueBlocker {
    type Output = ();

    fn run(self, context: JobContext) -> Result<(), JobError> {
        context.check_cancelled()?;
        self.started.send(()).unwrap();
        assert!(
            self.gate.wait_until(self.deadline),
            "queue blocker timed out"
        );
        Ok(())
    }
}

struct RegisteredScope {
    runtime: CoreRuntime,
    jobs: EditorJobSystem,
    handle: Option<ManagerServiceHandle<ProjectAssetManager>>,
    deadline: Instant,
    stop: Arc<Mutex<Option<ImportStop>>>,
    calls: Arc<AtomicUsize>,
    gates: Vec<Arc<PostCommitGate>>,
    fixtures: Vec<ProjectFixture>,
}

impl RegisteredScope {
    fn original(&self) -> ManagerServiceHandle<ProjectAssetManager> {
        self.handle.as_ref().unwrap().clone()
    }

    fn fixture(&mut self) -> usize {
        self.fixtures.push(ProjectFixture::new());
        self.fixtures.len() - 1
    }

    fn open(&self, fixture: usize) -> ProjectManager {
        let manager = resolve_manager_service(&self.runtime.handle(), self.original()).unwrap();
        AssetManager::open_project(&*manager, self.fixtures[fixture].root.to_str().unwrap())
            .unwrap();
        while !manager.shutdown_project_watchers_until(self.deadline) {
            assert!(Instant::now() < self.deadline, "real watchers did not join");
            std::thread::yield_now();
        }
        AssetManager::current_project_snapshot(&*manager).unwrap()
    }

    fn snapshot(&self) -> ProjectManager {
        let manager = resolve_manager_service(&self.runtime.handle(), self.original()).unwrap();
        AssetManager::current_project_snapshot(&*manager).unwrap()
    }

    fn flow(&self, index: &Arc<Mutex<EditorAssetIndex>>) -> EditorAssetImportFlow {
        EditorAssetImportFlow::new(
            self.jobs.clone(),
            self.runtime.weak(),
            self.original(),
            Arc::clone(index),
            Arc::new(EditorLogService::default()),
        )
        .unwrap()
    }

    fn gate(&mut self) -> Arc<PostCommitGate> {
        let gate = Arc::new(PostCommitGate::default());
        self.gates.push(Arc::clone(&gate));
        gate
    }

    fn block_queue(&mut self) -> Arc<PostCommitGate> {
        let gate = self.gate();
        let (started, observed) = mpsc::channel();
        let _ticket = self
            .jobs
            .submit(
                EditorJobSpec::new("block real import queue", JobCategory::Import),
                QueueBlocker {
                    gate: Arc::clone(&gate),
                    started,
                    deadline: self.deadline,
                },
            )
            .unwrap();
        observed
            .recv_timeout(self.deadline.saturating_duration_since(Instant::now()))
            .unwrap();
        gate
    }

    fn block_import(&mut self, fixture: usize) -> (Arc<PostCommitGate>, mpsc::Receiver<()>) {
        let gate = self.gate();
        let (started, observed) = mpsc::channel();
        *self.stop.lock().unwrap_or_else(|p| p.into_inner()) = Some(ImportStop {
            path: self.fixtures[fixture].source(),
            gate: Arc::clone(&gate),
            started,
            deadline: self.deadline,
        });
        (gate, observed)
    }
}

fn exercise(body: impl FnOnce(&mut RegisteredScope)) {
    let runtime =
        CoreRuntime::try_with_task_graph_options(EngineTaskGraphOptions::with_worker_threads(3))
            .unwrap();
    let jobs = EditorJobSystem::with_scheduler_and_bus(
        runtime.scheduler().clone(),
        SharedEditorMessageBus::default(),
        EditorJobLimits::default().with_limit(JobCategory::Import, 1),
    );
    let mut scope = RegisteredScope {
        runtime,
        jobs,
        handle: None,
        deadline: Instant::now() + BLOCK_LIMIT,
        stop: Arc::new(Mutex::new(None)),
        calls: Arc::new(AtomicUsize::new(0)),
        gates: Vec::new(),
        fixtures: Vec::new(),
    };
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        scope
            .runtime
            .register_module(TasksModule.descriptor())
            .unwrap();
        scope
            .runtime
            .register_module(AssetModule::default().descriptor())
            .unwrap();
        scope
            .runtime
            .activate_registered_modules_with_ready_timeout(
                scope.deadline.saturating_duration_since(Instant::now()),
            )
            .unwrap();
        let handle = project_asset_manager_handle(&scope.runtime.handle()).unwrap();
        scope.handle = Some(handle.clone());
        resolve_manager_service(&scope.runtime.handle(), handle)
            .unwrap()
            .register_asset_importer(GatedImporter {
                delegate: data_importer(),
                stop: Arc::clone(&scope.stop),
                calls: Arc::clone(&scope.calls),
            })
            .unwrap();
        body(&mut scope);
    }));
    // The original gates are released before waiting for the actual job and worker owners.
    for gate in &scope.gates {
        gate.release();
    }
    let unfinished = scope.jobs.shutdown(scope.deadline);
    let closed = scope.runtime.shutdown_until(scope.deadline);
    let quiescent = unfinished.is_empty()
        && closed
            .as_ref()
            .is_ok_and(|report| !report.has_in_flight_work());
    if !quiescent {
        // Unknown worker completion must never permit fixture deletion on this failure exit.
        for fixture in std::mem::take(&mut scope.fixtures) {
            std::mem::forget(fixture);
        }
        eprintln!("registered fixture retained: unfinished={unfinished:?}, close={closed:?}");
    }
    if let Err(payload) = outcome {
        resume_unwind(payload);
    }
    assert!(
        quiescent,
        "actual Runtime owners must settle before fixture Drop"
    );
}

fn index(project: &ProjectManager) -> Arc<Mutex<EditorAssetIndex>> {
    Arc::new(Mutex::new(
        EditorAssetIndex::from_runtime_project(project).unwrap(),
    ))
}

fn replace_index(index: &Arc<Mutex<EditorAssetIndex>>, project: &ProjectManager) {
    index
        .lock()
        .unwrap()
        .replace_authoritative_projection(EditorAssetIndex::from_runtime_project(project).unwrap());
}

fn request() -> EditorAssetImportRequest {
    EditorAssetImportRequest::new(
        AssetUri::parse(TARGET).unwrap(),
        EditorAssetImportReason::Manual,
    )
}

fn assert_inactive(error: &JobError, service_name: &str) {
    match error.downcast_ref::<CoreError>() {
        Some(CoreError::Initialization(owner, message)) => {
            assert_eq!(owner, service_name);
            assert_eq!(
                message,
                "asset import belongs to an inactive project activation"
            );
        }
        other => panic!("expected original typed activation failure, got {other:?}: {error:?}"),
    }
}

#[test]
fn queued_import_rejects_switched_project_and_preserves_new_scope_lease() {
    exercise(|scope| {
        let a = scope.fixture();
        let b = scope.fixture();
        let project_a = scope.open(a);
        let index = index(&project_a);
        let flow_a = scope.flow(&index);
        let blocker = scope.block_queue();
        let old = flow_a.submit(request()).unwrap();
        let old_shared = flow_a.submit(request()).unwrap();
        let old_model = flow_a
            .submit_model_source(scope.fixtures[a].root.join("not-opened.gltf"))
            .unwrap();
        assert_eq!(old.id(), old_shared.id());
        // Deliberately make URI, UUID and source digest equal across the two real roots.
        fs::copy(scope.fixtures[a].meta(), scope.fixtures[b].meta()).unwrap();
        let project_b = scope.open(b);
        let target = AssetUri::parse(TARGET).unwrap();
        assert_eq!(
            project_a
                .asset_registry()
                .entry_by_path(&target)
                .unwrap()
                .uuid(),
            project_b
                .asset_registry()
                .entry_by_path(&target)
                .unwrap()
                .uuid()
        );
        assert_ne!(
            project_a.activation_identity(),
            project_b.activation_identity()
        );
        replace_index(&index, &project_b);
        let flow_b = scope.flow(&index);
        fs::write(scope.fixtures[b].source(), AFTER).unwrap();
        let before_meta = fs::read(scope.fixtures[b].meta()).unwrap();
        let registry_path = scope.fixtures[b]
            .paths()
            .registry_root()
            .join("asset-registry.json");
        let before_registry = fs::read(&registry_path).unwrap();
        let before_catalog = project_b.catalog_input_generation().sequence();
        let before_resource = resolve_manager_service(&scope.runtime.handle(), scope.original())
            .unwrap()
            .resource_manager()
            .registry()
            .get_by_locator(&target)
            .unwrap()
            .revision;
        let before_calls = scope.calls.load(Ordering::SeqCst);
        let (b_gate, b_started) = scope.block_import(b);
        let new = flow_b.submit(request()).unwrap();
        assert_ne!(old.id(), new.id());
        blocker.release();
        b_started
            .recv_timeout(scope.deadline.saturating_duration_since(Instant::now()))
            .unwrap();
        let old_error = old.wait_until(scope.deadline).unwrap().unwrap_err();
        assert_inactive(&old_error, scope.original().service_name().as_str());
        assert_eq!(
            old_error,
            old_shared.wait_until(scope.deadline).unwrap().unwrap_err()
        );
        assert_inactive(
            &old_model.wait_until(scope.deadline).unwrap().unwrap_err(),
            scope.original().service_name().as_str(),
        );
        assert_eq!(scope.calls.load(Ordering::SeqCst), before_calls + 1);
        assert_eq!(
            import_state(&index, &target),
            EditorAssetImportState::Importing
        );
        assert_eq!(fs::read(scope.fixtures[b].meta()).unwrap(), before_meta);
        assert_eq!(fs::read(&registry_path).unwrap(), before_registry);
        assert_eq!(
            scope.snapshot().catalog_input_generation().sequence(),
            before_catalog
        );
        assert_eq!(
            resolve_manager_service(&scope.runtime.handle(), scope.original())
                .unwrap()
                .resource_manager()
                .registry()
                .get_by_locator(&target)
                .unwrap()
                .revision,
            before_resource
        );
        b_gate.release();
        assert!(new.wait_until(scope.deadline).unwrap().is_ok());
        assert_ne!(
            import_state(&index, &target),
            EditorAssetImportState::Importing
        );
        // The original flow remains closed to the new index instead of following latest PAM.
        assert!(matches!(
            flow_a.submit(request()),
            Err(
                super::super::super::EditorAssetImportSubmitError::ProjectActivationSuperseded { .. }
            )
        ));
    });
}

#[test]
fn same_activation_catalog_update_keeps_flow_valid() {
    exercise(|scope| {
        let a = scope.fixture();
        let project_a = scope.open(a);
        let index = index(&project_a);
        let flow = scope.flow(&index);
        let token = resolve_manager_service(&scope.runtime.handle(), scope.original())
            .unwrap()
            .current_project_generation_snapshot()
            .unwrap()
            .into_parts()
            .1;
        fs::write(scope.fixtures[a].source(), AFTER).unwrap();
        let manager = resolve_manager_service(&scope.runtime.handle(), scope.original()).unwrap();
        assert!(manager
            .import_asset_for_project(project_a.activation_identity(), TARGET)
            .unwrap()
            .is_some());
        let updated = AssetManager::current_project_snapshot(&*manager).unwrap();
        assert_eq!(
            updated.activation_identity(),
            project_a.activation_identity()
        );
        assert_ne!(
            updated.catalog_input_generation().sequence(),
            project_a.catalog_input_generation().sequence()
        );
        assert!(matches!(
            manager.check_project_generation(&token),
            zircon_runtime::asset::pipeline::manager::ProjectGenerationMatch::Superseded {
                newer_same_project_generation: true,
            }
        ));
        drop(manager);
        replace_index(&index, &updated);
        fs::write(scope.fixtures[a].source(), "{\"version\":3}").unwrap();
        let imported = flow
            .submit(request())
            .unwrap()
            .wait_until(scope.deadline)
            .unwrap()
            .unwrap();
        assert!(imported.status().imported);
        assert_eq!(
            scope.snapshot().activation_identity(),
            project_a.activation_identity()
        );
    });
}

#[test]
fn same_root_reopen_replaces_scope_with_equal_source_identity() {
    exercise(|scope| {
        let a = scope.fixture();
        let first = scope.open(a);
        let index = index(&first);
        let old_flow = scope.flow(&index);
        let blocker = scope.block_queue();
        let old = old_flow.submit(request()).unwrap();
        resolve_manager_service(&scope.runtime.handle(), scope.original())
            .unwrap()
            .close_project()
            .unwrap();
        let reopened = scope.open(a);
        assert_ne!(first.activation_identity(), reopened.activation_identity());
        let target = AssetUri::parse(TARGET).unwrap();
        let first_entry = first.asset_registry().entry_by_path(&target).unwrap();
        let reopened_entry = reopened.asset_registry().entry_by_path(&target).unwrap();
        assert_eq!(first_entry.uuid(), reopened_entry.uuid());
        assert_eq!(first_entry.source_digest(), reopened_entry.source_digest());
        replace_index(&index, &reopened);
        assert_ne!(
            import_state(&index, &AssetUri::parse(TARGET).unwrap()),
            EditorAssetImportState::Importing
        );
        assert!(matches!(
            old_flow.submit(request()),
            Err(
                super::super::super::EditorAssetImportSubmitError::ProjectActivationSuperseded { .. }
            )
        ));
        blocker.release();
        assert_inactive(
            &old.wait_until(scope.deadline).unwrap().unwrap_err(),
            scope.original().service_name().as_str(),
        );
        fs::write(scope.fixtures[a].source(), AFTER).unwrap();
        assert!(scope
            .flow(&index)
            .submit(request())
            .unwrap()
            .wait_until(scope.deadline)
            .unwrap()
            .is_ok());
    });
}

#[test]
fn failed_unpublished_activation_leaves_original_flow_usable() {
    exercise(|scope| {
        let a = scope.fixture();
        let b = scope.fixture();
        let original = scope.open(a);
        let index = index(&original);
        let flow = scope.flow(&index);
        let blocker = scope.block_queue();
        let queued = flow.submit(request()).unwrap();
        let before_calls = scope.calls.load(Ordering::SeqCst);
        let prepared_b = ProjectManager::open(&scope.fixtures[b].root).unwrap();
        // A real malformed metadata document fails full-generation preparation after admission.
        fs::write(scope.fixtures[b].meta(), "not valid metadata = [").unwrap();
        let manager = resolve_manager_service(&scope.runtime.handle(), scope.original()).unwrap();
        assert!(AssetManager::open_prepared_project(&*manager, prepared_b).is_err());
        assert!(manager.is_project_activation_current(original.activation_identity()));
        assert_eq!(
            AssetManager::current_project_snapshot(&*manager)
                .unwrap()
                .paths()
                .root(),
            scope.fixtures[a].root.as_path()
        );
        drop(manager);
        fs::write(scope.fixtures[a].source(), AFTER).unwrap();
        blocker.release();
        assert!(queued.wait_until(scope.deadline).unwrap().is_ok());
        assert_eq!(scope.calls.load(Ordering::SeqCst), before_calls + 1);
        assert_eq!(
            scope.snapshot().activation_identity(),
            original.activation_identity()
        );
    });
}

#[test]
fn project_switch_during_preparation_never_publishes_old_candidate() {
    exercise(|scope| {
        let a = scope.fixture();
        let b = scope.fixture();
        let original = scope.open(a);
        let index = index(&original);
        let flow = scope.flow(&index);
        fs::write(scope.fixtures[a].source(), AFTER).unwrap();
        let (a_gate, a_started) = scope.block_import(a);
        let queued = flow.submit(request()).unwrap();
        a_started
            .recv_timeout(scope.deadline.saturating_duration_since(Instant::now()))
            .unwrap();
        let next = scope.open(b);
        replace_index(&index, &next);
        let before_meta = fs::read(scope.fixtures[b].meta()).unwrap();
        let registry_path = scope.fixtures[b]
            .paths()
            .registry_root()
            .join("asset-registry.json");
        let before_registry = fs::read(&registry_path).unwrap();
        let before_sequence = next.catalog_input_generation().sequence();
        let target = AssetUri::parse(TARGET).unwrap();
        let before_revision = resolve_manager_service(&scope.runtime.handle(), scope.original())
            .unwrap()
            .resource_manager()
            .registry()
            .get_by_locator(&target)
            .unwrap()
            .revision;
        a_gate.release();
        assert_inactive(
            &queued.wait_until(scope.deadline).unwrap().unwrap_err(),
            scope.original().service_name().as_str(),
        );
        assert_eq!(fs::read(scope.fixtures[b].meta()).unwrap(), before_meta);
        assert_eq!(fs::read(&registry_path).unwrap(), before_registry);
        assert_eq!(
            scope.snapshot().activation_identity(),
            next.activation_identity()
        );
        assert_eq!(
            scope.snapshot().catalog_input_generation().sequence(),
            before_sequence
        );
        assert_eq!(
            resolve_manager_service(&scope.runtime.handle(), scope.original())
                .unwrap()
                .resource_manager()
                .registry()
                .get_by_locator(&target)
                .unwrap()
                .revision,
            before_revision
        );
    });
}
