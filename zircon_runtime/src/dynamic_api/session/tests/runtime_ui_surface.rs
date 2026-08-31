use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::ui::accessibility::{
    UiAccessibilityAction, UiAccessibilityActionRequest,
};
use zircon_runtime_interface::{
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    ZR_RUNTIME_TOUCH_PHASE_STARTED_V1, ZrByteSlice, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeEventV1, ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatusCode,
};

use crate::asset::AssetUri;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::dynamic_api::session::project::RuntimeProjectConfig;
use crate::dynamic_api::session::{RuntimeDynamicSession, RuntimeDynamicSessionProfile};
use zircon_runtime_interface::project::{ProjectTemplateId, render_project_template};

const RUNTIME_UI_VIEW: &str = r#"
[asset]
kind = "view"
id = "product.runtime.inventory"
version = 2
display_name = "Runtime Inventory"

[imports]
widgets = ["res://ui/action_button.zui#ActionButton"]

[root]
node = "inventory"

[nodes.inventory]
component = "ActionButton"
control_id = "Inventory"
props = { text = "Inventory" }
layout = { width = { min = 240.0, preferred = 240.0, max = 240.0, stretch = "Fixed" }, height = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" } }
"#;

const RUNTIME_UI_ACTION_BUTTON_COMPONENT: &str = r#"
[asset]
kind = "component"
id = "product.runtime.action_button"
version = 2
display_name = "Runtime Action Button"

[components.ActionButton]
root = "action_button"

[nodes.action_button]
component = "Button"
control_id = "ActionButtonPrototype"
props = { text = "Action" }
layout = { width = { min = 240.0, preferred = 240.0, max = 240.0, stretch = "Fixed" }, height = { min = 48.0, preferred = 48.0, max = 48.0, stretch = "Fixed" } }
"#;

const RUNTIME_UI_OVERLAY_VIEW: &str = r#"
[asset]
kind = "view"
id = "product.runtime.overlay"
version = 2
display_name = "Runtime Overlay"

[root]
node = "overlay"

[nodes.overlay]
component = "Text"
control_id = "Overlay"
props = { text = "Overlay" }
layout = { width = { min = 180.0, preferred = 180.0, max = 180.0, stretch = "Fixed" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" } }
"#;

const RUNTIME_UI_UNUSED_INVALID_VIEW: &str = r#"
[asset]
kind = "view"
id = "product.runtime.unused_invalid"
version = 2
display_name = "Unused Invalid UI"

[imports]
widgets = ["res://ui/missing_component.zui#MissingComponent"]

[root]
node = "unused"

[nodes.unused]
component = "Label"
props = { text = "Unused" }
"#;

const RUNTIME_UI_PRODUCT_VIEW: &str = r#"
[asset]
kind = "view"
id = "product.runtime.play_surface"
version = 2
display_name = "Runtime Play Surface"

[root]
node = "runtime_root"

[nodes.inventory_action]
component = "Button"
control_id = "InventoryAction"
props = { text = "Inventory" }
layout = { width = { min = 112.0, preferred = 128.0, max = 160.0, stretch = "Fixed" }, height = { min = 44.0, preferred = 44.0, max = 44.0, stretch = "Fixed" } }
events = [{ id = "RuntimePlay/Inventory", event = "Click", route = "runtime.play.inventory" }]

[nodes.map_action]
component = "Button"
control_id = "MapAction"
props = { text = "Map" }
layout = { width = { min = 88.0, preferred = 96.0, max = 120.0, stretch = "Fixed" }, height = { min = 44.0, preferred = 44.0, max = 44.0, stretch = "Fixed" } }
events = [{ id = "RuntimePlay/Map", event = "Click", route = "runtime.play.map" }]

[nodes.action_bar]
component = "HorizontalBox"
control_id = "ActionBar"
layout = { container = { kind = "HorizontalBox", gap = 8.0 }, width = { stretch = "Stretch" }, height = { min = 44.0, preferred = 44.0, max = 44.0, stretch = "Fixed" } }
children = [{ node = "inventory_action" }, { node = "map_action" }]

[nodes.inventory_slot_sword]
component = "Button"
control_id = "InventorySlotSword"
props = { text = "Iron Sword" }
layout = { width = { min = 104.0, preferred = 128.0, max = 160.0, stretch = "Stretch" }, height = { min = 64.0, preferred = 72.0, max = 88.0, stretch = "Fixed" } }

[nodes.inventory_slot_potion]
component = "Button"
control_id = "InventorySlotPotion"
props = { text = "Health Potion" }
layout = { width = { min = 104.0, preferred = 128.0, max = 160.0, stretch = "Stretch" }, height = { min = 64.0, preferred = 72.0, max = 88.0, stretch = "Fixed" } }

[nodes.inventory_slot_ore]
component = "Button"
control_id = "InventorySlotOre"
props = { text = "Copper Ore" }
layout = { width = { min = 104.0, preferred = 128.0, max = 160.0, stretch = "Stretch" }, height = { min = 64.0, preferred = 72.0, max = 88.0, stretch = "Fixed" } }

[nodes.inventory_grid]
component = "GridGroup"
control_id = "InventoryGrid"
layout = { container = { kind = "GridBox", columns = 3, rows = 1, column_gap = 8.0, row_gap = 8.0 }, width = { stretch = "Stretch" }, height = { min = 64.0, preferred = 72.0, max = 88.0, stretch = "Fixed" } }
children = [{ node = "inventory_slot_sword" }, { node = "inventory_slot_potion" }, { node = "inventory_slot_ore" }]

[nodes.chat_input]
component = "TextField"
control_id = "PartyChat"
props = { value_text = "", placeholder = "Chat with party", accessibility_label = "Party chat", input_interactive = true, input_focusable = true }
layout = { width = { min = 260.0, preferred = 420.0, max = 640.0, stretch = "Stretch" }, height = { min = 40.0, preferred = 40.0, max = 48.0, stretch = "Fixed" } }
events = [{ id = "RuntimePlay/Chat", event = "Change", route = "runtime.play.chat" }]

[nodes.popup_text]
component = "Label"
control_id = "QuestPopupText"
props = { text = "Quest Accepted" }
layout = { width = { stretch = "Stretch" }, height = { min = 32.0, preferred = 32.0, max = 40.0, stretch = "Fixed" } }

[nodes.popup_dismiss]
component = "Button"
control_id = "QuestPopupDismiss"
props = { text = "Dismiss" }
layout = { width = { min = 96.0, preferred = 112.0, max = 136.0, stretch = "Fixed" }, height = { min = 36.0, preferred = 40.0, max = 44.0, stretch = "Fixed" } }
events = [{ id = "RuntimePlay/QuestDismiss", event = "Click", route = "runtime.play.quest.dismiss" }]

[nodes.quest_popup]
component = "HorizontalBox"
control_id = "QuestPopup"
layout = { container = { kind = "HorizontalBox", gap = 12.0 }, width = { stretch = "Stretch" }, height = { min = 40.0, preferred = 44.0, max = 48.0, stretch = "Fixed" } }
children = [{ node = "popup_text" }, { node = "popup_dismiss" }]

[nodes.touch_action]
component = "Button"
control_id = "TouchAction"
props = { text = "Touch Action" }
layout = { width = { min = 132.0, preferred = 152.0, max = 176.0, stretch = "Fixed" }, height = { min = 48.0, preferred = 52.0, max = 56.0, stretch = "Fixed" } }
events = [{ id = "RuntimePlay/TouchAction", event = "Click", route = "runtime.play.touch_action" }]

[nodes.runtime_root]
component = "VerticalBox"
control_id = "RuntimePlaySurface"
layout = { container = { kind = "VerticalBox", gap = 12.0 }, padding = { left = 16.0, right = 16.0, top = 16.0, bottom = 16.0 }, width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }
children = [{ node = "action_bar" }, { node = "inventory_grid" }, { node = "chat_input" }, { node = "quest_popup" }, { node = "touch_action" }]
"#;

#[test]
fn woc_project_ui_surface_runtime_round_trip() {
    let fixture = RuntimeUiFixture::create("round-trip");
    let mut session = runtime_session(&fixture);

    let submission = session
        .current_ui_submission()
        .expect("build declared project UI render extract")
        .expect("declared project UI root should produce a render extract");
    assert!(
        submission
            .segments()
            .iter()
            .all(|segment| segment.route_tree_id().as_ref() == "zircon-runtime-project-ui")
    );
    assert!(
        submission
            .commands()
            .any(|command| command.text.as_deref() == Some("Inventory"))
    );

    let snapshot = session
        .capture_accessibility_tree(accessibility_request())
        .expect("declared project UI root should produce an accessibility tree");
    assert_eq!(snapshot.tree_id.0, "zircon-runtime-project-ui");
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|node| node.name.as_deref() == Some("Inventory"))
    );

    drop(session);
    fixture.assert_removable();
}

#[test]
fn woc_project_ui_input_render_accessibility_share_surface() {
    let fixture = RuntimeUiFixture::create("shared-surface");
    let mut session = runtime_session(&fixture);
    let snapshot = session
        .capture_accessibility_tree(accessibility_request())
        .expect("capture the project UI accessibility tree");
    let action_target = snapshot
        .nodes
        .iter()
        .find(|node| node.actions.contains(&UiAccessibilityAction::Activate))
        .expect("project Button should expose an activate action")
        .node_id;

    let action = UiAccessibilityActionRequest {
        target: action_target,
        action: UiAccessibilityAction::Activate,
        ..UiAccessibilityActionRequest::default()
    };
    let bytes = serde_json::to_vec(&action).expect("serialize UI accessibility action");
    let status = session.handle_event(ZrRuntimeEventV1::accessibility_action(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        },
    ));
    assert_eq!(
        status.status_code(),
        ZrStatusCode::Ok,
        "the dynamic runtime ABI must route an accessibility action into the live project UI surface"
    );

    let submission = session
        .current_ui_submission()
        .expect("rebuild the acted-on project UI surface")
        .expect("the acted-on project surface should remain renderable");
    assert!(
        submission
            .commands()
            .any(|command| command.node_id == action_target)
    );

    drop(session);
    fixture.assert_removable();
}

