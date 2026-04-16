use serde::{Deserialize, Serialize};
use zircon_ui::UiBindingCall;

use crate::binding::{
    AssetCommand, DockCommand, DraftCommand, SelectionCommand, ViewportCommand, WelcomeCommand,
};

use super::InspectorFieldChange;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorUiBindingPayload {
    PositionOfTrackAndFrame {
        track_path: String,
        frame: u32,
    },
    MenuAction {
        action_id: String,
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
