use std::path::Path;
use std::time::{Duration, Instant};

use zircon_runtime_interface::project::session_lock::{
    decode_project_session_admission_record, project_session_lock_path,
    ProjectSessionAdmissionLifecycleV1 as Lifecycle,
};
use zircon_runtime_interface::project::{
    ProjectLaunchIntent, ProjectLaunchProfile, ProjectLaunchSource,
};

use crate::core::editing::engine::HistoryContextId;
use crate::core::gateway::DetachedEditorRuntimeGateway;
use crate::core::project::{NewProjectDraft, ProjectAuthority, ProjectTemplateId};
use crate::core::settings::SETTINGS_USER_ROOT_ENV;
use crate::tests::editor_event::support::EnvironmentVariableRestoreGuard as RestoreEnv;
use crate::tests::support::test_project_creation_provenance;
use crate::ui::host::{ProjectCloseCoordinatorPhase as Phase, ProjectCloseOperation};
use crate::ui::retained_host::primitives::CloseRequestResponse;
use crate::ui::workbench::startup::WELCOME_PAGE_ID;

use super::super::support::*;

const STALE_OWNER: &str = "The close coordinator no longer owns the approved activation.";

#[test]
#[ignore = "requires a preregistered physical Editor fixture root"]
fn r21_reopen_control_cold_project_native_close() {
    run(false);
}

#[test]
#[ignore = "requires a preregistered physical Editor fixture root"]
fn r21_reopen_file_close_open_other_project_then_native_close() {
    run(true);
}

fn run(reopen: bool) {
    let _guard = lock_env();
    let output = fixture_root();
    let _restore =
        ["ZIRCON_CONFIG_PATH", "USERPROFILE", SETTINGS_USER_ROOT_ENV].map(RestoreEnv::capture);
    std::env::set_var("ZIRCON_CONFIG_PATH", output.join("config.json"));
    std::env::set_var("USERPROFILE", output.join("home"));
    std::env::set_var(SETTINGS_USER_ROOT_ENV, output.join("settings"));
    let host = ColdHost::new(&output);
    let a = host.ready(&output.join("ProjectA"));
    let (target, retired) = if reopen {
        let location = ProjectPaths::display_path(&output)
            .to_string_lossy()
            .into_owned();
        // Create B's normal on-disk input; do not open it through a second host or manager.
        let b = ProjectAuthority::default()
            .create_project(
                &NewProjectDraft {
                    project_name: "ProjectB".into(),
                    location: location.clone(),
                    template: ProjectTemplateId::RenderableEmpty,
                },
                &test_project_creation_provenance(),
            )
            .expect("fixture: create an existing project B");
        host.close_project_to_welcome(&a);
        host.open_existing_from_welcome(&location);
        let b = host.ready(&b.root);
        assert_ne!(a.project_root(), b.project_root());
        assert_ne!(a.operation_id(), b.operation_id());
        assert_released(&a);
        (b, Some(a))
    } else {
        (a, None)
    };
    eprintln!("R21_REOPEN_BEFORE_X target={target:?} retired={retired:?}");
    host.native_close(&target, retired.as_ref());
}

fn fixture_root() -> PathBuf {
    let approved = fs::canonicalize(r"E:\cargo-targets\zircon-local").unwrap();
    assert!(approved
        .to_string_lossy()
        .replace('\\', "/")
        .eq_ignore_ascii_case("//?/E:/cargo-targets/zircon-local"));
    let base = fs::canonicalize(
        std::env::var_os("ZIRCON_TEST_OUTPUT_ROOT")
            .expect("use an isolated, preregistered physical E fixture directory"),
    )
    .unwrap();
    assert!(base != approved && base.starts_with(&approved));
    assert!(base.join(".zr-editor-owner-test").is_file());
    assert_eq!(fs::canonicalize(std::env::temp_dir()).unwrap(), base);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let output = base.join(format!("r21-ro-{}-{nonce}", std::process::id()));
    fs::create_dir(&output).expect("fresh run directory");
    eprintln!("R21_REOPEN_OUTPUT={}", output.display());
    output // Retain both passing and failing fixtures for the runner; never delete user roots.
}

fn assert_released(operation: &ProjectCloseOperation) {
    assert!(!project_session_lock_path(operation.project_root())
        .try_exists()
        .unwrap());
}

