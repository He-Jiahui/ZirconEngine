use super::EditorCommandDescriptor;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorCommandContext {
    pub project_open: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub selection_present: bool,
    pub play_mode_active: bool,
}

impl EditorCommandContext {
    pub fn is_enabled(self, descriptor: &EditorCommandDescriptor) -> bool {
        descriptor.enablement().is_enabled(self)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EditorCommandEnablement {
    #[default]
    Always,
    ProjectOpen,
    UndoAvailable,
    RedoAvailable,
    SelectionPresent,
    CanEnterPlayMode,
    CanExitPlayMode,
}

impl EditorCommandEnablement {
    pub fn is_enabled(self, context: EditorCommandContext) -> bool {
        match self {
            Self::Always => true,
            Self::ProjectOpen => context.project_open,
            Self::UndoAvailable => context.can_undo,
            Self::RedoAvailable => context.can_redo,
            Self::SelectionPresent => context.selection_present,
            Self::CanEnterPlayMode => context.project_open && !context.play_mode_active,
            Self::CanExitPlayMode => context.play_mode_active,
        }
    }

    pub fn route(self) -> Option<&'static str> {
        match self {
            Self::Always => None,
            Self::ProjectOpen => Some("project.open"),
            Self::UndoAvailable => Some("history.can_undo"),
            Self::RedoAvailable => Some("history.can_redo"),
            Self::SelectionPresent => Some("selection.present"),
            Self::CanEnterPlayMode => Some("runtime.play_mode.can_enter"),
            Self::CanExitPlayMode => Some("runtime.play_mode.active"),
        }
    }
}
