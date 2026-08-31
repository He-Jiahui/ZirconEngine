mod analog;
mod drag_drop;
mod focus_loss;
mod pointer_capture;
mod pointer_drag;
mod pointer_position;
mod popup_tooltip;
mod text_document_epoch;

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    dispatch::{
        UiComponentEventReport, UiDispatchHostRequest, UiDispatchHostRequestKind,
        UiInputDispatchResult, UiInputMethodRequest, UiPointerId, UiPointerLockPolicy,
    },
    event_ui::UiNodeId,
    layout::UiPoint,
};

pub use analog::{UiSurfaceAnalogControlState, UiSurfaceAnalogNavigationState};
pub use drag_drop::UiSurfaceDragDropState;
use focus_loss::{UiPendingTextFocusLoss, UiPendingTextFocusLossOwners};
pub use pointer_capture::UiSurfacePointerCaptureState;
pub use pointer_drag::UiSurfacePointerDragState;
pub use pointer_position::UiSurfacePointerPositionState;
pub use popup_tooltip::{UiSurfacePopupState, UiSurfaceTooltipState};
use text_document_epoch::UiTextDocumentEpochs;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSurfaceInputState {
    pub pointer_captures: BTreeMap<UiPointerId, UiSurfacePointerCaptureState>,
    pub high_precision_owner: Option<UiNodeId>,
    pub pointer_lock_owner: Option<UiNodeId>,
    pub pointer_lock_policy: Option<UiPointerLockPolicy>,
    pub input_method_owner: Option<UiNodeId>,
    pub input_method_request: Option<UiInputMethodRequest>,
    pub popup_stack: Vec<UiSurfacePopupState>,
    pub popup_anchor_points: BTreeMap<UiNodeId, UiPoint>,
    pub tooltip: Option<UiSurfaceTooltipState>,
    pub drag_drop: Option<UiSurfaceDragDropState>,
    pub pointer_drags: BTreeMap<UiNodeId, UiSurfacePointerDragState>,
    pub last_cursor_point: Option<UiSurfacePointerPositionState>,
    pub analog_controls: BTreeMap<String, UiSurfaceAnalogControlState>,
    pub analog_navigation: BTreeMap<String, UiSurfaceAnalogNavigationState>,
    #[serde(skip)]
    deferred_focus_input_lifecycle: UiDeferredFocusInputLifecycle,
    #[serde(skip)]
    text_document_epochs: UiTextDocumentEpochs,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiDeferredFocusInputLifecycle {
    pub(crate) component_events: Vec<UiComponentEventReport>,
    pub(crate) input_method_requests: Vec<UiInputMethodRequest>,
    focus_loss: UiPendingTextFocusLoss,
}

impl UiSurfaceInputState {
    pub(crate) fn text_document_epoch(&self, owner: UiNodeId) -> Option<u64> {
        self.text_document_epochs.current(owner)
    }

    pub(crate) fn advance_text_document_epoch(&mut self, owner: UiNodeId) -> Option<u64> {
        self.text_document_epochs.advance(owner)
    }

    pub(crate) fn drop_text_document_epoch(&mut self, owner: UiNodeId) {
        self.text_document_epochs.drop_owner(owner);
    }

    pub fn clear_pointer_capture(&mut self) {
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

    pub(crate) fn queue_focus_input_lifecycle(
        &mut self,
        component_event: Option<UiComponentEventReport>,
        input_method_request: UiInputMethodRequest,
    ) {
        if let Some(component_event) = component_event {
            self.deferred_focus_input_lifecycle
                .component_events
                .push(component_event);
        }
        self.deferred_focus_input_lifecycle
            .input_method_requests
            .push(input_method_request);
    }

    pub(crate) fn queue_focus_component_event(
        &mut self,
        component_event: Option<UiComponentEventReport>,
    ) {
        if let Some(component_event) = component_event {
            self.deferred_focus_input_lifecycle
                .component_events
                .push(component_event);
        }
    }

    pub(crate) fn record_focus_loss(&mut self, owner: UiNodeId) {
        self.deferred_focus_input_lifecycle.focus_loss.record(owner);
    }

    pub(crate) fn take_focus_loss_owners(&mut self) -> UiPendingTextFocusLossOwners {
        self.deferred_focus_input_lifecycle.focus_loss.take()
    }

    pub(crate) fn append_deferred_focus_input_lifecycle(
        &mut self,
        result: &mut UiInputDispatchResult,
        effect_index: usize,
    ) {
        result
            .component_events
            .append(&mut self.deferred_focus_input_lifecycle.component_events);
        for request in self
            .deferred_focus_input_lifecycle
            .input_method_requests
            .drain(..)
        {
            result.host_requests.push(UiDispatchHostRequest {
                effect_index,
                request: UiDispatchHostRequestKind::InputMethod(request),
                reason: "focus input-method lifecycle".to_string(),
            });
        }
    }

    pub(crate) fn take_deferred_focus_input_lifecycle(&mut self) -> UiDeferredFocusInputLifecycle {
        std::mem::take(&mut self.deferred_focus_input_lifecycle)
    }
}
