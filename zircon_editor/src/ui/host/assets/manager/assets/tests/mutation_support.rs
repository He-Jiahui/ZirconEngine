use std::fs;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use zircon_runtime::asset::pipeline::manager::{project_asset_manager_handle, ProjectAssetManager};
use zircon_runtime::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetImporterHandler, AssetKind, AssetManager, AssetModule, AssetUri, DataAsset,
    DataAssetFormat, FunctionAssetImporter, ImportedAsset, ImportedAssetEntry,
};
use zircon_runtime::core::manager::{resolve_manager_service, ManagerServiceHandle};
use zircon_runtime::core::{CoreRuntime, EngineTaskGraphOptions, TasksModule};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime_interface::project::RelPath;

use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::jobs::{
    EditorJob, EditorJobLimits, EditorJobSpec, EditorJobSystem, JobCategory, JobContext, JobError,
    MutexGroup,
};
use crate::core::logging::EditorLogService;
use crate::ui::host::runtime_services::EditorProjectAssetRuntimeAccess;

use super::DefaultEditorAssetManager;

pub(super) const SOURCE: &str = "res://data/item.u09data";
const WAIT: Duration = Duration::from_secs(30);

pub(super) fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

struct Fixture {
    parent: PathBuf,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let executable = fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
        let intended = executable
            .parent()
            .unwrap()
            .join("zircon-editor-u09-fixtures");
        fs::create_dir_all(&intended).unwrap();
        let parent = fs::canonicalize(&intended).unwrap();
        assert_eq!(parent, intended, "fixture parent must remain physical");
        for _ in 0..128 {
            let root = parent.join(format!(
                "u09-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&root) {
                Ok(()) => {
                    let fixture = Self { parent, root };
                    assert_eq!(fs::canonicalize(&fixture.root).unwrap(), fixture.root);
                    let paths = ProjectPaths::from_root(&fixture.root).unwrap();
                    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
                    ProjectManifest::new(
                        "Folder Operations",
                        uri("res://scenes/main.scene.toml"),
                        1,
                    )
                    .save(paths.manifest_path())
                    .unwrap();
                    let source = paths
                        .asset_root(&RelPath::project_assets())
                        .join("data/item.u09data");
                    fs::create_dir_all(source.parent().unwrap()).unwrap();
                    fs::write(source, "{\"version\":1}").unwrap();
                    return fixture;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create u09 fixture: {error}"),
            }
        }
        panic!("exclusive u09 fixture unavailable");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if self.root.parent() == Some(self.parent.as_path())
            && fs::canonicalize(&self.parent).ok().as_ref() == Some(&self.parent)
            && fs::canonicalize(&self.root).ok().as_ref() == Some(&self.root)
            && fs::symlink_metadata(&self.root)
                .is_ok_and(|metadata| !metadata.file_type().is_symlink())
        {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

#[derive(Debug)]
struct CountingImporter {
    delegate: FunctionAssetImporter,
    calls: Arc<AtomicUsize>,
}

impl AssetImporterHandler for CountingImporter {
    fn descriptor(&self) -> &AssetImporterDescriptor {
        self.delegate.descriptor()
    }
    fn import(&self, context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.delegate.import(context)
    }
}

fn importer(calls: Arc<AtomicUsize>) -> CountingImporter {
    fn import(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        let text = context.source_text()?;
        let canonical_json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|error| AssetImportError::Parse(error.to_string()))?;
        let with_subasset = canonical_json
            .get("with_subasset")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let mut outcome = AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::Data(DataAsset {
                uri: context.uri.clone(),
                format: DataAssetFormat::Json,
                text: text.clone(),
                canonical_json: canonical_json.clone(),
            }),
        );
        if with_subasset {
            let child = AssetUri::new(
                context.uri.scheme(),
                context.uri.path(),
                Some("detail".to_owned()),
            )?;
            outcome = outcome.with_entry(ImportedAssetEntry::new(
                child.clone(),
                ImportedAsset::Data(DataAsset {
                    uri: child,
                    format: DataAssetFormat::Json,
                    text,
                    canonical_json,
                }),
            ));
        }
        Ok(outcome)
    }
    CountingImporter {
        delegate: FunctionAssetImporter::new(
            AssetImporterDescriptor::new(
                "test.editor.u09.data",
                "test.editor.u09",
                AssetKind::Data,
                1,
            )
            .with_source_extensions(["u09data"]),
            import,
        ),
        calls,
    }
}

pub(super) struct TestAssets {
    pub(super) runtime: CoreRuntime,
    pub(super) jobs: EditorJobSystem,
    pub(super) handle: Option<ManagerServiceHandle<ProjectAssetManager>>,
    pub(super) editor: DefaultEditorAssetManager,
    pub(super) calls: Arc<AtomicUsize>,
    fixture: Option<Fixture>,
}

pub(super) struct QueueGate(Arc<(Mutex<bool>, Condvar)>);

impl Drop for QueueGate {
    fn drop(&mut self) {
        let (released, changed) = &*self.0;
        *released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = true;
        changed.notify_all();
    }
}

struct BlockMutationQueue {
    gate: Arc<(Mutex<bool>, Condvar)>,
    started: mpsc::Sender<()>,
    deadline: Instant,
}

impl EditorJob for BlockMutationQueue {
    type Output = ();
    fn run(self, context: JobContext) -> Result<(), JobError> {
        context.check_cancelled()?;
        self.started.send(()).unwrap();
        let (released, changed) = &*self.gate;
        let released = released
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let (released, _) = changed
            .wait_timeout_while(
                released,
                self.deadline.saturating_duration_since(Instant::now()),
                |released| !*released,
            )
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(*released, "u09 mutation queue blocker timed out");
        Ok(())
    }
}

impl TestAssets {
    pub(super) fn root(&self) -> &std::path::Path {
        &self.fixture.as_ref().unwrap().root
    }
    pub(super) fn paths(&self) -> ProjectPaths {
        ProjectPaths::from_root(self.root()).unwrap()
    }
    pub(super) fn assets_root(&self) -> PathBuf {
        self.paths().asset_root(&RelPath::project_assets())
    }
    pub(super) fn snapshot(&self) -> ProjectManager {
        let manager = resolve_manager_service(
            &self.runtime.handle(),
            self.handle.as_ref().unwrap().clone(),
        )
        .unwrap();
        AssetManager::current_project_snapshot(&*manager).unwrap()
    }
    pub(super) fn deadline(&self) -> Instant {
        Instant::now() + WAIT
    }
    pub(super) fn block_mutation_queue(&self) -> QueueGate {
        let gate = QueueGate(Arc::new((Mutex::new(false), Condvar::new())));
        let (started, observed) = mpsc::channel();
        self.jobs
            .submit(
                EditorJobSpec::new("block u09 mutation queue", JobCategory::InteractiveSave)
                    .with_mutex_group(MutexGroup::parse("project_asset_mutation").unwrap()),
                BlockMutationQueue {
                    gate: Arc::clone(&gate.0),
                    started,
                    deadline: self.deadline(),
                },
            )
            .unwrap();
        observed.recv_timeout(WAIT).unwrap();
        gate
    }
    pub(super) fn reopen(&self) {
        let manager = resolve_manager_service(
            &self.runtime.handle(),
            self.handle.as_ref().unwrap().clone(),
        )
        .unwrap();
        let original_activation = AssetManager::current_project_snapshot(&*manager)
            .unwrap()
            .activation_identity()
            .clone();
        manager
            .open_prepared_project(ProjectManager::open(self.root()).unwrap())
            .unwrap();
        assert_ne!(
            AssetManager::current_project_snapshot(&*manager)
                .unwrap()
                .activation_identity(),
            &original_activation
        );
        let deadline = self.deadline();
        while !manager.shutdown_project_watchers_until(deadline) {
            assert!(
                Instant::now() < deadline,
                "reopened project watchers did not join"
            );
            std::thread::yield_now();
        }
        drop(manager);
        self.editor.refresh_from_runtime_project().unwrap();
    }
}

pub(super) fn exercise(body: impl FnOnce(&TestAssets)) {
    exercise_with_watcher_policy(body, false);
}

pub(super) fn exercise_with_reopen(body: impl FnOnce(&TestAssets)) {
    exercise_with_watcher_policy(body, true);
}

fn exercise_with_watcher_policy(body: impl FnOnce(&TestAssets), allow_reopen: bool) {
    let fixture = Fixture::new();
    let runtime =
        CoreRuntime::try_with_task_graph_options(EngineTaskGraphOptions::with_worker_threads(3))
            .unwrap();
    let jobs = EditorJobSystem::with_scheduler_and_bus(
        runtime.scheduler().clone(),
        SharedEditorMessageBus::default(),
        EditorJobLimits::default(),
    );
    let mut scope = TestAssets {
        runtime,
        jobs,
        handle: None,
        editor: DefaultEditorAssetManager::new(),
        calls: Arc::new(AtomicUsize::new(0)),
        fixture: Some(fixture),
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
            .activate_registered_modules_with_ready_timeout(WAIT)
            .unwrap();
        let handle = project_asset_manager_handle(&scope.runtime.handle()).unwrap();
        scope.handle = Some(handle.clone());
        {
            let manager = resolve_manager_service(&scope.runtime.handle(), handle).unwrap();
            manager
                .register_asset_importer(importer(Arc::clone(&scope.calls)))
                .unwrap();
            AssetManager::open_project(&*manager, scope.root().to_str().unwrap()).unwrap();
            if !allow_reopen {
                let deadline = scope.deadline();
                while !manager.shutdown_project_watchers_until(deadline) {
                    assert!(Instant::now() < deadline, "project watchers did not join");
                    std::thread::yield_now();
                }
            }
        }
        scope.editor = DefaultEditorAssetManager::with_runtime_project_manager(
            EditorProjectAssetRuntimeAccess::new(&scope.runtime.handle()).unwrap(),
            scope.jobs.clone(),
            Arc::new(EditorLogService::default()),
        );
        scope.editor.refresh_from_runtime_project().unwrap();
        body(&scope);
    }));
    let deadline = scope.deadline();
    let unfinished = scope.jobs.shutdown(deadline);
    let closed = scope.runtime.shutdown_until(deadline);
    let quiescent = unfinished.is_empty()
        && closed
            .as_ref()
            .is_ok_and(|report| !report.has_in_flight_work());
    if !quiescent {
        if let Some(fixture) = scope.fixture.take() {
            std::mem::forget(fixture);
        }
        eprintln!("u09 fixture retained: unfinished={unfinished:?}, close={closed:?}");
    }
    if let Err(payload) = outcome {
        resume_unwind(payload);
    }
    assert!(
        quiescent,
        "runtime owners must settle before u09 fixture cleanup"
    );
}
