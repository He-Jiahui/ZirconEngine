#![cfg(windows)]

use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use zircon_runtime::asset::pipeline::manager::ProjectAssetGenerationToken;
use zircon_runtime::asset::{AssetManager, ImportedAsset};
use zircon_runtime_interface::project::{
    AssetRef, ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource, RelPath,
};

use super::*;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::editor_message::{EditorMessagePayload, EditorTopic, TOPIC_JOB};
use crate::core::gateway::DetachedEditorRuntimeGateway;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
use crate::core::jobs::{EditorJobShutdownState as Shutdown, JobCategory, JobEventKind, JobId};
use crate::core::logging::LogFilter;
use crate::core::notifications::DecisionNotificationSnapshot;
use crate::core::project::{ProjectAuthority, ProjectTemplateId};
use crate::core::recovery::ProjectSessionEffect;
use crate::core::settings::SETTINGS_USER_ROOT_ENV;
use crate::tests::editor_event::support::EnvironmentVariableRestoreGuard as RestoreEnv;
use crate::tests::support::test_project_creation_provenance;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetReadObservation,
};
use crate::ui::host::{ProjectCloseCoordinatorPhase as Phase, ProjectCloseOperation};
use crate::ui::retained_host::app::project_close::RetainedProjectCloseError as CloseError;
use crate::ui::retained_host::primitives::CloseRequestResponse;

const STARTUP_WAIT: Duration = Duration::from_secs(15);
const EXIT_WAIT: Duration = Duration::from_secs(6);
const SCENE_URI: &str = "res://scenes/main.scene.toml";
const MODEL_FILE: &str = "close-triangle.obj";
const MODEL_URI: &str = "res://models/close-triangle.obj";
const TRIANGLE: &[u8] = b"o CloseTriangle\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";

