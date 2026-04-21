use crate::core::editor_event::InspectorFieldChange;
use crate::ui::binding::{
    AnimationCommand, AssetCommand, DockCommand, DraftCommand, SelectionCommand, ViewportCommand,
    WelcomeCommand,
};

use super::EditorUiBindingPayload;

impl EditorUiBindingPayload {
    pub fn animation_command(command: AnimationCommand) -> Self {
        Self::AnimationCommand(command)
    }

    pub fn add_animation_key(track_path: impl Into<String>, frame: u32) -> Self {
        Self::AnimationCommand(AnimationCommand::AddKey {
            track_path: track_path.into(),
            frame,
        })
    }

    pub fn menu_action(action_id: impl Into<String>) -> Self {
        Self::MenuAction {
            action_id: action_id.into(),
        }
    }

    pub fn draft_command(command: DraftCommand) -> Self {
        Self::DraftCommand(command)
    }

    pub fn inspector_field_batch(
        subject_path: impl Into<String>,
        changes: impl Into<Vec<InspectorFieldChange>>,
    ) -> Self {
        Self::InspectorFieldBatch {
            subject_path: subject_path.into(),
            changes: changes.into(),
        }
    }

    pub fn dock_command(command: DockCommand) -> Self {
        Self::DockCommand(command)
    }

    pub fn selection_command(command: SelectionCommand) -> Self {
        Self::SelectionCommand(command)
    }

    pub fn asset_command(command: AssetCommand) -> Self {
        Self::AssetCommand(command)
    }

    pub fn welcome_command(command: WelcomeCommand) -> Self {
        Self::WelcomeCommand(command)
    }

    pub fn viewport_command(command: ViewportCommand) -> Self {
        Self::ViewportCommand(command)
    }
}