#[test]
fn project_runtime_ui_merges_multiple_manifest_roots_without_node_id_collisions() {
    let fixture = RuntimeUiFixture::create("multiple-roots");
    fixture.add_ui_view("overlay.zui", RUNTIME_UI_OVERLAY_VIEW);
    let mut session = runtime_session(&fixture);

    let submission = session
        .current_ui_submission()
        .expect("build project UI roots")
        .expect("project UI roots should render");
    assert!(
        submission
            .commands()
            .any(|command| command.text.as_deref() == Some("Inventory"))
    );
    assert!(
        submission
            .commands()
            .any(|command| command.text.as_deref() == Some("Overlay"))
    );

    let snapshot = session
        .capture_accessibility_tree(accessibility_request())
        .expect("capture all project UI roots");
    assert_eq!(snapshot.roots.len(), 2);
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .map(|node| node.node_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        snapshot.nodes.len(),
        "each root must retain a distinct global node namespace"
    );
    assert!(
        snapshot
            .nodes
            .iter()
            .filter_map(|node| node.node_path.as_ref())
            .any(|path| path.0.starts_with("surface-1:"))
    );

    drop(session);
    fixture.assert_removable();
}

#[test]
fn project_runtime_ui_expands_imported_component_assets_from_project_uris() {
    let fixture = RuntimeUiFixture::create("imported-component");
    let mut session = runtime_session(&fixture);

    let submission = session
        .current_ui_submission()
        .expect("build project UI with imported component")
        .expect("imported component root should render");
    assert!(
        submission
            .commands()
            .any(|command| command.text.as_deref() == Some("Inventory"))
    );

    let snapshot = session
        .capture_accessibility_tree(accessibility_request())
        .expect("capture imported component accessibility tree");
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|node| node.name.as_deref() == Some("Inventory"))
    );

    drop(session);
    fixture.assert_removable();
}

