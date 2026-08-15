use crate::core::editor_event::{
    ConsoleMessageFilter, ConsoleSourceFilter, MenuAction, ViewDescriptorId,
};
use crate::core::play::PlayKind;

use super::node_kind_from_id::{node_kind_from_control_id, node_kind_from_id};

pub(super) fn menu_action_from_id(action_id: &str) -> Option<MenuAction> {
    let action_id = action_id
        .strip_prefix("menu_action.")
        .or_else(|| action_id.strip_prefix("MenuAction."))
        .unwrap_or(action_id);

    match action_id {
        "workbench.project.open" => Some(MenuAction::OpenProject),
        "workbench.scene.open" => Some(MenuAction::OpenScene),
        "workbench.scene.create" => Some(MenuAction::CreateScene),
        "workbench.project.save" => Some(MenuAction::SaveProject),
        "workbench.project.close" => Some(MenuAction::CloseProject),
        "workbench.layout.save" => Some(MenuAction::SaveLayout),
        "workbench.layout.reset" => Some(MenuAction::ResetLayout),
        "workbench.console.clear" => Some(MenuAction::ClearConsole),
        "workbench.console.filter.all" | "SetConsoleMessageFilter.All" => Some(
            MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::All),
        ),
        "workbench.console.filter.info" | "SetConsoleMessageFilter.Info" => Some(
            MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::Info),
        ),
        "workbench.console.filter.warning" | "SetConsoleMessageFilter.Warning" => Some(
            MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::Warning),
        ),
        "workbench.console.filter.error" | "SetConsoleMessageFilter.Error" => Some(
            MenuAction::SetConsoleMessageFilter(ConsoleMessageFilter::Error),
        ),
        "workbench.console.source.all" | "SetConsoleSourceFilter.All" => {
            Some(MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::All))
        }
        "workbench.console.source.editor" | "SetConsoleSourceFilter.Editor" => Some(
            MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Editor),
        ),
        "workbench.console.source.runtime" | "SetConsoleSourceFilter.Runtime" => Some(
            MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Runtime),
        ),
        "workbench.console.source.play" | "SetConsoleSourceFilter.Play" => Some(
            MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Play),
        ),
        "workbench.console.source.plugin" | "SetConsoleSourceFilter.Plugin" => Some(
            MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Plugin),
        ),
        "workbench.console.source.import" | "SetConsoleSourceFilter.Import" => Some(
            MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::Import),
        ),
        "workbench.console.source.script_build" | "SetConsoleSourceFilter.ScriptBuild" => Some(
            MenuAction::SetConsoleSourceFilter(ConsoleSourceFilter::ScriptBuild),
        ),
        "workbench.play_mode.select.play" => Some(MenuAction::SelectPlayMode(PlayKind::Play)),
        "workbench.play_mode.select.simulate" => {
            Some(MenuAction::SelectPlayMode(PlayKind::Simulate))
        }
        "workbench.play_mode.enter" => Some(MenuAction::EnterPlayMode),
        "workbench.play_mode.exit" => Some(MenuAction::ExitPlayMode),
        "workbench.history.undo" => Some(MenuAction::Undo),
        "workbench.history.redo" => Some(MenuAction::Redo),
        "workbench.selection.delete_selected" => Some(MenuAction::DeleteSelected),
        "OpenProject" => Some(MenuAction::OpenProject),
        "OpenScene" => Some(MenuAction::OpenScene),
        "CreateScene" => Some(MenuAction::CreateScene),
        "SaveProject" => Some(MenuAction::SaveProject),
        "CloseProject" => Some(MenuAction::CloseProject),
        "SaveLayout" => Some(MenuAction::SaveLayout),
        "ResetLayout" => Some(MenuAction::ResetLayout),
        "ClearConsole" => Some(MenuAction::ClearConsole),
        "SelectPlayMode.Play" => Some(MenuAction::SelectPlayMode(PlayKind::Play)),
        "SelectPlayMode.Simulate" => Some(MenuAction::SelectPlayMode(PlayKind::Simulate)),
        "EnterPlayMode" => Some(MenuAction::EnterPlayMode),
        "ExitPlayMode" => Some(MenuAction::ExitPlayMode),
        "Undo" => Some(MenuAction::Undo),
        "Redo" => Some(MenuAction::Redo),
        "DeleteSelected" => Some(MenuAction::DeleteSelected),
        _ => {
            if let Some(kind) = action_id.strip_prefix("workbench.scene.node.create.") {
                return node_kind_from_id(kind)
                    .or_else(|| node_kind_from_control_id(kind))
                    .map(MenuAction::CreateNode);
            }
            if let Some(kind) = action_id.strip_prefix("CreateNode.") {
                return node_kind_from_control_id(kind)
                    .or_else(|| node_kind_from_id(kind))
                    .map(MenuAction::CreateNode);
            }
            if let Some(descriptor_id) = action_id.strip_prefix("workbench.view.open.") {
                return Some(MenuAction::OpenView(ViewDescriptorId::new(descriptor_id)));
            }
            if let Some(descriptor_id) = action_id.strip_prefix("OpenView.") {
                return Some(MenuAction::OpenView(ViewDescriptorId::new(descriptor_id)));
            }
            None
        }
    }
}
