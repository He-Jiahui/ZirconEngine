use zircon_runtime::scene::components::NodeKind;

use crate::core::commands::MenuItemModel;
use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter, MenuAction};
use crate::core::editor_operation::EditorOperationPath;
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::{editor_operation_binding, menu_action_binding};

pub fn menu_item_binding(item: &MenuItemModel) -> EditorUiBinding {
    if let Some(operation_path) = &item.operation_path {
        return editor_operation_binding(operation_path);
    }
    if let Some(action) = &item.action {
        return menu_action_binding(action);
    }
    EditorUiBinding::new(
        "WorkbenchMenuBar",
        "",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action(""),
    )
}

pub(crate) fn operation_path_for_menu_action(action: &MenuAction) -> Option<EditorOperationPath> {
    let path = match action {
        MenuAction::OpenProject => "file.project.open",
        MenuAction::SaveProject => "file.project.save",
        MenuAction::CloseProject => "file.project.close",
        MenuAction::SaveLayout => "window.layout.save",
        MenuAction::ResetLayout => "window.layout.reset",
        MenuAction::ClearConsole => "view.console.clear",
        MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::All) => "view.console.filter.all",
        MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::Info) => {
            "view.console.filter.info"
        }
        MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::Warning) => {
            "view.console.filter.warning"
        }
        MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::Error) => {
            "view.console.filter.error"
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::All) => "view.console.source.all",
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Editor) => {
            "view.console.source.editor"
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Runtime) => {
            "view.console.source.runtime"
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Play) => "view.console.source.play",
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Plugin) => {
            "view.console.source.plugin"
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Import) => {
            "view.console.source.import"
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::ScriptBuild) => {
            "view.console.source.script_build"
        }
        MenuAction::EnterPlayMode => "runtime.play_mode.enter",
        MenuAction::ExitPlayMode => "runtime.play_mode.exit",
        MenuAction::Undo => "edit.history.undo",
        MenuAction::Redo => "edit.history.redo",
        MenuAction::CreateNode(NodeKind::Cube) => "scene.node.create_cube",
        MenuAction::CreateNode(NodeKind::Camera) => "scene.node.create_camera",
        MenuAction::CreateNode(NodeKind::AmbientLight) => "scene.node.create_ambient_light",
        MenuAction::CreateNode(NodeKind::DirectionalLight) => "scene.node.create_directional_light",
        MenuAction::CreateNode(NodeKind::PointLight) => "scene.node.create_point_light",
        MenuAction::CreateNode(NodeKind::RectLight) => "scene.node.create_rect_light",
        MenuAction::CreateNode(NodeKind::SpotLight) => "scene.node.create_spot_light",
        MenuAction::DeleteSelected => "scene.node.delete_selected",
        MenuAction::OpenView(descriptor_id) => builtin_view_operation_path(&descriptor_id.0)?,
        _ => return None,
    };
    Some(EditorOperationPath::parse(path).expect("menu operation path is valid"))
}

fn builtin_view_operation_path(descriptor_id: &str) -> Option<&'static str> {
    match descriptor_id {
        "editor.project" => Some("view.project.open"),
        "editor.hierarchy" => Some("view.hierarchy.open"),
        "editor.inspector" => Some("view.inspector.open"),
        "editor.scene" => Some("view.scene.open"),
        "editor.game" => Some("view.game.open"),
        "editor.assets" => Some("view.assets.open"),
        "editor.console" => Some("view.console.open"),
        "editor.runtime_diagnostics" => Some("view.runtime_diagnostics.open"),
        "editor.performance_timeline" => Some("view.performance_timeline.open"),
        "editor.debug_observatory" => Some("window.debug_observatory.open"),
        "editor.module_plugins" => Some("view.plugin_manager.open"),
        "editor.build_export_desktop" => Some("view.build_export.open"),
        "editor.prefab" => Some("view.prefab.open"),
        "editor.asset_browser" => Some("view.asset_browser.open"),
        "editor.prefab_editor_window" => Some("window.prefab_editor.open"),
        "editor.material_editor_window" => Some("window.material_editor.open"),
        "editor.ui_component_showcase" => Some("window.ui_component_showcase.open"),
        "editor.material_demo_window" => Some("window.material_demo.open"),
        "editor.ui_asset_editor_window" => Some("window.ui_asset_editor.open"),
        "editor.animation_editor_window" => Some("window.animation_editor.open"),
        "editor.asset_browser_window" => Some("window.asset_browser.open"),
        "editor.diagnostics_window" => Some("window.diagnostics.open"),
        _ => None,
    }
}