#[test]
fn project_runtime_ui_ignores_unreferenced_assets_with_missing_imports() {
    let fixture = RuntimeUiFixture::create("unreferenced-invalid");
    fixture.write_ui_asset("unused_invalid.zui", RUNTIME_UI_UNUSED_INVALID_VIEW);
    let mut session = runtime_session(&fixture);

    let submission = session
        .current_ui_submission()
        .expect("unreferenced invalid UI must not block a declared runtime root")
        .expect("declared runtime root should still render");
    assert!(
        submission
            .commands()
            .any(|command| command.text.as_deref() == Some("Inventory"))
    );

    drop(session);
    fixture.assert_removable();
}

#[test]
fn project_runtime_ui_product_surface_routes_touch_and_extracts_action_inventory_chat_and_popup() {
    let fixture = RuntimeUiFixture::create("product-surface");
    fixture.add_ui_view("play_surface.zui", RUNTIME_UI_PRODUCT_VIEW);
    let mut session = runtime_session(&fixture);

    for phase in [
        ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
        ZR_RUNTIME_TOUCH_PHASE_ENDED_V1,
    ] {
        let status = session.handle_event(ZrRuntimeEventV1::touch(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            41,
            phase,
            24.0,
            24.0,
        ));
        assert_eq!(
            status.status_code(),
            ZrStatusCode::Ok,
            "touch input must reach the project-owned retained UI surface before gameplay"
        );
    }

    let action_output = session
        .prepare_host_request_output()
        .expect("encode runtime UI action host request");
    session.rollback_host_request_output();
    let retried_action_output = session
        .prepare_host_request_output()
        .expect("retry runtime UI action host request after rollback");
    assert_eq!(retried_action_output, action_output);
    let action_batch: ZrRuntimeHostRequestBatchV1 = serde_json::from_slice(&retried_action_output)
        .expect("decode runtime UI action host request");
    let action = action_batch
        .requests
        .iter()
        .find_map(|request| match request {
            ZrRuntimeHostRequestV1::UiAction(request) => Some(request),
            _ => None,
        })
        .expect("touch activation should leave the dynamic session as a typed UI action");
    assert_eq!(action.target_surface, 1);
    assert_eq!(action.invocation.target_id(), "runtime.play.inventory");
    assert!(action.secure_value.is_none());
    session.commit_host_request_output();

    let submission = session
        .current_ui_submission()
        .expect("build product runtime UI render extract")
        .expect("product runtime UI root should render");
    for label in [
        "Inventory",
        "Iron Sword",
        "Health Potion",
        "Copper Ore",
        "Quest Accepted",
        "Touch Action",
    ] {
        assert!(
            submission
                .commands()
                .any(|command| command.text.as_deref() == Some(label)),
            "product UI render extract should include {label}"
        );
    }

    let snapshot = session
        .capture_accessibility_tree(accessibility_request())
        .expect("capture product runtime UI accessibility tree");
    for label in [
        "Inventory",
        "Iron Sword",
        "Health Potion",
        "Copper Ore",
        "Quest Accepted",
        "Touch Action",
    ] {
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.name.as_deref() == Some(label)),
            "product UI accessibility snapshot should include {label}"
        );
    }
    let focused = snapshot
        .focused
        .and_then(|node_id| snapshot.nodes.iter().find(|node| node.node_id == node_id))
        .expect("touch input should focus the topmost project UI action");
    assert_eq!(focused.name.as_deref(), Some("Inventory"));
    assert!(
        snapshot
            .nodes
            .iter()
            .any(|node| node.name.as_deref() == Some("Party chat")),
        "the same retained product surface should expose the chat text field to accessibility"
    );

    drop(session);
    fixture.assert_removable();
}

