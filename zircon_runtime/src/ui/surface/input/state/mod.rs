mod analog;
mod drag_drop;
mod pointer_capture;
mod pointer_drag;
mod pointer_position;
mod popup_tooltip;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    dispatch::{UiInputMethodRequest, UiPointerId, UiPointerLockPolicy},
    event_ui::UiNodeId,
};

pub use analog::{UiSurfaceAnalogControlState, UiSurfaceAnalogNavigationState};
pub use drag_drop::UiSurfaceDragDropState;
pub use pointer_capture::UiSurfacePointerCaptureState;
pub use pointer_drag::UiSurfacePointerDragState;
pub use pointer_position::UiSurfacePointerPositionState;
pub use popup_tooltip::{UiSurfacePopupState, UiSurfaceTooltipState};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSurfaceInputState {
    pub captured_pointer_id: Option<UiPointerId>,
    pub pointer_captures: BTreeMap<UiPointerId, UiSurfacePointerCaptureState>,
    pub high_precision_owner: Option<UiNodeId>,
    pub pointer_lock_owner: Option<UiNodeId>,
    pub pointer_lock_policy: Option<UiPointerLockPolicy>,
    pub input_method_owner: Option<UiNodeId>,
    pub input_method_request: Option<UiInputMethodRequest>,
    pub popup_stack: Vec<UiSurfacePopupState>,
    pub tooltip: Option<UiSurfaceTooltipState>,
    pub drag_drop: Option<UiSurfaceDragDropState>,
    pub pointer_drags: BTreeMap<UiNodeId, UiSurfacePointerDragState>,
    pub last_cursor_point: Option<UiSurfacePointerPositionState>,
    pub analog_controls: BTreeMap<String, UiSurfaceAnalogControlState>,
    pub analog_navigation: BTreeMap<String, UiSurfaceAnalogNavigationState>,
}

impl UiSurfaceInputState {
    pub fn clear_pointer_capture(&mut self) {
        self.captured_pointer_id = None;
        self.pointer_captures.clear();
    }

    pub fn clear_pointer_capture_for(&mut self, owner: UiNodeId) {
        self.clear_pointer_captures_for_owner(owner);
    }

    pub fn clear_high_precision_for(&mut self, owner: UiNodeId) {
        if self.high_precision_owner == Some(owner) {
            self.high_precision_owner = None;
        }
    }

    pub fn clear_input_method(&mut self) {
        self.input_method_owner = None;
        self.input_method_request = None;
    }
}
