use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::project::{render_project_template, ProjectTemplateId};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeSessionConfigV3, ZrRuntimeSessionHandle, ZrRuntimeWakeSinkV1, ZrStatus,
    ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V3,
    ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

use crate::asset::project::ProjectPaths;
use crate::core::{LifecycleState, TaskGraphShutdownReport};
use crate::scene::DynamicScene;

use super::super::registry::{
    observe_runtime_startup_for_test, session_handle_is_published_for_test, OwnerShutdownReceipt,
    RuntimeStartupObservation,
};

use super::super::project::RuntimeProjectConfig;
use super::super::RuntimeProjectError;
use super::super::{
    RuntimeDynamicSession, RuntimeDynamicSessionError, RuntimeDynamicSessionProfile,
};

const DEFAULT_NAVMESH: &str = "assets/navigation/main.navmesh.toml";
const PLAY_SCENE: &str = ".zircon/play/instance/late-error.zrscene.json";

#[test]
fn normal_invalid_utf8_play_scene_preserves_primary_and_closes_activated_core() {
    let fixture = PlaySceneProject::create();
    let mut healthy = RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Headless,
        Some(fixture.config()),
    )
    .expect("the same real template and valid Play snapshot must start successfully");
    assert!(healthy.shutdown_before_library_unload());
    let healthy_report = healthy
        .runtime
        .shutdown_task_graph_until(Instant::now() + Duration::from_secs(5))
        .expect("explicit shutdown must retain the native join report");
    assert_joined(&healthy_report);
    drop(healthy);

    std::fs::write(fixture.root.join(PLAY_SCENE), [0xff])
        .expect("ordinary invalid UTF-8 Play file");
    let failure = match RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Headless,
        Some(fixture.config()),
    ) {
        Ok(_) => panic!("invalid UTF-8 must return the original real project read error"),
        Err(failure) => failure,
    };
    match failure.primary() {
        RuntimeDynamicSessionError::ProjectStep {
            step: "load Play scene override",
            source: RuntimeProjectError::ReadPlayScene { source, .. },
        } => assert_eq!(source.kind(), std::io::ErrorKind::InvalidData),
        other => panic!("unexpected primary constructor error: {other:?}"),
    }
    assert!(
        failure.running_modules_before_close > 0,
        "the ordinary error occurs after activation"
    );
    assert!(!failure.has_pending_core());
    assert!(failure.shutdown_error().is_none());
    assert_joined(
        failure
            .shutdown_report()
            .expect("real graph shutdown receipt"),
    );
    let core = failure
        .observed_core
        .as_ref()
        .expect("the same constructor Core");
    assert!(core
        .inner
        .modules
        .lock()
        .expect("test module registry")
        .values()
        .all(|entry| entry.lifecycle == LifecycleState::Unloaded));
    fixture.assert_removable();
}

#[test]
fn app_core_shutdown_owner_public_abi_late_play_error_returns_diagnostic_without_slot() {
    let fixture = PlaySceneProject::create();
    let root = fixture.root.to_str().expect("UTF-8 fixture root");
    let api = public_runtime_api();
    let create = api.create_session.expect("public create_session");
    let destroy = api.destroy_session.expect("public destroy_session");
    let config = public_startup_config(root);
    let mut healthy = ZrRuntimeSessionHandle::invalid();
    let status = unsafe { create(config, &mut healthy) };
    assert_eq!(
        status.status_code(),
        ZrStatusCode::Ok,
        "{}",
        copy_status_diagnostic(status)
    );
    assert!(healthy.is_valid());
    assert!(session_handle_is_published_for_test(healthy));
    let status = unsafe { destroy(healthy) };
    assert_eq!(
        status.status_code(),
        ZrStatusCode::Ok,
        "{}",
        copy_status_diagnostic(status)
    );
    assert!(!session_handle_is_published_for_test(healthy));

    std::fs::write(fixture.root.join(PLAY_SCENE), [0xff]).expect("corrupt this existing Play file");
    // A nonzero, already-destroyed handle proves the public function clears caller output.
    let mut output = healthy;
    let (status, observation) = observe_runtime_startup_for_test(|| unsafe {
        create(public_startup_config(root), &mut output)
    });
    let diagnostic = copy_status_diagnostic(status);
    assert_eq!(status.status_code(), ZrStatusCode::Error);
    assert_eq!(output, ZrRuntimeSessionHandle::invalid());
    assert_eq!(
        diagnostic,
        observation
            .primary_diagnostic
            .as_ref()
            .expect("original typed diagnostic")
            .as_str(),
    );
    assert_public_startup_closed(&observation, "load Play scene override");
    fixture.assert_removable();
}