fn assert_persisted_ready(operation: &ProjectCloseOperation) {
    let payload = fs::read_to_string(project_session_lock_path(operation.project_root())).unwrap();
    let record = decode_project_session_admission_record(&payload).unwrap();
    assert_eq!(record.process_id(), std::process::id());
    assert_eq!(record.operation_id(), operation.operation_id());
    assert_eq!(record.lifecycle(), Lifecycle::Ready);
}

struct ColdHost {
    retained: Rc<RefCell<RetainedEditorHost>>,
    ui: UiHostWindow,
    _core: CoreRuntime,
}

impl ColdHost {
    fn new(output: &Path) -> Self {
        let core = CoreRuntime::new();
        for module in zircon_runtime::builtin::runtime_core_modules() {
            core.register_module(module.descriptor()).unwrap();
        }
        core.register_module(zircon_runtime::ui::module_descriptor())
            .unwrap();
        core.register_module(module::module_descriptor()).unwrap();
        for name in [
            FOUNDATION_MODULE_NAME,
            zircon_runtime::asset::ASSET_MODULE_NAME,
            zircon_runtime::core::framework::scene::SCENE_MODULE_NAME,
            module::EDITOR_MODULE_NAME,
        ] {
            core.activate_module(name).unwrap();
        }
        let provenance = test_project_creation_provenance();
        let intent = ProjectLaunchIntent::create_project(
            provenance.operation_id(),
            ProjectLaunchSource::Cli,
            ProjectLaunchProfile::Normal,
            "ProjectA",
            ProjectPaths::display_path(output),
            ProjectTemplateId::RenderableEmpty,
        )
        .unwrap();
        let startup = EditorGuiStartupRequest::project(intent)
            .preflight_project(&ProjectAuthority::default())
            .unwrap();
        let ui = UiHostWindow::new().unwrap();
        ui.window().set_size(PhysicalSize::new(1280, 720));
        ui.show().unwrap();
        let mut retained = RetainedEditorHost::new(
            core.handle(),
            Arc::new(DetachedEditorRuntimeGateway),
            ui.clone_strong(),
            Some(startup),
            Some(provenance.build_set_id().clone()),
            None,
        )
        .expect("fixture: one real cold retained Project host");
        // Only swap the lazy viewport renderer before the first frame, not the authoring world.
        let viewport = RetainedViewportController::new_test_stub();
        viewport.bind_jobs(retained.runtime.context().jobs().clone());
        ui.set_runtime_presenter_factory(viewport.runtime_presenter_factory());
        retained.viewport = viewport;
        let retained = Rc::new(RefCell::new(retained));
        wire_callbacks(&ui, &retained, None);
        let weak = Rc::downgrade(&retained);
        ui.window()
            .on_close_requested(move || match weak.upgrade() {
                Some(host) => host.borrow_mut().native_main_window_close_requested(),
                None => CloseRequestResponse::KeepWindowShown,
            });
        retained.borrow_mut().self_handle = Some(Rc::downgrade(&retained));
        retained.borrow_mut().refresh_ui();
        retained.borrow_mut().sync_hub_focus_binding().unwrap();
        Self {
            retained,
            ui,
            _core: core,
        }
    }

    fn ready(&self, root: &Path) -> ProjectCloseOperation {
        let host = self.retained.borrow();
        let manager = host.editor_manager.enter().unwrap();
        let ready = manager.project_close_activation_identity();
        assert!(ready.is_some(), "fixture: {}", host.runtime.status_line());
        let ready = ready.unwrap();
        let actual = ready.project_root().canonicalize().unwrap();
        assert_eq!(actual, root.canonicalize().unwrap());
        let scene = manager
            .active_scene_identity_for_session()
            .expect("fixture: startup Scene document");
        assert_eq!(scene.project_root(), ready.project_root());
        let history = HistoryContextId::Document(scene.document());
        assert_eq!(host.runtime.active_scene_history_context(), Some(history));
        let snapshot = host
            .runtime
            .project_scene_snapshot()
            .expect("fixture: current gateway world");
        assert!(snapshot.is_some());
        assert!(manager.project_session_heartbeat_deadline().is_some());
        assert_eq!(host.startup_session.mode, EditorSessionMode::Project);
        assert_eq!(
            manager.current_layout().active_main_page,
            MainPageId::workbench()
        );
        assert!(manager.dirty_document_toolkits().unwrap().is_empty());
        assert!(host.dirty_project_scene_token().unwrap().is_none());
        assert!(host.pending_model_import.is_none());
        assert!(!host.main_window_close_requested());
        assert_persisted_ready(&ready);
        ready
    }

