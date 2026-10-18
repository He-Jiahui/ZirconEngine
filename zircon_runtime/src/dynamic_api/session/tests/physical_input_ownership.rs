use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::project::{render_project_template, ProjectTemplateId};
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
};

use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::AssetUri;
use crate::core::framework::input::{InputButton, InputEvent};
use crate::dynamic_api::session::project::RuntimeProjectConfig;
use crate::dynamic_api::session::{RuntimeDynamicSession, RuntimeDynamicSessionProfile};

const CAPTURE_SLIDER_VIEW: &str = r#"
[asset]
kind = "view"
id = "tests.runtime.physical_input_ownership"
version = 2
display_name = "Physical Input Ownership"

[root]
node = "capture_slider"

[nodes.capture_slider]
component = "Slider"
control_id = "CaptureSlider"
props = { value = 42.0, min = 0.0, max = 100.0, step = 1.0, input_interactive = true, input_clickable = true, input_hoverable = true, input_focusable = true }
layout = { width = { min = 240.0, preferred = 240.0, max = 240.0, stretch = "Fixed" }, height = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" } }
"#;

#[test]
fn ui_capture_release_commits_physical_state_before_propagation_stop() {
    let fixture = PhysicalInputFixture::create();
    let mut session = runtime_session(&fixture);

    assert_ok(session.handle_event(mouse_button(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, 24.0, 24.0)));
    assert!(session
        .resolve_input_manager()
        .expect("runtime input manager must be available")
        .button_pressed(&InputButton::MouseLeft));
    assert_eq!(
        session
            .resolve_input_manager()
            .expect("runtime input manager must be available")
            .drain_events(),
        vec![InputEvent::ButtonPressed(InputButton::MouseLeft)]
    );

    assert_ok(session.handle_event(mouse_button(
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
        960.0,
        640.0,
    )));

    assert!(
        !session
            .resolve_input_manager()
            .expect("runtime input manager must be available")
            .button_pressed(&InputButton::MouseLeft),
        "UI propagation must not hide the physical release from InputManager"
    );
    assert!(
        session
            .resolve_input_manager()
            .expect("runtime input manager must be available")
            .drain_events()
            .iter()
            .any(|event| *event == InputEvent::ButtonReleased(InputButton::MouseLeft)),
        "the physical release must be journaled even when UI capture stops semantic propagation"
    );

    drop(session);
    fixture.assert_removable();
}

fn mouse_button(state: u32, x: f32, y: f32) -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::mouse_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
        state,
        x,
        y,
    )
}

fn assert_ok(status: zircon_runtime_interface::ZrStatus) {
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}

fn runtime_session(fixture: &PhysicalInputFixture) -> RuntimeDynamicSession {
    RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(
            RuntimeProjectConfig::from_root(&fixture.root)
                .expect("physical input fixture must resolve as a project"),
        ),
    )
    .expect("runtime must load the physical input fixture")
}

struct PhysicalInputFixture {
    root: PathBuf,
}

impl PhysicalInputFixture {
    fn create() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let root = fixture_root().join(format!(
            "physical-input-ownership-{}_{}_{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed),
            unique
        ));
        write_template_project(&root);
        write_ui_root(&root);
        Self { root }
    }

    fn assert_removable(&self) {
        std::fs::remove_dir_all(&self.root)
            .expect("physical input fixture must not retain file handles");
        assert!(!self.root.exists());
    }
}

impl Drop for PhysicalInputFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn fixture_root() -> PathBuf {
    let executable = std::env::current_exe().expect("locate test executable");
    let binary_directory = executable.parent().expect("test executable parent");
    ProjectPaths::resolve_existing(binary_directory)
        .expect("resolve test binary directory")
        .operation_path()
        .join("zircon-physical-input-fixtures")
}

fn write_template_project(root: &Path) {
    let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "PhysicalInput")
        .expect("render physical input fixture template");
    for entry in rendered.entries {
        let destination = entry.path.join_to(root);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).expect("create template directory");
        }
        std::fs::write(destination, entry.bytes).expect("write template asset");
    }
    make_template_scene_dynamic(root);
    ProjectPaths::from_root(root)
        .expect("project paths")
        .ensure_derived_layout()
        .expect("project derived layout");
}

fn write_ui_root(root: &Path) {
    let paths = ProjectPaths::from_root(root).expect("project paths");
    let ui_path = root.join("assets/ui/physical_input_ownership.zui");
    std::fs::create_dir_all(ui_path.parent().expect("UI source parent"))
        .expect("create UI asset directory");
    std::fs::write(&ui_path, CAPTURE_SLIDER_VIEW).expect("write capture slider UI");

    let mut manifest = ProjectManifest::load(paths.manifest_path()).expect("load project manifest");
    manifest.ui_roots.push(
        AssetUri::parse("res://ui/physical_input_ownership.zui").expect("physical input UI URI"),
    );
    manifest
        .save(paths.manifest_path())
        .expect("save physical input UI root");
}

