use crate::core::editor_event::{EditorEvent, EditorEventUndoPolicy, MenuAction};

pub(crate) fn undo_policy_for_event(event: &EditorEvent) -> EditorEventUndoPolicy {
    match event {
        EditorEvent::WorkbenchMenu(
            MenuAction::CreateNode(_)
            | MenuAction::DeleteSelected
            | MenuAction::Undo
            | MenuAction::Redo,
        )
        | EditorEvent::Inspector(_)
        | EditorEvent::Animation(_)
        | EditorEvent::Hierarchy(_)
        | EditorEvent::Viewport(_) => EditorEventUndoPolicy::DelegatedToTransactionEngine,
        EditorEvent::Layout(_)
        | EditorEvent::Asset(_)
        | EditorEvent::WorkbenchMenu(
            MenuAction::OpenProject
            | MenuAction::OpenScene
            | MenuAction::CreateScene
            | MenuAction::SaveProject
            | MenuAction::CloseProject
            | MenuAction::SaveLayout
            | MenuAction::ResetLayout
            | MenuAction::EnterPlayMode
            | MenuAction::ExitPlayMode
            | MenuAction::OpenView(_),
        ) => EditorEventUndoPolicy::FutureInverseEvent,
        EditorEvent::WorkbenchMenu(
            MenuAction::ClearConsole
            | MenuAction::SetConsoleMessageFilter(_)
            | MenuAction::SetConsoleSourceFilter(_)
            | MenuAction::SelectPlayMode(_),
        )
        | EditorEvent::Draft(_)
        | EditorEvent::Selection(_)
        | EditorEvent::Operation(_)
        | EditorEvent::Transient(_) => EditorEventUndoPolicy::NonUndoable,
    }
}
