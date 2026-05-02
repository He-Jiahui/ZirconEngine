use crate::core::editor_event::MenuAction;
use crate::core::editor_operation::EditorOperationPath;
use crate::ui::binding::EditorUiBinding;
use zircon_runtime::scene::components::NodeKind;

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItemModel {
    pub label: String,
    pub action: Option<MenuAction>,
    pub binding: EditorUiBinding,
    pub operation_path: Option<EditorOperationPath>,
    pub shortcut: Option<String>,
    pub enabled: bool,
}

pub(crate) fn operation_path_for_menu_action(action: &MenuAction) -> Option<EditorOperationPath> {
    let path = match action {
        MenuAction::OpenProject => "File.Project.Open",
        MenuAction::SaveProject => "File.Project.Save",
        MenuAction::SaveLayout => "Window.Layout.Save",
        MenuAction::ResetLayout => "Window.Layout.Reset",
        MenuAction::EnterPlayMode => "Runtime.PlayMode.Enter",
        MenuAction::ExitPlayMode => "Runtime.PlayMode.Exit",
        MenuAction::Undo => "Edit.History.Undo",
        MenuAction::Redo => "Edit.History.Redo",
        MenuAction::CreateNode(NodeKind::Cube) => "Scene.Node.CreateCube",
        MenuAction::DeleteSelected => "Scene.Node.DeleteSelected",
        MenuAction::OpenView(descriptor_id) => builtin_view_operation_path(&descriptor_id.0)?,
        _ => return None,
    };
    Some(EditorOperationPath::parse(path).expect("menu operation path is valid"))
}

fn builtin_view_operation_path(descriptor_id: &str) -> Option<&'static str> {
    match descriptor_id {
        "editor.project" => Some("View.Project.Open"),
        "editor.hierarchy" => Some("View.Hierarchy.Open"),
        "editor.inspector" => Some("View.Inspector.Open"),
        "editor.scene" => Some("View.Scene.Open"),
        "editor.game" => Some("View.Game.Open"),
        "editor.assets" => Some("View.Assets.Open"),
        "editor.console" => Some("View.Console.Open"),
        "editor.runtime_diagnostics" => Some("View.RuntimeDiagnostics.Open"),
        "editor.module_plugins" => Some("View.PluginManager.Open"),
        "editor.prefab" => Some("View.Prefab.Open"),
        "editor.asset_browser" => Some("View.AssetBrowser.Open"),
        _ => None,
    }
}
