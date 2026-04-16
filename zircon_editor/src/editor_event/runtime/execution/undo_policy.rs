use crate::editor_event::{EditorEvent, EditorEventUndoPolicy};
use crate::MenuAction;

pub(in crate::editor_event::runtime) fn undo_policy_for_event(
    event: &EditorEvent,
) -> EditorEventUndoPolicy {
    match event {
        EditorEvent::WorkbenchMenu(
            MenuAction::CreateNode(_)
            | MenuAction::DeleteSelected
            | MenuAction::Undo
            | MenuAction::Redo,
        )
        | EditorEvent::Inspector(_)
        | EditorEvent::Viewport(_) => EditorEventUndoPolicy::DelegatedToEditorHistory,
        EditorEvent::Layout(_)
        | EditorEvent::Asset(_)
        | EditorEvent::WorkbenchMenu(
            MenuAction::OpenProject
            | MenuAction::OpenScene
            | MenuAction::CreateScene
            | MenuAction::SaveProject
            | MenuAction::SaveLayout
            | MenuAction::ResetLayout
            | MenuAction::OpenView(_),
        ) => EditorEventUndoPolicy::FutureInverseEvent,
        EditorEvent::Draft(_) | EditorEvent::Selection(_) | EditorEvent::Transient(_) => {
            EditorEventUndoPolicy::NonUndoable
        }
    }
}
