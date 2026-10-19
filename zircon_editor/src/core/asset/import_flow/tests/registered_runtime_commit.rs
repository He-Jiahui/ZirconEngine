//! Exercises a real registered Runtime transaction and the Editor's shared-flight projection.
use std::collections::BTreeSet;
use std::fs;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use zircon_runtime::asset::pipeline::manager::{project_asset_manager_handle, ProjectAssetManager};
use zircon_runtime::asset::project::{AssetMetaDocument, ProjectManifest, ProjectPaths};
use zircon_runtime::asset::registry::AssetRegistryEntry;
use zircon_runtime::asset::watch::AssetChangeKind;
use zircon_runtime::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor,
    AssetKind, AssetManager, AssetModule, AssetStatusRecord, AssetUri, DataAsset, DataAssetFormat,
    FunctionAssetImporter, ImportedAsset, ASSET_MODULE_NAME,
};
use zircon_runtime::core::manager::{resolve_manager_service, ManagerServiceHandle};
use zircon_runtime::core::resource::ResourceState;
use zircon_runtime::core::{CoreError, CoreRuntime, CoreWeak, EngineTaskGraphOptions, TasksModule};
use zircon_runtime::engine_module::EngineModule;
use zircon_runtime_interface::project::RelPath;

use crate::core::asset::{EditorAssetImportState, EditorAssetIndex};
use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, SharedEditorMessageBus, TOPIC_JOB,
};
use crate::core::jobs::{
    EditorJobLimits, EditorJobSystem, JobCategory, JobEventKind, JobEventPumpBudget,
};
use crate::core::logging::{EditorLogService, LogChannel, LogFilter, LogSeverity};

use super::{
    import_state, AssetImportBackend, EditorAssetImportFlow, EditorAssetImportReason,
    EditorAssetImportRequest,
};

const WAIT: Duration = Duration::from_secs(5);
const BLOCK_LIMIT: Duration = Duration::from_secs(30);
const TARGET: &str = "res://data/committed.zrcommit";
const BEFORE: &str = "{\"version\":1}";
const AFTER: &str = "{\"version\":2}";

#[derive(Default)]
struct PostCommitGate {
    released: Mutex<bool>,
    changed: Condvar,
}

impl PostCommitGate {
    fn release(&self) {
        *self.released.lock().unwrap_or_else(|p| p.into_inner()) = true;
        self.changed.notify_all();
    }

    fn wait_until(&self, deadline: Instant) -> bool {
        let mut released = self.released.lock().unwrap_or_else(|p| p.into_inner());
        while !*released {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            (released, _) = self
                .changed
                .wait_timeout(released, remaining)
                .unwrap_or_else(|p| p.into_inner());
        }
        true
    }
}

struct ReleaseOnDrop(Arc<PostCommitGate>);
impl Drop for ReleaseOnDrop {
    fn drop(&mut self) {
        self.0.release();
    }
}

struct RegisteredBackend {
    core: CoreWeak,
    handle: ManagerServiceHandle<ProjectAssetManager>,
    committed: mpsc::Sender<AssetStatusRecord>,
    gate: Arc<PostCommitGate>,
    calls: AtomicUsize,
}

impl AssetImportBackend for RegisteredBackend {
    fn import(&self, uri: &AssetUri) -> Result<Option<AssetStatusRecord>, CoreError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let core = self
            .core
            .upgrade()
            .ok_or_else(|| CoreError::ServiceUnavailable(self.handle.service_name().to_string()))?;
        let manager = resolve_manager_service(&core, self.handle.clone())?;
        let original = AssetManager::import_asset(&*manager, &uri.to_string());
        if let Ok(Some(status)) = &original {
            // The signal is after the real Runtime transaction, not an importer callback.
            let _ = self.committed.send(status.clone());
            assert!(
                self.gate.wait_until(Instant::now() + BLOCK_LIMIT),
                "postcommit test gate timed out"
            );
        }
        // Preserve original None and typed Core errors; the normal runner catches panics.
        original
    }
}

pub(super) struct ProjectFixture {
    parent: PathBuf,
    pub(super) root: PathBuf,
}

