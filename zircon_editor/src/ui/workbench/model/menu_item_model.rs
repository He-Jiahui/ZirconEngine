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
    pub children: Vec<MenuItemModel>,
}

impl MenuItemModel {
    pub fn leaf(
        label: impl Into<String>,
        action: Option<MenuAction>,
        binding: EditorUiBinding,
        operation_path: Option<EditorOperationPath>,
        shortcut: Option<String>,
        enabled: bool,
    ) -> Self {
        Self {
            label: label.into(),
            action,
            binding,
            operation_path,
            shortcut,
            enabled,
            children: Vec::new(),
        }
    }

    pub fn branch(label: impl Into<String>, children: Vec<MenuItemModel>) -> Self {
        Self {
            label: label.into(),
            action: None,
            binding: EditorUiBinding::new(
                "WorkbenchMenuBar",
                "",
                crate::ui::binding::EditorUiEventKind::Click,
                crate::ui::binding::EditorUiBindingPayload::menu_action(""),
            ),
            operation_path: None,
            shortcut: None,
            enabled: children.iter().any(|child| child.enabled),
            children,
        }
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
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
        "editor.build_export_desktop" => Some("View.BuildExport.Open"),
        "editor.prefab" => Some("View.Prefab.Open"),
        "editor.asset_browser" => Some("View.AssetBrowser.Open"),
        _ => None,
    }
}