#[test]
fn app_core_shutdown_owner_public_linked_late_navmesh_error_closes_without_slot() {
    let fixture = PlaySceneProject::create();
    fixture.prepare_core_only_linked_project();
    let api = public_runtime_api();
    let healthy = crate::dynamic_api::create_linked_runtime_session(
        b"headless",
        Some(&fixture.root),
        Vec::new(),
    )
    .expect("the same real linked project and nonempty navmesh must start publicly");
    assert!(healthy.is_valid());
    assert!(session_handle_is_published_for_test(healthy));
    let status = unsafe { api.destroy_session.expect("public destroy_session")(healthy) };
    assert_eq!(
        status.status_code(),
        ZrStatusCode::Ok,
        "{}",
        copy_status_diagnostic(status)
    );
    assert!(!session_handle_is_published_for_test(healthy));

    std::fs::write(fixture.root.join(DEFAULT_NAVMESH), [0xff])
        .expect("corrupt this existing valid default navmesh file");
    let (result, observation) = observe_runtime_startup_for_test(|| {
        crate::dynamic_api::create_linked_runtime_session(
            b"headless",
            Some(&fixture.root),
            Vec::new(),
        )
    });
    let error = result.expect_err("ordinary late linked read failure");
    match &error {
        RuntimeDynamicSessionError::ModuleDiscovery { message } => assert_eq!(
            message,
            observation
                .primary_diagnostic
                .as_ref()
                .expect("original typed diagnostic"),
        ),
        other => panic!("the current public owner boundary returns ModuleDiscovery: {other:?}"),
    }
    assert_eq!(
        error.to_string(),
        format!(
            "runtime module discovery failed: {}",
            observation.primary_diagnostic.as_ref().unwrap(),
        ),
    );
    assert!(std::error::Error::source(&error).is_none());
    assert_public_startup_closed(&observation, "load default project navigation");
    fixture.assert_removable();
}

fn assert_public_startup_closed(observation: &RuntimeStartupObservation, step: &'static str) {
    let allocated = observation
        .allocated_handle
        .expect("this public call's real allocated handle");
    assert_ne!(allocated, 0);
    assert!(!session_handle_is_published_for_test(
        ZrRuntimeSessionHandle::new(allocated)
    ));
    assert_eq!(observation.primary_step, Some(step));
    assert_eq!(
        observation.primary_read_error_kind,
        Some(std::io::ErrorKind::InvalidData),
    );
    assert!(observation.core_identity.is_some());
    assert!(observation.registered_module_count > 0);
    assert_eq!(observation.all_modules_unloaded, Some(true));
    assert_eq!(observation.core_pending, Some(false));
    assert!(observation.secondary_shutdown_diagnostic.is_none());
    assert_joined(
        observation
            .core_shutdown_report
            .as_ref()
            .expect("the actual construction Core report"),
    );
    assert_eq!(observation.owner_cleanup_completed, Some(true));
    assert_eq!(
        observation.owner_shutdown_receipt,
        Some(OwnerShutdownReceipt::Joined)
    );
}

fn public_runtime_api() -> &'static zircon_runtime_interface::ZrRuntimeApiV8 {
    unsafe { &*crate::dynamic_api::zircon_runtime_get_api_v8(core::ptr::null()) }
}

fn public_startup_config(root: &str) -> ZrRuntimeSessionConfigV3 {
    ZrRuntimeSessionConfigV3 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
        profile: ZrByteSlice::from_static(b"headless"),
        project_root: byte_slice(root.as_bytes()),
        play_scene: ZrByteSlice::from_static(PLAY_SCENE.as_bytes()),
        play_report_pipe: ZrByteSlice::empty(),
        wake_sink: ZrRuntimeWakeSinkV1::disabled(),
    }
}

fn copy_status_diagnostic(status: ZrStatus) -> String {
    let bytes = unsafe {
        status
            .diagnostics
            .checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
    }
    .expect("public bounded diagnostic");
    String::from_utf8(bytes.to_vec()).expect("public UTF-8 diagnostic")
}

fn assert_joined(report: &TaskGraphShutdownReport) {
    assert!(!report.has_in_flight_work());
    assert!(report.timer_joined);
    assert!(report.scopes.iter().all(|scope| scope.is_quiescent()));
    assert_eq!(report.worker_shutdowns.len(), 3);
    assert!(report
        .worker_shutdowns
        .iter()
        .all(|workers| workers.expected_worker_count > 0 && workers.all_joined()));
}

struct PlaySceneProject {
    root: PathBuf,
    fixture_parent: PathBuf,
    created_root: PathBuf,
    removed: Cell<bool>,
}