fn make_template_scene_dynamic(root: &Path) {
    let scene_path = root.join("assets/scenes/main.scene.toml");
    let source = std::fs::read_to_string(&scene_path).expect("read template scene");
    let mut scene = toml::from_str::<toml::Value>(&source).expect("parse template scene");
    let entities = scene
        .get_mut("entities")
        .and_then(toml::Value::as_array_mut)
        .expect("template scene entities");
    for entity in entities {
        let entity = entity.as_table_mut().expect("template scene entity");
        if entity.contains_key("mobility") {
            entity.insert(
                "mobility".to_owned(),
                toml::Value::String("Dynamic".to_owned()),
            );
        }
    }
    std::fs::write(
        scene_path,
        toml::to_string(&scene).expect("serialize dynamic fixture scene"),
    )
    .expect("write dynamic fixture scene");
}

#[test]
fn runtime220_session_owns_exact_ingress_and_resets_before_core_teardown() {
    use crate::input::InputIngressPhase;

    let fixture = PhysicalInputFixture::create();
    let mut session = runtime_session(&fixture);
    let generation = session.input_attachment.generation();
    let health = session.resolve_input_driver().unwrap().health();
    assert_eq!(health.generation, Some(generation));
    assert_eq!(health.phase, InputIngressPhase::Unattached);
    assert_ok(session.handle_event(mouse_button(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, 24.0, 24.0)));
    assert_eq!(
        session.resolve_input_driver().unwrap().health().phase,
        InputIngressPhase::Receiving
    );
    assert!(session
        .resolve_input_manager()
        .unwrap()
        .button_pressed(&InputButton::MouseLeft));
    session.resolve_input_manager().unwrap().drain_events();

    session.detach_input_ingress().unwrap();
    assert_eq!(
        session.resolve_input_driver().unwrap().health().phase,
        InputIngressPhase::Detached
    );
    assert!(!session
        .resolve_input_manager()
        .unwrap()
        .button_pressed(&InputButton::MouseLeft));
    assert_eq!(
        session.resolve_input_manager().unwrap().drain_events(),
        vec![InputEvent::FocusLost]
    );
    session.detach_input_ingress().unwrap();
    assert_ok(session.handle_event(mouse_button(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, 24.0, 24.0)));
    assert!(!session
        .resolve_input_manager()
        .unwrap()
        .button_pressed(&InputButton::MouseLeft));
    assert!(session
        .resolve_input_manager()
        .unwrap()
        .drain_events()
        .is_empty());
    assert_eq!(
        session.resolve_input_driver().unwrap().health().generation,
        Some(generation)
    );
    drop(session);
    fixture.assert_removable();
}

#[test]
fn runtime220_session_touch_is_atomic_and_capacity_gap_releases_physical_state() {
    use crate::input::{InputIngressLimits, InputIngressPhase, TouchPhase};
    use zircon_runtime_interface::{
        ZR_RUNTIME_EVENT_KIND_TOUCH_V1, ZR_RUNTIME_TOUCH_PHASE_MOVED_V1,
        ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
    };

    let fixture = PhysicalInputFixture::create();
    let mut session = runtime_session(&fixture);
    let mut touch = mouse_button(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, 10.0, 20.0);
    touch.kind = ZR_RUNTIME_EVENT_KIND_TOUCH_V1;
    touch.state = ZR_RUNTIME_TOUCH_PHASE_STARTED_V1;
    touch.pointer_id = 21;
    assert_ok(session.handle_event(touch));
    assert_eq!(
        session.resolve_input_manager().unwrap().drain_events(),
        vec![
            InputEvent::CursorMoved { x: 10.0, y: 20.0 },
            InputEvent::Touch {
                id: 21,
                phase: TouchPhase::Started,
                x: 10.0,
                y: 20.0
            },
        ]
    );
    let old_cursor = session.cursor;
    touch.state = ZR_RUNTIME_TOUCH_PHASE_MOVED_V1;
    touch.x = f32::NAN;
    assert_eq!(
        session.handle_event(touch).status_code(),
        ZrStatusCode::InvalidArgument
    );
    assert_eq!(session.cursor, old_cursor);
    assert!(session
        .resolve_input_manager()
        .unwrap()
        .drain_events()
        .is_empty());

    assert_ok(session.handle_event(mouse_button(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, 24.0, 24.0)));
    assert!(session
        .resolve_input_manager()
        .unwrap()
        .button_pressed(&InputButton::MouseLeft));
    let used = session
        .resolve_input_driver()
        .unwrap()
        .health()
        .frame_events;
    let limit = InputIngressLimits::default().events_per_frame;
    for _ in used..(limit - 1) {
        assert!(session.submit_input_event(InputEvent::CursorEntered));
    }
    session.resolve_input_manager().unwrap().drain_events();
    let old_cursor = session.cursor;
    touch.x = 33.0;
    touch.y = 44.0;
    assert_ok(session.handle_event(touch));
    assert_eq!(
        session.cursor, old_cursor,
        "rejected physical batch must not reach pointer semantics"
    );
    assert_eq!(
        session.resolve_input_manager().unwrap().drain_events(),
        vec![InputEvent::FocusLost]
    );
    assert!(!session
        .resolve_input_manager()
        .unwrap()
        .button_pressed(&InputButton::MouseLeft));
    assert!(session
        .resolve_input_manager()
        .unwrap()
        .frame_snapshot()
        .active_touches
        .is_empty());
    let health = session.resolve_input_driver().unwrap().health();
    assert_eq!(health.phase, InputIngressPhase::Gap);
    assert_eq!(health.gap_resets, 1);
    assert!(!session.submit_input_event(InputEvent::CursorEntered));
    assert!(session
        .resolve_input_manager()
        .unwrap()
        .drain_events()
        .is_empty());
    drop(session);
    fixture.assert_removable();
}