    fn close_project_to_welcome(&self, a: &ProjectCloseOperation) {
        let mut host = self.retained.borrow_mut();
        let effects =
            callback_dispatch::dispatch_menu_action(&host.runtime, "workbench.project.close")
                .unwrap();
        assert!(effects.close_active_project);
        host.apply_dispatch_effects(effects);
        host.refresh_ui();
        assert_eq!(
            host.startup_session.mode,
            EditorSessionMode::Welcome,
            "fixture: {}",
            host.runtime.status_line()
        );
        assert_eq!(
            host.runtime.current_layout().unwrap().active_main_page,
            MainPageId::new(WELCOME_PAGE_ID)
        );
        let manager = host.editor_manager.enter().unwrap();
        assert!(manager.project_close_activation_identity().is_none());
        assert!(host.runtime.project_scene_snapshot().unwrap().is_none());
        assert!(!host.main_window_close_requested());
        assert!(!self.ui.exit_requested_for_test());
        assert_released(a);
    }

    fn open_existing_from_welcome(&self, location: &str) {
        let pane = pane_surface_host(&self.ui);
        pane.invoke_welcome_control_changed(
            "welcome.project.location.edit".into(),
            location.into(),
        );
        pane.invoke_welcome_control_changed("welcome.project.name.edit".into(), "ProjectB".into());
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.retained.borrow_mut().tick();
            let welcome = pane.get_welcome_pane();
            let enabled = welcome.form.can_open_existing
                && welcome.form.project_name.as_str() == "ProjectB"
                && welcome.form.location.as_str() == location
                && welcome.nodes.iter().any(|node| {
                    node.control_id.as_str() == "WelcomeOpenExistingButton"
                        && node.action_id.as_str() == "welcome.project.open_existing"
                        && !node.disabled
                });
            if enabled {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "fixture: real Welcome B probe/projection: {}",
                self.retained.borrow().startup_session.creation_validation
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        pane.invoke_welcome_control_clicked("welcome.project.open_existing".into());
        self.retained.borrow_mut().tick();
    }

    fn native_close(
        &self,
        target: &ProjectCloseOperation,
        retired: Option<&ProjectCloseOperation>,
    ) {
        assert_eq!(
            self.ui.dispatch_native_close_request_for_test(),
            CloseRequestResponse::KeepWindowShown
        );
        assert!(self.retained.borrow().main_window_close_requested());
        assert!(self.retained.borrow().pending_close_prompt.is_none());
        let deadline = Instant::now() + Duration::from_secs(6);
        while !self.ui.exit_requested_for_test() {
            self.retained.borrow_mut().tick();
            let host = self.retained.borrow();
            let status = host.runtime.status_line();
            if status == STALE_OWNER {
                let a = retired.expect("same-host stale owner requires a previous project");
                assert_eq!(host.project_close_coordinator.phase(), Phase::Closed);
                assert_eq!(host.project_close_coordinator.operation(), Some(a));
                assert!(host.main_window_close_blocks_session_ui());
                assert!(!self.ui.exit_requested_for_test());
                let manager = host.editor_manager.enter().unwrap();
                let active = manager.project_close_activation_identity();
                assert_eq!(active.as_ref(), Some(target));
                assert_persisted_ready(target);
                assert_released(a);
                panic!("R21_REOPEN_STALE_OWNER: {status} A={a:?} B={target:?}");
            }
            assert!(
                Instant::now() < deadline,
                "fixture or another close failure (not R21 RED): {status}"
            );
            drop(host);
            std::thread::sleep(Duration::from_millis(5));
        }
        let host = self.retained.borrow();
        assert!(host.main_window_close_complete());
        let manager = host.editor_manager.enter().unwrap();
        assert!(manager.project_close_activation_identity().is_none());
        assert_released(target);
        if let Some(a) = retired {
            assert_released(a);
        }
    }
}
