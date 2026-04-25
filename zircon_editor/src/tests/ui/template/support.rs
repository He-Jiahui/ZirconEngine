pub(super) const EDITOR_HOST_WINDOW_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.workbench.asset"
version = 1
display_name = "Editor Host Window Asset"

[root]
node = "root"

[nodes.root]
kind = "component"
component = "UiHostWindow"
children = [
  { child = "menu_bar_root", mount = "menu_bar" },
  { child = "status_bar_root", mount = "status_bar" },
]

[nodes.menu_bar_root]
kind = "component"
component = "MenuBar"

[nodes.status_bar_root]
kind = "native"
type = "StatusBar"
control_id = "StatusBarRoot"

[components.UiHostWindow]
root = "host_window_root"

[components.UiHostWindow.slots.menu_bar]
required = true

[components.UiHostWindow.slots.status_bar]
required = true

[components.MenuBar]
root = "menu_bar_component_root"

[nodes.host_window_root]
kind = "native"
type = "UiHostWindow"
children = [
  { child = "host_window_menu_bar_slot", mount = "menu_bar" },
  { child = "host_window_status_bar_slot", mount = "status_bar" },
]

[nodes.host_window_menu_bar_slot]
kind = "slot"
slot_name = "menu_bar"

[nodes.host_window_status_bar_slot]
kind = "slot"
slot_name = "status_bar"

[nodes.menu_bar_component_root]
kind = "native"
type = "UiHostToolbar"
children = [
  { child = "open_project" },
  { child = "save_project" },
]

[nodes.open_project]
kind = "native"
type = "UiHostIconButton"
control_id = "OpenProject"
bindings = [{ id = "WorkbenchMenuBar/OpenProject", event = "Click", route = "MenuAction.OpenProject" }]

[nodes.save_project]
kind = "native"
type = "UiHostIconButton"
control_id = "SaveProject"
bindings = [{ id = "WorkbenchMenuBar/SaveProject", event = "Click", route = "MenuAction.SaveProject" }]
"##;
