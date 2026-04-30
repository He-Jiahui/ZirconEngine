use serde::{Deserialize, Serialize};
use zircon_runtime::ui::binding::UiBindingCall;

use crate::core::editor_event::InspectorFieldChange;
use crate::ui::binding::{
    AnimationCommand, AssetCommand, DockCommand, DraftCommand, SelectionCommand, ViewportCommand,
    WelcomeCommand,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorUiBindingPayload {
    AnimationCommand(AnimationCommand),
    MenuAction {
        action_id: String,
    },
    EditorOperation {
        operation_id: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        arguments: Vec<zircon_runtime::ui::binding::UiBindingValue>,
    },
    DraftCommand(DraftCommand),
    InspectorFieldBatch {
        subject_path: String,
        changes: Vec<InspectorFieldChange>,
    },
    SelectionCommand(SelectionCommand),
    AssetCommand(AssetCommand),
    WelcomeCommand(WelcomeCommand),
    DockCommand(DockCommand),
    ViewportCommand(ViewportCommand),
    Custom(UiBindingCall),
}
