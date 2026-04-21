pub(super) use crate::ui::asset_editor::{
    UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
};
pub(super) use crate::ui::control::EditorUiControlService;
pub(super) use crate::ui::template_runtime::{
    EditorUiCompatibilityHarness, EditorUiHostRuntime, SlintUiHostAdapter,
    SlintUiHostComponentKind, SlintUiHostValue, UI_HOST_WINDOW_DOCUMENT_ID,
    WORKBENCH_SHELL_DOCUMENT_ID,
};
pub(super) use toml::Value;
pub(super) use zircon_runtime::ui::template::{UiLegacyTemplateAdapter, UiTemplateLoader};
pub(super) use zircon_runtime::ui::{
    binding::UiEventKind, layout::UiFrame, layout::UiSize, template::UiAssetKind,
    tree::UiInputPolicy,
};

pub(super) const ASSET_WORKBENCH_DOCUMENT_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.workbench.asset"
version = 1
display_name = "Editor Workbench Asset"

[root]
node_id = "root"
kind = "component"
component = "UiHostWindow"
children = [
  { mount = "menu_bar", node = { node_id = "menu_bar_root", kind = "component", component = "MenuBar" } },
  { mount = "status_bar", node = { node_id = "status_bar_root", kind = "native", type = "StatusBar", control_id = "StatusBarRoot" } },
]

[components.UiHostWindow]
style_scope = "closed"

[components.UiHostWindow.root]
node_id = "workbench_shell_root"
kind = "native"
type = "UiHostWindow"
children = [
  { mount = "menu_bar", node = { node_id = "workbench_shell_menu_bar_slot", kind = "slot", slot_name = "menu_bar" } },
  { mount = "status_bar", node = { node_id = "workbench_shell_status_bar_slot", kind = "slot", slot_name = "status_bar" } },
]

[components.UiHostWindow.slots.menu_bar]
required = true

[components.UiHostWindow.slots.status_bar]
required = true

[components.MenuBar]
style_scope = "closed"

[components.MenuBar.root]
node_id = "menu_bar_component_root"
kind = "native"
type = "UiHostToolbar"
children = [
  { node = { node_id = "open_project", kind = "native", type = "UiHostIconButton", control_id = "OpenProject", bindings = [{ id = "WorkbenchMenuBar/OpenProject", event = "Click", route = "MenuAction.OpenProject" }] } },
  { node = { node_id = "save_project", kind = "native", type = "UiHostIconButton", control_id = "SaveProject", bindings = [{ id = "WorkbenchMenuBar/SaveProject", event = "Click", route = "MenuAction.SaveProject" }] } },
]
"##;

pub(super) const SIMPLE_LEGACY_TEMPLATE_TOML: &str = r#"
version = 1

[root]
component = "VerticalBox"
control_id = "LegacyRoot"
children = [
  { component = "Button", control_id = "OpenProject", bindings = [{ id = "Legacy/OpenProject", event = "Click", route = "MenuAction.OpenProject" }], attributes = { text = "Open" } }
]
"#;
