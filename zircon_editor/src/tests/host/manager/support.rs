use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};

use crate::ui::host::module::{self, module_descriptor};
use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DocumentNode, MainHostPageLayout,
    MainPageId, TabStackLayout, WorkbenchLayout,
};

pub(super) fn unique_temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

pub(super) fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

pub(super) fn write_ui_asset(path: impl AsRef<Path>, source: &str) {
    crate::tests::support::write_test_ui_asset(path, source).unwrap();
}

pub(super) const SIMPLE_UI_LAYOUT_ASSET: &str = r#"
[asset]
kind = "layout"
id = "editor.tests.asset"
version = 1
display_name = "Test UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "status" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "Status"
props = { text = "Ready" }
"#;

pub(super) const STYLE_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.style"
version = 1
display_name = "Styled UI Asset"

[tokens]
accent = "#4488ff"
panel_gap = 12

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }
style_overrides = { self = { text = { color = "#ffffff" } } }

[[stylesheets]]
id = "local"

[[stylesheets.rules]]
selector = ".primary"
set = { self.background = { color = "#224488" } }

[[stylesheets.rules]]
selector = ".primary:hover"
set = { self.text = { color = "#ffeeaa" } }
"##;

pub(super) const DETACH_THEME_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.detach_theme"
version = 1
display_name = "Detach Theme UI Asset"

[imports]
styles = ["res://ui/theme/shared_theme.ui.toml"]

[tokens]
accent = "#4488ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "#SaveButton"
set = { self = { text = "Local Save" } }
"##;

pub(super) const IMPORTED_THEME_COLLISION_ASSET: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
accent = "#223344"
panel = "$accent"

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

pub(super) const MOCK_PREVIEW_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.mock_preview"
version = 1
display_name = "Mock Preview UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", checked = false, count = 3, mode = "Full", icon = "asset://ui/icons/save.png", items = ["Save", "Publish"], metadata = { state = "Ready", enabled = true }, text_expr = "=preview.save_label" }
"##;

pub(super) const DEEP_MOCK_PREVIEW_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.deep_mock_preview"
version = 1
display_name = "Deep Mock Preview UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { context = { dialog = { title = "Ready", steps = [{ label = "Plan" }, { label = "Dirty" }] } } }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", text_expr = "=StatusLabel.context.dialog.steps[1].label" }
"##;

pub(super) const TREE_REPARENT_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.tree_reparent"
version = 1
display_name = "Tree Reparent UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "group_a" }, { child = "loose" }, { child = "group_b" }]

[nodes.group_a]
kind = "native"
type = "VerticalBox"
control_id = "GroupA"
children = [{ child = "nested_a" }]

[nodes.nested_a]
kind = "native"
type = "Label"
control_id = "NestedA"
props = { text = "Nested A" }

[nodes.loose]
kind = "native"
type = "Button"
control_id = "LooseButton"
props = { text = "Loose" }

[nodes.group_b]
kind = "native"
type = "VerticalBox"
control_id = "GroupB"
children = [{ child = "nested_b" }]

[nodes.nested_b]
kind = "native"
type = "Label"
control_id = "NestedB"
props = { text = "Nested B" }
"##;

pub(super) const SEMANTIC_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.semantic"
version = 1
display_name = "Semantic UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Overlay"
control_id = "Root"
children = [{ child = "scroll_panel", slot = { layout = { anchor = { x = 1.0, y = 0.0 }, pivot = { x = 1.0, y = 0.0 }, position = { x = -16.0, y = 12.0 }, z_index = 4 } } }]

[nodes.scroll_panel]
kind = "native"
type = "ScrollableBox"
control_id = "ScrollPanel"
layout = { container = { kind = "ScrollableBox", axis = "Vertical", gap = 6, scrollbar_visibility = "Always", virtualization = { item_extent = 28, overscan = 2 } }, clip = true }
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "ActionButton"
props = { text = "Run" }
"##;

pub(super) const STRUCTURED_BINDING_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.structured_binding"
version = 1
display_name = "Structured Binding UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "MenuAction.SaveProject", action = { route = "MenuAction.SaveProject", payload = { confirm = true, mode = "full" } } }]
"##;

pub(super) const CONTEXTUAL_BINDING_SUGGESTION_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.contextual_binding_suggestion"
version = 1
display_name = "Contextual Binding Suggestion UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "Route.Form.ValueChanged" }]
"##;

pub(super) fn env_lock() -> &'static Mutex<()> {
    crate::tests::support::env_lock()
}

pub(super) fn editor_runtime_with_config_path(path: &Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    runtime
}

pub(super) fn empty_layout_with_page(page_id: &str) -> WorkbenchLayout {
    let page_id = MainPageId::new(page_id);
    WorkbenchLayout {
        active_main_page: page_id.clone(),
        main_pages: vec![MainHostPageLayout::WorkbenchPage {
            id: page_id,
            title: "Workbench".to_string(),
            document_workspace: DocumentNode::Tabs(TabStackLayout {
                tabs: Vec::new(),
                active_tab: None,
            }),
        }],
        drawers: ActivityDrawerSlot::ALL
            .into_iter()
            .map(|slot| {
                (
                    slot,
                    ActivityDrawerLayout {
                        slot,
                        tab_stack: TabStackLayout::default(),
                        active_view: None,
                        mode: ActivityDrawerMode::Pinned,
                        extent: if matches!(
                            slot,
                            ActivityDrawerSlot::BottomLeft | ActivityDrawerSlot::BottomRight
                        ) {
                            200.0
                        } else {
                            260.0
                        },
                        visible: true,
                    },
                )
            })
            .collect(),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    }
}

pub(super) fn slot_value<'a>(
    slot: &'a std::collections::BTreeMap<String, toml::Value>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let value = slot.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let toml::Value::Table(table) = value else {
        return None;
    };
    slot_table_value(table, rest)
}

pub(super) fn layout_value<'a>(
    layout: Option<&'a std::collections::BTreeMap<String, toml::Value>>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let layout = layout?;
    slot_value(layout, path)
}

pub(super) fn slot_table_value<'a>(
    table: &'a toml::map::Map<String, toml::Value>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let value = table.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let toml::Value::Table(child) = value else {
        return None;
    };
    slot_table_value(child, rest)
}