impl PlaySceneProject {
    fn create() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let executable = std::env::current_exe().expect("test executable");
        let binary_directory = executable.parent().expect("test executable parent");
        let binary_directory = std::fs::canonicalize(
            ProjectPaths::resolve_existing(binary_directory)
                .expect("physical test binary directory")
                .operation_path(),
        )
        .expect("canonical physical test binary directory");
        let fixture_parent = binary_directory.join("runtime-construction-error-cleanup");
        std::fs::create_dir_all(&fixture_parent).expect("named fixture parent");
        let fixture_parent =
            std::fs::canonicalize(&fixture_parent).expect("resolve fixture parent");
        assert_eq!(fixture_parent.parent(), Some(binary_directory.as_path()));
        let root = fixture_parent.join(format!(
            "play-{}-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).expect("exclusively reserve this test's new fixture child");
        let created_root = std::fs::canonicalize(&root).expect("resolve exclusively created child");
        assert_eq!(created_root.parent(), Some(fixture_parent.as_path()));
        let fixture = Self {
            root: created_root.clone(),
            fixture_parent,
            created_root,
            removed: Cell::new(false),
        };
        write_template_project(&fixture.root);
        let path = fixture.root.join(PLAY_SCENE);
        std::fs::create_dir_all(path.parent().unwrap()).expect("Play snapshot directory");
        let document = DynamicScene::empty()
            .to_versioned_json_pretty()
            .expect("empty scene JSON");
        std::fs::write(path, document).expect("valid Play snapshot");
        fixture
    }

    fn config(&self) -> RuntimeProjectConfig {
        let root = self.root.to_string_lossy();
        RuntimeProjectConfig::from_abi_startup_config(
            byte_slice(root.as_bytes()),
            byte_slice(PLAY_SCENE.as_bytes()),
            ZrByteSlice::empty(),
        )
        .expect("normal ABI project configuration")
        .expect("project config")
    }

    fn prepare_core_only_linked_project(&self) {
        let manifest_path = self.root.join("zircon-project.toml");
        let mut manifest = toml::from_str::<toml::Table>(
            &std::fs::read_to_string(&manifest_path).expect("own real template manifest"),
        )
        .expect("valid template manifest");
        let mut plugins = crate::builtin::manifest_with_mode_baseline(
            RuntimeDynamicSessionProfile::Headless.target_mode(),
            None,
        );
        for selection in &mut plugins.selections {
            selection.enabled = false;
            selection.required = false;
        }
        manifest.insert(
            "plugins".to_owned(),
            toml::Value::try_from(plugins).expect("actual project plugin manifest"),
        );
        manifest.insert(
            "default_scene".to_owned(),
            toml::Value::String("res://scenes/startup-baseline.scene.toml".to_owned()),
        );
        std::fs::write(
            manifest_path,
            toml::to_string_pretty(&manifest).expect("real core-only manifest"),
        )
        .expect("rewrite only this fixture's manifest");
        let scene = crate::asset::assets::SceneAsset {
            entities: Vec::new(),
        };
        let scene_document = scene
            .to_project_toml_string(|_| unreachable!("the empty scene has no asset references"))
            .expect("actual project scene serializer");
        std::fs::write(
            self.root.join("assets/scenes/startup-baseline.scene.toml"),
            scene_document,
        )
        .expect("valid dependency-free authored default scene");
        let navmesh =
            crate::core::framework::navigation::NavMeshAsset::simple_quad("startup-baseline", 1.0);
        let document =
            toml::to_string_pretty(&navmesh).expect("actual nonempty navmesh serializer");
        assert_eq!(
            toml::from_str::<crate::core::framework::navigation::NavMeshAsset>(&document)
                .expect("actual navmesh deserializer"),
            navmesh,
        );
        let navmesh_path = self.root.join(DEFAULT_NAVMESH);
        std::fs::create_dir_all(navmesh_path.parent().unwrap()).expect("own navmesh directory");
        std::fs::write(navmesh_path, document).expect("valid existing default navmesh");
    }

    fn assert_removable(&self) {
        self.remove_owned_root()
            .expect("closed constructor must release project handles");
        assert!(!self.root.exists());
    }

    fn remove_owned_root(&self) -> std::io::Result<()> {
        if self.removed.get() {
            return Ok(());
        }
        let current = std::fs::canonicalize(&self.root)?;
        if current != self.created_root || current.parent() != Some(self.fixture_parent.as_path()) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "test fixture identity changed; refuse recursive removal",
            ));
        }
        std::fs::remove_dir_all(&self.root)?;
        self.removed.set(true);
        Ok(())
    }
}

impl Drop for PlaySceneProject {
    fn drop(&mut self) {
        let _ = self.remove_owned_root();
    }
}

fn write_template_project(root: &Path) {
    let rendered =
        render_project_template(ProjectTemplateId::RenderableEmpty, "ConstructionCleanup")
            .expect("real product template");
    for entry in rendered.entries {
        let destination = entry.path.join_to(root);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).expect("template directory");
        }
        std::fs::write(destination, entry.bytes).expect("template entry");
    }
    ProjectPaths::from_root(root)
        .expect("project paths")
        .ensure_derived_layout()
        .expect("derived project layout");
}

fn byte_slice(bytes: &[u8]) -> ZrByteSlice {
    ZrByteSlice {
        data: bytes.as_ptr(),
        len: bytes.len(),
    }
}

#[cfg(windows)]
#[path = "../error_cleanup_tests/tests/receipt_publication.rs"]
mod receipt_publication;
