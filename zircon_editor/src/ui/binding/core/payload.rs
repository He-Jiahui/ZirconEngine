use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::binding::{UiBindingCall, UiBindingValue};

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
        arguments: Vec<UiBindingValue>,
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
