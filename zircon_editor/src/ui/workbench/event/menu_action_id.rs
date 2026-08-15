use crate::core::editor_event::{ConsoleMessageFilter, ConsoleSourceFilter, MenuAction};
use crate::core::play::PlayKind;

use super::node_kind_id::{node_kind_action_id, node_kind_id};

pub(super) fn menu_action_id(action: &MenuAction) -> String {
    match action {
        MenuAction::OpenProject => "workbench.project.open".to_string(),
        MenuAction::OpenScene => "workbench.scene.open".to_string(),
        MenuAction::CreateScene => "workbench.scene.create".to_string(),
        MenuAction::SaveProject => "workbench.project.save".to_string(),
        MenuAction::CloseProject => "workbench.project.close".to_string(),
        MenuAction::SaveLayout => "workbench.layout.save".to_string(),
        MenuAction::ResetLayout => "workbench.layout.reset".to_string(),
        MenuAction::ClearConsole => "workbench.console.clear".to_string(),
        MenuAction::SetConsoleMessageFilter(filter) => {
            format!("workbench.console.filter.{}", filter.as_str())
        }
        MenuAction::SetConsoleSourceFilter(filter) => {
            format!("workbench.console.source.{}", filter.as_str())
        }
        MenuAction::SelectPlayMode(PlayKind::Play) => "workbench.play_mode.select.play".to_string(),
        MenuAction::SelectPlayMode(PlayKind::Simulate) => {
            "workbench.play_mode.select.simulate".to_string()
        }
        MenuAction::EnterPlayMode => "workbench.play_mode.enter".to_string(),
        MenuAction::ExitPlayMode => "workbench.play_mode.exit".to_string(),
        MenuAction::Undo => "workbench.history.undo".to_string(),
        MenuAction::Redo => "workbench.history.redo".to_string(),
        MenuAction::CreateNode(kind) => {
            format!("workbench.scene.node.create.{}", node_kind_action_id(kind))
        }
        MenuAction::DeleteSelected => "workbench.selection.delete_selected".to_string(),
        MenuAction::OpenView(descriptor_id) => format!("workbench.view.open.{}", descriptor_id.0),
    }
}

pub(super) fn menu_action_control_id(action: &MenuAction) -> String {
    match action {
        MenuAction::OpenProject => "OpenProject".to_string(),
        MenuAction::OpenScene => "OpenScene".to_string(),
        MenuAction::CreateScene => "CreateScene".to_string(),
        MenuAction::SaveProject => "SaveProject".to_string(),
        MenuAction::CloseProject => "CloseProject".to_string(),
        MenuAction::SaveLayout => "SaveLayout".to_string(),
        MenuAction::ResetLayout => "ResetLayout".to_string(),
        MenuAction::ClearConsole => "ClearConsole".to_string(),
        MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::All) => {
            "SetConsoleMessageFilter.All".to_string()
        }
        MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::Info) => {
            "SetConsoleMessageFilter.Info".to_string()
        }
        MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::Warning) => {
            "SetConsoleMessageFilter.Warning".to_string()
        }
        MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::Error) => {
            "SetConsoleMessageFilter.Error".to_string()
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::All) => {
            "SetConsoleSourceFilter.All".to_string()
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Editor) => {
            "SetConsoleSourceFilter.Editor".to_string()
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Runtime) => {
            "SetConsoleSourceFilter.Runtime".to_string()
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Play) => {
            "SetConsoleSourceFilter.Play".to_string()
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Plugin) => {
            "SetConsoleSourceFilter.Plugin".to_string()
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Import) => {
            "SetConsoleSourceFilter.Import".to_string()
        }
        MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::ScriptBuild) => {
            "SetConsoleSourceFilter.ScriptBuild".to_string()
        }
        MenuAction::SelectPlayMode(PlayKind::Play) => "SelectPlayMode.Play".to_string(),
        MenuAction::SelectPlayMode(PlayKind::Simulate) => "SelectPlayMode.Simulate".to_string(),
        MenuAction::EnterPlayMode => "EnterPlayMode".to_string(),
        MenuAction::ExitPlayMode => "ExitPlayMode".to_string(),
        MenuAction::Undo => "Undo".to_string(),
        MenuAction::Redo => "Redo".to_string(),
        MenuAction::CreateNode(kind) => format!("CreateNode.{}", node_kind_id(kind)),
        MenuAction::DeleteSelected => "DeleteSelected".to_string(),
        MenuAction::OpenView(descriptor_id) => format!("OpenView.{}", descriptor_id.0),
    }
}