#[test]
fn runtime220_focus_loss_and_detach_end_real_camera_drag() {
    use crate::core::framework::input::InputBatchRejection;
    use crate::input::{InputIngressAdmission, InputIngressRejection};
    use zircon_runtime_interface::{
        ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1, ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1,
        ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1, ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1,
        ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1, ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
    };

    let fixture = PhysicalInputFixture::create();
    let mut session = runtime_session(&fixture);
    let pose = |session: &RuntimeDynamicSession| {
        session
            .level
            .with_world(|scene| scene.world_transform(scene.active_camera()).unwrap())
    };
    for button in [
        ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1,
        ZR_RUNTIME_MOUSE_BUTTON_MIDDLE_V1,
    ] {
        let press = ZrRuntimeEventV1::mouse_button(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            button,
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
            500.0,
            300.0,
        );
        let mut moved = press;
        moved.kind = ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1;
        moved.x = 540.0;
        moved.y = 340.0;
        let before = pose(&session);
        assert_ok(session.handle_event(press));
        assert_ok(session.handle_event(moved));
        assert_ne!(
            pose(&session),
            before,
            "real drag must move the scene camera"
        );

        assert!(matches!(
            session
                .submit_input_event_with_admission(InputEvent::CursorMoved {
                    x: f32::NAN,
                    y: 0.0,
                })
                .unwrap(),
            InputIngressAdmission::Rejected(InputIngressRejection::Batch(
                InputBatchRejection::NonFinite { .. }
            ))
        ));
        let before_rejected_move = pose(&session);
        moved.x += 20.0;
        assert_ok(session.handle_event(moved));
        assert_ne!(
            pose(&session),
            before_rejected_move,
            "ordinary invalid input must not cancel an owned drag"
        );

        let mut lifecycle = press;
        lifecycle.kind = ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1;
        lifecycle.state = ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1;
        assert_ok(session.handle_event(lifecycle));
        let stopped = pose(&session);
        moved.x += 20.0;
        assert_ok(session.handle_event(moved));
        assert_eq!(pose(&session), stopped, "focus loss must end orbit and pan");
        lifecycle.state = ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1;
        assert_ok(session.handle_event(lifecycle));
        assert_ok(session.handle_event(press));
        assert_ok(session.handle_event(moved));
        assert_ne!(
            pose(&session),
            stopped,
            "a fresh press may start a new drag"
        );
        lifecycle.state = ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1;
        assert_ok(session.handle_event(lifecycle));
        lifecycle.state = ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1;
        assert_ok(session.handle_event(lifecycle));
    }
    let mut press = mouse_button(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, 500.0, 300.0);
    press.button = ZR_RUNTIME_MOUSE_BUTTON_RIGHT_V1;
    assert_ok(session.handle_event(press));
    let mut moved = press;
    moved.kind = ZR_RUNTIME_EVENT_KIND_POINTER_MOVED_V1;
    moved.x = 560.0;
    assert_ok(session.handle_event(moved));
    session.detach_input_ingress().unwrap();
    let detached = pose(&session);
    moved.x += 20.0;
    assert_ok(session.handle_event(moved));
    assert_eq!(pose(&session), detached);
    assert!(session.input_ingress_detached);
    assert!(!session.input_consumer_reset_pending);
    drop(session);
    fixture.assert_removable();
}