fn runtime_session(fixture: &RuntimeUiFixture) -> RuntimeDynamicSession {
    RuntimeDynamicSession::new(
        RuntimeDynamicSessionProfile::Runtime,
        Some(
            RuntimeProjectConfig::from_root(&fixture.root)
                .expect("runtime UI fixture should resolve as a project"),
        ),
    )
    .expect("runtime should load the project-declared UI root")
}

fn accessibility_request() -> ZrRuntimeAccessibilityTreeRequestV1 {
    ZrRuntimeAccessibilityTreeRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrRuntimeViewportSizeV1::new(1280, 720),
        0,
    )
}

struct RuntimeUiFixture {
    root: PathBuf,
}

impl RuntimeUiFixture {
    fn create(label: &str) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after Unix epoch")
            .as_nanos();
        let root = fixture_root().join(format!(
            "runtime-ui-{label}-{}_{}_{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed),
            unique
        ));
        write_template_project(&root);

        let fixture = Self { root };
        fixture.write_ui_asset("action_button.zui", RUNTIME_UI_ACTION_BUTTON_COMPONENT);
        fixture.add_ui_view("inventory.zui", RUNTIME_UI_VIEW);
        fixture
    }

    fn add_ui_view(&self, filename: &str, source: &str) {
        self.write_ui_asset(filename, source);

        let paths = ProjectPaths::from_root(&self.root).expect("project paths");
        let mut manifest =
            ProjectManifest::load(paths.manifest_path()).expect("load template manifest");
        manifest
            .ui_roots
            .push(AssetUri::parse(&format!("res://ui/{filename}")).expect("UI root URI"));
        manifest
            .save(paths.manifest_path())
            .expect("save manifest UI roots");
    }

    fn write_ui_asset(&self, filename: &str, source: &str) {
        let ui_path = self.root.join("assets/ui").join(filename);
        std::fs::create_dir_all(
            ui_path
                .parent()
                .expect("runtime UI source should have a parent directory"),
        )
        .expect("create runtime UI asset directory");
        std::fs::write(&ui_path, source).expect("write runtime UI asset");
    }

    fn assert_removable(&self) {
        std::fs::remove_dir_all(&self.root)
            .expect("runtime UI fixture should not retain file handles");
        assert!(!self.root.exists());
    }
}

impl Drop for RuntimeUiFixture {
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
        .join("zircon-runtime-ui-fixtures")
}

fn write_template_project(root: &Path) {
    let rendered = render_project_template(ProjectTemplateId::RenderableEmpty, "RuntimeUiFixture")
        .expect("render project template");
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
    let source = toml::to_string(&scene).expect("serialize dynamic UI fixture scene");
    std::fs::write(scene_path, source).expect("write dynamic UI fixture scene");
}