fn managed_fixture_path(label: &str) -> PathBuf {
    let approved = Path::new(r"E:\cargo-targets\zircon-local");
    let base = fs::canonicalize(approved).expect("preregister the physical E fixture base");
    assert!(ProjectPaths::same_lexical_path(approved, &base).unwrap());
    let supplied =
        std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT").map_or_else(std::env::temp_dir, PathBuf::from);
    assert!(supplied.is_absolute());
    let scratch = fs::canonicalize(&supplied).expect("preregister the isolated fixture directory");
    assert!(
        scratch != base && scratch.starts_with(&base),
        "set ZIRCON_TEST_OUTPUT_ROOT or TEMP/TMP to an owned physical E: fixture directory"
    );
    assert!(ProjectPaths::same_lexical_path(&supplied, &scratch).unwrap());
    for path in supplied.ancestors().chain(approved.ancestors()) {
        let metadata = fs::symlink_metadata(path).expect("read physical fixture ancestor");
        assert!(metadata.is_dir() && metadata.file_attributes() & 0x400 == 0);
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = scratch.join(format!("mc-{label}-{}-{nonce}", std::process::id()));
    fs::create_dir(&root).expect("create a fresh owned close fixture");
    eprintln!("MODEL_CLOSE_OWNER_FIXTURE={}", root.display());
    root
}

struct ColdHost {
    host: Rc<RefCell<RetainedEditorHost>>,
    root_ui: UiHostWindow,
    _core: CoreRuntime,
}

impl ColdHost {
    fn new(root: &Path) -> Self {
        Self::with_launch(root, None)
    }

    fn with_launch(root: &Path, existing: Option<&Path>) -> Self {
        let _config = RestoreEnv::capture("ZIRCON_CONFIG_PATH");
        std::env::set_var("ZIRCON_CONFIG_PATH", root.join("config.json"));
        let core = CoreRuntime::new();
        for engine_module in zircon_runtime::builtin::runtime_core_modules() {
            core.register_module(engine_module.descriptor()).unwrap();
        }
        core.register_module(zircon_runtime::ui::module_descriptor())
            .unwrap();
        core.register_module(module::module_descriptor()).unwrap();
        core.activate_module(FOUNDATION_MODULE_NAME).unwrap();
        core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
            .unwrap();
        core.activate_module(zircon_runtime::core::framework::scene::SCENE_MODULE_NAME)
            .unwrap();
        core.activate_module(module::EDITOR_MODULE_NAME).unwrap();

        // Both CLI launch routes admit a project before constructing EditorState.
        let provenance = test_project_creation_provenance();
        let intent = match existing {
            Some(project_root) => ProjectLaunchIntent::open_existing(
                provenance.operation_id(),
                ProjectLaunchSource::Cli,
                ProjectLaunchProfile::Normal,
                ProjectPaths::display_path(project_root),
            ),
            None => ProjectLaunchIntent::create_project(
                provenance.operation_id(),
                ProjectLaunchSource::Cli,
                ProjectLaunchProfile::Normal,
                "CloseModel",
                ProjectPaths::display_path(root),
                ProjectTemplateId::RenderableEmpty,
            ),
        }
        .expect("fixture: cold project intent");
        let request = EditorGuiStartupRequest::project(intent)
            .preflight_project(&ProjectAuthority::default())
            .expect("fixture: preflight the real RenderableEmpty template");
        let root_ui = UiHostWindow::new().expect("fixture: root UI");
        root_ui.show().expect("fixture: show the root test window");
        let mut retained = RetainedEditorHost::new(
            core.handle(),
            Arc::new(DetachedEditorRuntimeGateway),
            root_ui.clone_strong(),
            Some(request),
            Some(provenance.build_set_id().clone()),
            None,
        )
        .expect("fixture: cold retained startup and Scene binding");

        // Preserve the original CPU-only viewport seam, before any frame or render job.
        let viewport = RetainedViewportController::new_test_stub();
        viewport.bind_jobs(retained.runtime.context().jobs().clone());
        root_ui.set_runtime_presenter_factory(viewport.runtime_presenter_factory());
        retained.viewport = viewport;
        let host = Rc::new(RefCell::new(retained));
        wire_callbacks(&root_ui, &host, None);
        let close_host = Rc::downgrade(&host);
        root_ui
            .window()
            .on_close_requested(move || match close_host.upgrade() {
                Some(host) => host.borrow_mut().native_main_window_close_requested(),
                None => CloseRequestResponse::KeepWindowShown,
            });
        host.borrow_mut().self_handle = Some(Rc::downgrade(&host));
        host.borrow_mut().refresh_ui();
        host.borrow_mut()
            .sync_hub_focus_binding()
            .expect("fixture: bind the admitted project's focus owner");
        Self {
            host,
            root_ui,
            _core: core,
        }
    }
}

struct Fixture {
    h: ColdHost,
    root: PathBuf,
    activation: ProjectCloseOperation,
    _restore: [RestoreEnv; 2],
}

impl Fixture {
    fn new(label: &str) -> Self {
        Self::with_host(label, ColdHost::new)
    }

    fn with_host(label: &str, start: impl FnOnce(&Path) -> ColdHost) -> Self {
        let root = managed_fixture_path(label);
        let restore = [
            RestoreEnv::capture("USERPROFILE"),
            RestoreEnv::capture(SETTINGS_USER_ROOT_ENV),
        ];
        std::env::set_var("USERPROFILE", root.join("home"));
        std::env::set_var(SETTINGS_USER_ROOT_ENV, root.join("settings"));
        let h = start(&root);
        let activation = {
            let host = h.host.borrow();
            let manager = host.editor_manager.enter().unwrap();
            let activation = manager
                .project_close_activation_identity()
                .expect("real close owner");
            let scene = manager
                .active_scene_identity_for_session()
                .expect("active Scene document");
            assert!(fs::canonicalize(activation.project_root())
                .unwrap()
                .starts_with(&root));
            assert!(manager.project_session_heartbeat_deadline().is_some());
            assert_eq!(scene.scene_uri(), SCENE_URI);
            assert_eq!(
                host.runtime.active_scene_history_context(),
                Some(HistoryContextId::Document(scene.document()))
            );
            assert!(host
                .runtime
                .project_scene_snapshot()
                .expect("fixture: current authoring gateway")
                .is_some());
            assert!(manager.dirty_document_toolkits().unwrap().is_empty());
            assert!(host.dirty_project_scene_token().unwrap().is_none());
            assert_eq!(host.project_close_coordinator.phase(), Phase::Decision);
            activation
        };
        Self {
            h,
            root,
            activation,
            _restore: restore,
        }
    }

    fn wait_for_catalog(&self) {
        // Only progress real startup frame work, before submitting the import under test.
        let deadline = Instant::now() + STARTUP_WAIT;
        let mut host = self.h.host.borrow_mut();
        let open_log = host
            .runtime
            .context()
            .logs()
            .snapshot_tail(&LogFilter::default(), 64)
            .into_iter()
            .rev()
            .find(|record| record.entry().message().starts_with("editor_project_open "));
        if let Some(record) = open_log {
            eprintln!(
                "MODEL_CLOSE_OWNER_STARTUP_OPEN_LOG seq={} {}",
                record.sequence(),
                record.entry().message()
            );
        } else {
            eprintln!("MODEL_CLOSE_OWNER_STARTUP_OPEN_LOG not_in_bounded_tail");
        }
        let mut entry_sample_taken = false;
        let mut entry_token = None;
        loop {
            let editor = host.editor_asset_manager_at_use_point().unwrap();
            let catalog = editor.catalog_snapshot();
            drop(editor);
            let scene_indexed = catalog.asset_by_locator(SCENE_URI).is_some();
            let reload_pending = host.pending_active_scene_reload.is_some();
            if !entry_sample_taken {
                entry_token = self.observe_catalog_gate(&host, &catalog, "entry", None);
                entry_sample_taken = true;
            }
            if scene_indexed && !reload_pending {
                self.observe_catalog_gate(&host, &catalog, "predicates_met", entry_token.as_ref());
                let root = self.activation.project_root();
                assert!(
                    ProjectPaths::same_lexical_path(catalog.project_root.as_ref(), root).unwrap()
                );
                return;
            }
            let within_budget = Instant::now() < deadline;
            if !within_budget {
                self.observe_catalog_gate(&host, &catalog, "timeout", entry_token.as_ref());
            }
            assert!(
                within_budget,
                "fixture: startup catalog not available: {}",
                host.runtime.status_line()
            );
            host.tick();
            std::thread::yield_now();
        }
    }

    // Cross-service samples are not atomic; entry_token is the first *observer* token, not
    // the internal Runtime input that Editor's earlier startup refresh selected.
    fn observe_catalog_gate(
        &self,
        host: &RetainedEditorHost,
        catalog: &EditorAssetCatalogGeneration,
        point: &str,
        entry_token: Option<&ProjectAssetGenerationToken>,
    ) -> Option<ProjectAssetGenerationToken> {
        let (close_now, selected_scene) = host
            .editor_manager
            .enter()
            .ok()
            .map(|manager| {
                (
                    manager.project_close_activation_identity(),
                    manager.active_scene_identity_for_session(),
                )
            })
            .unwrap_or_default();
        let scene_index = catalog
            .asset_by_locator(SCENE_URI)
            .map(|row| row.uuid.as_str());
        let scene_row = catalog
            .assets
            .iter()
            .find(|row| row.locator == SCENE_URI)
            .map(|row| row.uuid.as_str());
        eprintln!(
            "MODEL_CLOSE_OWNER_CATALOG gate={point} scene_uri={SCENE_URI} scene_index={scene_index:?} scene_row={scene_row:?} pending_reload={} reload_admission={} reload_conflict={} catalog_root={:?} catalog_same_root={} revision={} epoch={} rows={} expected_close={:?} current_close={close_now:?} selected_scene={selected_scene:?} history={:?}",
            host.pending_active_scene_reload.is_some(),
            host.active_scene_reload_admission.is_some(),
            host.active_scene_reload_conflict.is_some(),
            catalog.project_root,
            !catalog.project_root.is_empty()
                && ProjectPaths::same_lexical_path(catalog.project_root.as_ref(), self.activation.project_root()).unwrap_or(false),
            catalog.catalog_revision,
            catalog.publish_epoch,
            catalog.assets.len(),
            self.activation,
            host.runtime.active_scene_history_context(),
        );
        let Some((project, sampled, sampled_check, entry_check, watch)) = host
            .asset_runtime_access
            .project_asset_manager()
            .ok()
            .and_then(|manager| {
                let (project, token) = manager.current_project_generation_snapshot()?.into_parts();
                let sampled_check = manager.check_project_generation(&token);
                let entry_check = entry_token.map(|entry| manager.check_project_generation(entry));
                let watch = manager.asset_watch_diagnostics();
                Some((project, token, sampled_check, entry_check, watch))
            })
        else {
            eprintln!("MODEL_CLOSE_OWNER_CATALOG_RUNTIME gate={point} unavailable");
            return None;
        };
        let uri = ResourceLocator::parse(SCENE_URI).expect("fixture: fixed scene URI");
        let entry = project.asset_registry().entry_by_path(&uri);
        let input = project.catalog_input_generation();
        let projection = entry.and_then(|record| {
            let path_hint = RelPath::parse(uri.path()).ok()?;
            let reference = AssetRef::try_new(record.uuid(), path_hint, None).ok()?;
            let editor = host.editor_asset_manager_at_use_point().ok()?;
            let observation = editor.readonly_asset_snapshot(&reference, None, None);
            let scope = observation.scope.as_ref().map(|scope| {
                (
                    scope.activation() == sampled.activation_identity(),
                    Arc::ptr_eq(scope.input(), &input),
                    scope.input().sequence(),
                    std::ptr::eq(scope.catalog().as_ref(), catalog),
                )
            });
            let result = match observation.observation {
                EditorAssetReadObservation::Known(facts) => {
                    format!("Known(row={})", facts.asset.is_some())
                }
                other => format!("{other:?}"),
            };
            Some((result, scope))
        });
        eprintln!(
            "MODEL_CLOSE_OWNER_CATALOG_RUNTIME gate={point} token={sampled:?} seq={} same_root={} sampled_check={sampled_check:?} entry_check={entry_check:?} same_entry_activation={:?} runtime_scene_resource={} runtime_scene_uuid={:?} editor_index={projection:?} scope_fields=(activation_matches,input_arc_matches,input_seq,catalog_arc_matches) watch_batches={} watch_reconciles={} watch_commits={} watch_failed={} watch_superseded={}",
            input.sequence(),
            ProjectPaths::same_lexical_path(sampled.project_root(), self.activation.project_root()).unwrap_or(false),
            entry_token.map(|entry| entry.activation_identity() == sampled.activation_identity()),
            project.registry().get_by_locator(&uri).is_some(),
            entry.map(|record| record.uuid()),
            watch.batch_count,
            watch.reconciliation_count,
            watch.committed_generation_count,
            watch.failed_batch_count,
            watch.superseded_generation_count,
        );
        Some(sampled)
    }
}

fn active_project(host: &RetainedEditorHost) -> ProjectManager {
    let assets = host.asset_runtime_access.project_asset_manager().unwrap();
    assets
        .current_project_generation_snapshot()
        .expect("active Runtime project")
        .into_parts()
        .0
}

fn scene_state(host: &RetainedEditorHost) -> (usize, HistoryStatus) {
    let runtime = &host.runtime;
    let history = runtime
        .active_scene_history_context()
        .expect("bound authoring history");
    let scene = runtime
        .project_scene_snapshot()
        .unwrap()
        .expect("authoring scene");
    let transactions = runtime.context().transactions();
    (
        scene.nodes().len(),
        transactions.history_status(history).unwrap(),
    )
}

fn submit_and_observe_worker(h: &ColdHost, source: &Path) -> JobId {
    let context = h.host.borrow().runtime.context().clone();
    let (bus, jobs) = (context.bus(), context.jobs());
    let topic = EditorTopic::parse(TOPIC_JOB).unwrap();
    let subscriber = bus.register_subscriber([topic]).unwrap();
    let label = format!("Import model {}", source.display());
    let pane = pane_surface_host(&h.root_ui);
    pane.invoke_mesh_import_path_edited(source.to_string_lossy().as_ref().into());
    pane.invoke_asset_control_clicked("activity".into(), "ImportModel".into());
    {
        let host = h.host.borrow();
        assert!(
            host.pending_model_import.is_some(),
            "fixture: import submission refused: {}",
            host.runtime.status_line()
        );
    }
    let deadline = Instant::now() + STARTUP_WAIT;
    let mut completed = None;
    let job = loop {
        // The jobs pump publishes lifecycle messages without polling the host ticket.
        jobs.pump_events();
        for delivery in bus.drain_deliveries(subscriber) {
            let event = match delivery.message().payload() {
                EditorMessagePayload::Job(event) => event,
                EditorMessagePayload::JobJournalGap(gap) => panic!("fixture: job gap {gap:?}"),
                _ => continue,
            };
            if event.category() != JobCategory::Import || event.label() != label.as_str() {
                continue;
            }
            match event.kind() {
                JobEventKind::Completed => assert!(completed.replace(event.id()).is_none()),
                JobEventKind::Failed { .. } | JobEventKind::Cancelled => {
                    panic!("fixture: model import did not commit: {event:?}");
                }
                _ => {}
            }
        }
        let progress = jobs.shutdown_progress();
        assert_eq!(progress.state(), Shutdown::NotRequested);
        assert!(progress.cleanup_failure().is_none());
        if let Some(id) = completed {
            // Completed is emitted before send; terminal bookkeeping follows send.
            if jobs.is_terminal_record(id)
                && progress.unfinished_jobs().is_empty()
                && !progress.completion_callbacks_pending()
                && !progress.owned_tasks_pending()
            {
                break id;
            }
        }
        assert!(
            Instant::now() < deadline,
            "fixture: model job not terminal: {progress:?}"
        );
        std::thread::yield_now();
    };
    assert!(bus.unregister_subscriber(subscriber));
    job
}

fn drive_native_close(h: &ColdHost, job: Option<JobId>) -> Option<DecisionNotificationSnapshot> {
    let response = h.root_ui.dispatch_native_close_request_for_test();
    assert_eq!(response, CloseRequestResponse::KeepWindowShown);
    assert!(h.host.borrow().main_window_close_requested());
    assert!(!h.root_ui.get_host_presentation().close_prompt.visible);
    assert!(!h.root_ui.exit_requested_for_test());
    let context = h.host.borrow().runtime.context().clone();
    let center = context.notifications().decisions().unwrap();
    let key = job.map(|id| format!("editor.asset.model_import.{}.", id.value()));
    let deadline = Instant::now() + EXIT_WAIT;
    loop {
        h.host.borrow_mut().tick();
        if let Some(key) = &key {
            for review in center.pending_snapshot() {
                let id = review.notification().id().as_str();
                if id.starts_with(key.as_str()) {
                    return Some(review);
                }
            }
        }
        let host = h.host.borrow();
        let phase = host.project_close_coordinator.phase();
        if h.root_ui.exit_requested_for_test() {
            let manager = host.editor_manager.enter().unwrap();
            let assets = host.asset_runtime_access.project_asset_manager().unwrap();
            assert!(host.main_window_close_complete() && host.pending_model_import.is_none());
            assert_eq!(phase, Phase::Closed);
            assert!(manager.project_close_activation_identity().is_none());
            assert!(assets.current_project_generation_snapshot().is_none());
            assert_eq!(
                context.jobs().shutdown_progress().state(),
                Shutdown::Quiescent
            );
            return None;
        }
        assert!(
            phase != Phase::RecoveryRequired && Instant::now() < deadline,
            "close failed without the model-review signature: {}",
            host.runtime.status_line()
        );
        std::thread::yield_now();
    }
}

#[test]
#[ignore = "requires a preregistered physical E: Windows host fixture"]
fn model_close_owner_clean_session_exits_without_import() {
    let _env = lock_env();
    let f = Fixture::new("clean");
    assert!(f.h.host.borrow().pending_model_import.is_none());
    assert!(drive_native_close(&f.h, None).is_none());
}

#[test]
#[ignore = "requires a preregistered physical E: Windows host fixture"]
fn model_close_owner_exits_after_committed_import_without_review() {
    let _env = lock_env();
    let f = Fixture::new("obj");
    assert_committed_import_close(&f);
}

#[test]
#[ignore = "independent cold --project control on a preregistered physical E: fixture"]
fn model_close_existing_session_exits_after_committed_import_without_review() {
    let _env = lock_env();
    let f = Fixture::with_host("existing", reopen_built_project);
    assert_committed_import_close(&f);
}

// Provision once through the unchanged customer-facing create/close path, then make one
// cold --project launch in a new Core. No reuse of the failed run's files or host state.
fn reopen_built_project(root: &Path) -> ColdHost {
    let first = ColdHost::new(root);
    let uri = ResourceLocator::parse(SCENE_URI).unwrap();
    let (project_root, scene_path, scene_uuid, original_runtime, original_close) = {
        let host = first.host.borrow();
        assert!(host.pending_model_import.is_none());
        let project = active_project(&host);
        let project_root = project.paths().root().to_path_buf();
        assert_eq!(
            project_root,
            fs::canonicalize(root.join("CloseModel")).unwrap()
        );
        assert!(matches!(
            project.load_artifact(&uri).unwrap(),
            ImportedAsset::Scene(_)
        ));
        let scene_uuid = project.asset_registry().entry_by_path(&uri).unwrap().uuid();
        let close = host
            .editor_manager
            .enter()
            .unwrap()
            .project_close_activation_identity()
            .unwrap();
        (
            project_root,
            project.source_path_for_uri(&uri).unwrap(),
            scene_uuid,
            project.activation_identity().clone(),
            close,
        )
    };
    let scene_bytes = fs::read(&scene_path).unwrap();
    assert!(drive_native_close(&first, None).is_none());
    drop(first);
    assert_eq!(fs::read(&scene_path).unwrap(), scene_bytes);
    assert!(!project_root.join(".zircon").join("session.lock").exists());

    let second = ColdHost::with_launch(root, Some(&project_root));
    {
        let host = second.host.borrow();
        let project = active_project(&host);
        assert_eq!(project.paths().root(), project_root.as_path());
        assert_ne!(project.activation_identity(), &original_runtime);
        let close = host
            .editor_manager
            .enter()
            .unwrap()
            .project_close_activation_identity()
            .unwrap();
        assert_ne!(close, original_close);
        assert_eq!(
            project.asset_registry().entry_by_path(&uri).unwrap().uuid(),
            scene_uuid
        );
        assert_eq!(fs::read(&scene_path).unwrap(), scene_bytes);
        eprintln!("MODEL_CLOSE_OWNER_COLD_OPEN_EXISTING root={project_root:?} scene_uuid={scene_uuid} previous_close={original_close:?} current_close={close:?}");
    }
    second
}

fn assert_committed_import_close(f: &Fixture) {
    f.wait_for_catalog();
    let uri = ResourceLocator::parse(MODEL_URI).unwrap();
    let (before, origin) = {
        let host = f.h.host.borrow();
        let project = active_project(&host);
        assert!(ProjectPaths::same_lexical_path(
            project.paths().root(),
            f.activation.project_root()
        )
        .unwrap());
        assert!(project.registry().get_by_locator(&uri).is_none());
        assert!(host.pending_model_import.is_none());
        assert!(host.dirty_project_scene_token().unwrap().is_none());
        (scene_state(&host), project.activation_identity().clone())
    };
    // External sibling source cannot be imported by the project's watcher ahead of us.
    let source = f.root.join(MODEL_FILE);
    fs::write(&source, TRIANGLE).unwrap();
    let job = submit_and_observe_worker(&f.h, &fs::canonicalize(source).unwrap());
    let (installed_source, scene_path, saved_scene) = {
        let host = f.h.host.borrow();
        assert!(
            host.pending_model_import.is_some(),
            "host still holds its unread ticket"
        );
        assert_eq!(scene_state(&host), before);
        let project = active_project(&host);
        assert_eq!(project.activation_identity(), &origin);
        let assets = host.asset_runtime_access.asset_manager().unwrap();
        let status = assets
            .asset_status(MODEL_URI)
            .expect("committed model status");
        assert_eq!(status.uri, MODEL_URI);
        assert!(status.imported && status.artifact_uri.is_some());
        assert_eq!(status.importer_id, "zircon.builtin.model.obj");
        let resources = host.asset_runtime_access.resource_manager().unwrap();
        resolve_ready_handle::<ModelMarker>(&*resources, &uri).expect("live Ready model");
        let artifact = project.load_artifact(&uri).expect("durable model artifact");
        let ImportedAsset::Model(model) = artifact else {
            panic!("fixture: wrong artifact for {MODEL_URI}")
        };
        assert_eq!(model.primitives.len(), 1);
        assert_eq!(model.primitives[0].indices, [0, 1, 2]);
        let source = project.source_path_for_uri(&uri).unwrap();
        assert_eq!(fs::read(&source).unwrap(), TRIANGLE);
        let scene = project
            .source_path_for_uri(&project.manifest().default_scene)
            .unwrap();
        let saved_scene = fs::read(&scene).unwrap();
        let id = job.value();
        eprintln!(
            "MODEL_CLOSE_OWNER_READY job={id} uri={MODEL_URI} builtin={} close={:?}",
            status.importer_id, f.activation
        );
        (source, scene, saved_scene)
    };

    if let Some(review) = drive_native_close(&f.h, Some(job)) {
        let (activation, notice) = (&f.activation, review.notification());
        let options: Vec<_> = notice
            .options()
            .iter()
            .map(|option| option.id().as_str())
            .collect();
        assert_eq!(
            notice.message_key(),
            "editor.asset.model_import.close_pending"
        );
        assert!(options.contains(&"keep") && !options.contains(&"resume"));
        let mut host = f.h.host.borrow_mut();
        let review_status = host.runtime.status_line();
        let coordinator = &host.project_close_coordinator;
        assert_eq!(coordinator.phase(), Phase::Quiescing);
        assert_eq!(coordinator.operation(), Some(activation));
        let manager = host.editor_manager.enter().unwrap();
        manager
            .validate_project_close_operation(activation, ProjectSessionEffect::Session)
            .unwrap();
        assert!(manager.project_session_heartbeat_deadline().is_none());
        drop(manager);
        assert!(host.pending_model_import.is_some());
        assert_eq!(scene_state(&host), before);
        let jobs = host.runtime.context().jobs();
        assert_eq!(jobs.shutdown_progress().state(), Shutdown::NotRequested);
        host.poll_owned_main_window_close();
        assert_eq!(host.project_close_coordinator.phase(), Phase::Quiescing);
        assert_eq!(
            host.runtime.status_line(),
            CloseError::PendingModelImport.to_string()
        );
        assert!(!f.h.root_ui.exit_requested_for_test());
        assert_eq!(fs::read(&installed_source).unwrap(), TRIANGLE);
        panic!("MODEL_CLOSE_OWNER_UNEXPECTED_REVIEW: committed {MODEL_URI} still blocks the same native close; options={options:?}; review status={review_status}");
    }
    assert_eq!(fs::read(installed_source).unwrap(), TRIANGLE);
    assert_eq!(fs::read(scene_path).unwrap(), saved_scene);
}
