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
    let input = session
        .resolve_input_manager()
        .expect("runtime input manager must be available");

    assert_ok(session.handle_event(mouse_button(ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, 24.0, 24.0)));
    assert!(input.button_pressed(&InputButton::MouseLeft));
    assert_eq!(
        input.drain_events(),
        vec![InputEvent::ButtonPressed(InputButton::MouseLeft)]
    );

    assert_ok(session.handle_event(mouse_button(
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
        960.0,
        640.0,
    )));

    assert!(
        !input.button_pressed(&InputButton::MouseLeft),
        "UI propagation must not hide the physical release from InputManager"
    );
    assert!(
        input
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
