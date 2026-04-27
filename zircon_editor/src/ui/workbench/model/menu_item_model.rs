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
        MenuAction::Undo => "Edit.History.Undo",
        MenuAction::Redo => "Edit.History.Redo",
        MenuAction::CreateNode(NodeKind::Cube) => "Scene.Node.CreateCube",
        MenuAction::DeleteSelected => "Scene.Node.DeleteSelected",
        MenuAction::OpenView(descriptor_id) if descriptor_id.0 == "editor.scene" => {
            "View.Scene.Open"
        }
        MenuAction::OpenView(descriptor_id) if descriptor_id.0 == "editor.game" => "View.Game.Open",
        _ => return None,
    };
    Some(EditorOperationPath::parse(path).expect("menu operation path is valid"))
}