impl ProjectFixture {
    pub(super) fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let executable = fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
        let executable_parent = executable.parent().unwrap();
        let intended_parent = executable_parent.join("zircon-editor-import-fixtures");
        fs::create_dir_all(&intended_parent).unwrap();
        let parent = fs::canonicalize(&intended_parent).unwrap();
        assert_eq!(
            parent, intended_parent,
            "fixture parent must remain under the physical executable directory"
        );
        for _ in 0..128 {
            let root = parent.join(format!(
                "postcommit-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&root) {
                Ok(()) => {
                    let fixture = Self { parent, root };
                    assert!(fixture.root.is_absolute());
                    assert_eq!(fs::canonicalize(&fixture.root).unwrap(), fixture.root);
                    let paths = fixture.paths();
                    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
                    ProjectManifest::new(
                        "Registered Import Commit",
                        AssetUri::parse(TARGET).unwrap(),
                        1,
                    )
                    .save(paths.manifest_path())
                    .unwrap();
                    let source = fixture.source();
                    fs::create_dir_all(source.parent().unwrap()).unwrap();
                    fs::write(source, BEFORE).unwrap();
                    return fixture;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create exclusive fixture: {error}"),
            }
        }
        panic!("no exclusive fixture directory available");
    }

    pub(super) fn open_project(&self) -> zircon_runtime::asset::project::ProjectManager {
        zircon_runtime::asset::project::ProjectManager::open(&self.root).unwrap()
    }

    fn paths(&self) -> ProjectPaths {
        ProjectPaths::from_root(&self.root).unwrap()
    }

    fn source(&self) -> PathBuf {
        self.paths()
            .asset_root(&RelPath::project_assets())
            .join("data/committed.zrcommit")
    }

    fn meta(&self) -> PathBuf {
        self.source().with_file_name("committed.zrcommit.zmeta")
    }
}

impl Drop for ProjectFixture {
    fn drop(&mut self) {
        let own_physical_child = self.root.is_absolute()
            && self.root.parent() == Some(self.parent.as_path())
            && fs::canonicalize(&self.parent).ok().as_ref() == Some(&self.parent)
            && fs::canonicalize(&self.root).ok().as_ref() == Some(&self.root)
            && fs::symlink_metadata(&self.root)
                .is_ok_and(|metadata| !metadata.file_type().is_symlink());
        if own_physical_child {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn data_importer() -> FunctionAssetImporter {
    fn import(context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        let text = context.source_text()?;
        let canonical_json = serde_json::from_str(&text)
            .map_err(|error| AssetImportError::Parse(error.to_string()))?;
        Ok(AssetImportOutcome::new(
            context.uri.clone(),
            ImportedAsset::Data(DataAsset {
                uri: context.uri.clone(),
                format: DataAssetFormat::Json,
                text,
                canonical_json,
            }),
        ))
    }
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new(
            "test.editor.registered.commit.data",
            "test.editor.registered.commit",
            AssetKind::Data,
            1,
        )
        .with_source_extensions(["zrcommit"]),
        import,
    )
}

#[test]
fn late_cancel_preserves_registered_runtime_commit_and_shared_settlement() {
    let fixture = ProjectFixture::new();
    let runtime =
        CoreRuntime::try_with_task_graph_options(EngineTaskGraphOptions::with_worker_threads(3))
            .unwrap();
    let bus = SharedEditorMessageBus::default();
    let subscriber = bus
        .register_subscriber([EditorTopic::parse(TOPIC_JOB).unwrap()])
        .unwrap();
    let jobs = EditorJobSystem::with_scheduler_and_bus(
        runtime.scheduler().clone(),
        bus.clone(),
        EditorJobLimits::default().with_limit(JobCategory::Import, 1),
    );
    let gate = Arc::new(PostCommitGate::default());
    let _release = ReleaseOnDrop(Arc::clone(&gate));
    let mut cleanup_handle: Option<ManagerServiceHandle<ProjectAssetManager>> = None;

    let outcome = catch_unwind(AssertUnwindSafe(|| {
        runtime.register_module(TasksModule.descriptor()).unwrap();
        runtime
            .register_module(AssetModule::default().descriptor())
            .unwrap();
        runtime
            .activate_registered_modules_with_ready_timeout(WAIT)
            .unwrap();
        let handle = project_asset_manager_handle(&runtime.handle()).unwrap();
        cleanup_handle = Some(handle.clone());
        let target = AssetUri::parse(TARGET).unwrap();
        let (index, before_status, before_sequence, before_revision, changes) = {
            let manager = resolve_manager_service(&runtime.handle(), handle.clone()).unwrap();
            manager.register_asset_importer(data_importer()).unwrap();
            AssetManager::open_project(&*manager, fixture.root.to_str().unwrap()).unwrap();
            // Joining real watchers prevents an automatic transaction after our write.
            let deadline = Instant::now() + WAIT;
            while !manager.shutdown_project_watchers_until(deadline) {
                assert!(Instant::now() < deadline, "project watchers did not join");
                std::thread::sleep(Duration::from_millis(5));
            }
            let project = AssetManager::current_project_snapshot(&*manager).unwrap();
            let before_status = AssetManager::asset_status(&*manager, TARGET).unwrap();
            assert!(before_status.imported);
            let before_revision = manager
                .resource_manager()
                .registry()
                .get_by_locator(&target)
                .unwrap()
                .revision;
            (
                Arc::new(Mutex::new(
                    EditorAssetIndex::from_runtime_project(&project).unwrap(),
                )),
                before_status,
                project.catalog_input_generation().sequence(),
                before_revision,
                AssetManager::subscribe_asset_changes(&*manager),
            )
        };
        // This projection remains the original catalog; completion only clears its lease.
        let before_index_state = import_state(&index, &target);
        assert_ne!(before_index_state, EditorAssetImportState::Importing);
        let before_meta = fs::read(fixture.meta()).unwrap();
        let registry_path = fixture.paths().registry_root().join("asset-registry.json");
        let before_registry = fs::read(&registry_path).unwrap();
        fs::write(fixture.source(), AFTER).unwrap();

        let (committed_tx, committed_rx) = mpsc::channel();
        let backend = Arc::new(RegisteredBackend {
            core: runtime.weak(),
            handle: handle.clone(),
            committed: committed_tx,
            gate: Arc::clone(&gate),
            calls: AtomicUsize::new(0),
        });
        let logs = Arc::new(EditorLogService::default());
        let flow = EditorAssetImportFlow::with_backend_and_logs(
            jobs.clone(),
            Arc::clone(&backend),
            Arc::clone(&index),
            Arc::clone(&logs),
        );
        let request =
            EditorAssetImportRequest::new(target.clone(), EditorAssetImportReason::Manual);
        let first = flow.submit(request.clone()).unwrap();
        let original = committed_rx
            .recv_timeout(WAIT)
            .expect("real Runtime transaction must commit before cancellation");
        assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
        assert!(first.try_result().is_none());
        assert!(!jobs.is_terminal_record(first.id()));
        assert!(original.imported);
        assert_eq!(original.id, before_status.id);
        assert_ne!(original.source_hash, before_status.source_hash);
        assert_eq!(
            import_state(&index, &target),
            EditorAssetImportState::Importing
        );

        let committed_sequence = {
            let manager = resolve_manager_service(&runtime.handle(), handle.clone()).unwrap();
            assert_eq!(
                AssetManager::asset_status(&*manager, TARGET).unwrap(),
                original
            );
            let project = AssetManager::current_project_snapshot(&*manager).unwrap();
            let sequence = project.catalog_input_generation().sequence();
            assert!(sequence > before_sequence);
            let resources = manager.resource_manager();
            let registry = resources.registry();
            let record = registry.get_by_locator(&target).unwrap();
            assert_eq!(record.state, ResourceState::Ready);
            assert!(record.revision > before_revision);
            let metadata = project.registry().get_by_locator(&target).unwrap();
            let artifact_uri = metadata.artifact_locator().unwrap();
            assert_eq!(Some(artifact_uri.to_string()), original.artifact_uri);
            assert!(project
                .paths()
                .asset_artifact_root()
                .join(artifact_uri.path())
                .is_file());
            let ImportedAsset::Data(data) = project.load_artifact(&target).unwrap() else {
                panic!("committed artifact must be Data");
            };
            assert_eq!(data.text, AFTER);
            assert_eq!(data.canonical_json, serde_json::json!({"version": 2}));
            let meta = AssetMetaDocument::load(fixture.meta()).unwrap();
            assert_eq!(meta.url, target);
            assert_eq!(meta.source_digest, original.source_hash);
            assert_eq!(AssetId::from_asset_uuid(meta.uuid).to_string(), original.id);
            assert_eq!(meta.importer_id, original.importer_id);
            assert_eq!(
                meta.artifact_locator.as_ref().map(ToString::to_string),
                original.artifact_uri
            );
            let entry = project.asset_registry().entry_by_path(&target).unwrap();
            assert_eq!(entry.uuid(), meta.uuid);
            assert_eq!(entry.source_digest(), original.source_hash);
            sequence
        };
        assert_ne!(fs::read(fixture.meta()).unwrap(), before_meta);
        let disk_registry = fs::read(&registry_path).unwrap();
        assert_ne!(disk_registry, before_registry);
        let document: serde_json::Value = serde_json::from_slice(&disk_registry).unwrap();
        assert_eq!(document["format_version"], 1);
        let disk_entries: Vec<AssetRegistryEntry> =
            serde_json::from_value(document["entries"].clone()).unwrap();
        let disk_entry = disk_entries
            .iter()
            .find(|entry| entry.path() == &target)
            .unwrap();
        assert_eq!(
            AssetId::from_asset_uuid(disk_entry.uuid()).to_string(),
            original.id
        );
        assert_eq!(disk_entry.source_digest(), original.source_hash);
        let change = changes
            .try_recv()
            .expect("real targeted transaction publishes Modified");
        assert_eq!(change.kind, AssetChangeKind::Modified);
        assert_eq!(change.uri, target);
        assert_eq!(change.previous_uri, None);
        assert!(changes.try_recv().is_err());

        let second = flow
            .submit(EditorAssetImportRequest::new(
                target.clone(),
                EditorAssetImportReason::DigestMismatch,
            ))
            .unwrap();
        assert_eq!(first.id(), second.id());
        assert!(jobs.cancel(first.id()));
        assert_eq!(
            import_state(&index, &target),
            EditorAssetImportState::Importing
        );
        gate.release();
        for ticket in [&first, &second] {
            let result = ticket
                .wait_until(Instant::now() + WAIT)
                .expect("released flight settles")
                .unwrap();
            assert_eq!(result.status(), &original);
            assert_eq!(result.uri(), &target);
            assert_eq!(
                result.reasons(),
                vec![
                    EditorAssetImportReason::DigestMismatch,
                    EditorAssetImportReason::Manual
                ]
            );
            assert_eq!(ticket.try_result().unwrap().unwrap().status(), &original);
        }
        assert_eq!(import_state(&index, &target), before_index_state);
        let job_id = first.id();
        let cached = flow.submit(request).unwrap();
        assert_eq!(
            cached.id(),
            job_id,
            "committed flight remains in the completion cache"
        );
        assert_eq!(
            cached
                .wait_until(Instant::now() + WAIT)
                .unwrap()
                .unwrap()
                .status(),
            &original
        );
        assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
        drop((first, second, cached));
        assert_eq!(import_state(&index, &target), before_index_state);

        assert!(jobs.shutdown(Instant::now() + WAIT).is_empty());
        assert!(jobs.is_terminal_record(job_id));
        assert_eq!(jobs.pending_job_count(), 0);
        assert_eq!(jobs.running_job_count(), 0);
        assert_eq!(jobs.scheduled_record_count(), 0);
        jobs.pump_events_with_budget(JobEventPumpBudget::new(usize::MAX, Duration::from_secs(1)));
        let events = bus
            .drain_deliveries(subscriber)
            .into_iter()
            .filter_map(|delivery| match delivery.message().payload() {
                EditorMessagePayload::Job(event) if event.id() == job_id => Some(event.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event.kind(), JobEventKind::Completed))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event.kind(),
                    JobEventKind::Completed | JobEventKind::Cancelled | JobEventKind::Failed { .. }
                ))
                .count(),
            1
        );
        let progress = events
            .iter()
            .filter_map(|event| match event.kind() {
                JobEventKind::Progress {
                    completed, total, ..
                } => Some((*completed, *total)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(progress, vec![(0, 1), (1, 1)]);
        let diagnostics = logs.snapshot(&LogFilter::new(
            BTreeSet::from([LogChannel::Import]),
            LogSeverity::Info,
        ));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].entry().severity(), LogSeverity::Info);
        assert!(diagnostics[0]
            .entry()
            .message()
            .contains("result=committed"));
        let manager = resolve_manager_service(&runtime.handle(), handle).unwrap();
        assert_eq!(
            AssetManager::asset_status(&*manager, TARGET).unwrap(),
            original
        );
        assert_eq!(
            AssetManager::current_project_snapshot(&*manager)
                .unwrap()
                .catalog_input_generation()
                .sequence(),
            committed_sequence
        );
        assert!(changes.try_recv().is_err());
    }));

    // Release before any real join, including the assertion-panic path.
    gate.release();
    let unfinished = jobs.shutdown(Instant::now() + WAIT);
    let watchers_joined = cleanup_handle.as_ref().map(|handle| {
        resolve_manager_service(&runtime.handle(), handle.clone())
            .map(|manager| manager.shutdown_project_watchers_until(Instant::now() + WAIT))
    });
    let modules_stopped = runtime.deactivate_module_with_drain_timeout(ASSET_MODULE_NAME, WAIT);
    let graph_stopped = runtime.shutdown_task_graph(WAIT);
    if let Err(payload) = outcome {
        resume_unwind(payload);
    }
    assert!(
        unfinished.is_empty(),
        "editor jobs must join: {unfinished:?}"
    );
    assert!(
        watchers_joined.unwrap().unwrap(),
        "project watchers must join"
    );
    modules_stopped.unwrap();
    graph_stopped.unwrap();
}

#[path = "registered_project_scope.rs"]
mod registered_project_scope;
