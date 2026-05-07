use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    component::UiDragPayload,
    dispatch::{UiDragSessionId, UiInputMethodRequest, UiPointerId, UiPointerLockPolicy},
    event_ui::UiNodeId,
    layout::UiPoint,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfacePopupState {
    pub popup_id: String,
    pub anchor: Option<UiPoint>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceTooltipState {
    pub tooltip_id: String,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceDragDropState {
    pub session_id: UiDragSessionId,
    pub source: UiNodeId,
    pub target: UiNodeId,
    pub pointer_id: UiPointerId,
    pub point: Option<UiPoint>,
    pub payload: Option<UiDragPayload>,
    pub accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceAnalogControlState {
    pub value: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSurfaceInputState {
    pub captured_pointer_id: Option<UiPointerId>,
    pub high_precision_owner: Option<UiNodeId>,
    pub pointer_lock_owner: Option<UiNodeId>,
    pub pointer_lock_policy: Option<UiPointerLockPolicy>,
    pub input_method_owner: Option<UiNodeId>,
    pub input_method_request: Option<UiInputMethodRequest>,
    pub popup_stack: Vec<UiSurfacePopupState>,
    pub tooltip: Option<UiSurfaceTooltipState>,
    pub drag_drop: Option<UiSurfaceDragDropState>,
    pub analog_controls: BTreeMap<String, UiSurfaceAnalogControlState>,
}

impl UiSurfaceInputState {
    pub fn clear_pointer_capture(&mut self) {
        self.captured_pointer_id = None;
    }

    pub fn clear_pointer_capture_for(&mut self, owner: UiNodeId) {
        self.clear_pointer_capture();
        self.clear_high_precision_for(owner);
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

    pub fn open_popup(&mut self, popup_id: String, anchor: Option<UiPoint>) {
        self.close_popup(popup_id.as_str());
        self.popup_stack
            .push(UiSurfacePopupState { popup_id, anchor });
    }

    pub fn close_popup(&mut self, popup_id: &str) -> bool {
        let previous_len = self.popup_stack.len();
        self.popup_stack
            .retain(|popup| popup.popup_id.as_str() != popup_id);
        previous_len != self.popup_stack.len()
    }

    pub fn toggle_popup(&mut self, popup_id: String, anchor: Option<UiPoint>) {
        if !self.close_popup(popup_id.as_str()) {
            self.open_popup(popup_id, anchor);
        }
    }

    pub fn arm_tooltip(&mut self, tooltip_id: String) {
        self.tooltip = Some(UiSurfaceTooltipState {
            tooltip_id,
            visible: false,
        });
    }

    pub fn show_tooltip(&mut self, tooltip_id: String) {
        self.tooltip = Some(UiSurfaceTooltipState {
            tooltip_id,
            visible: true,
        });
    }

    pub fn clear_tooltip(&mut self, tooltip_id: &str) {
        if self
            .tooltip
            .as_ref()
            .is_some_and(|tooltip| tooltip.tooltip_id.as_str() == tooltip_id)
        {
            self.tooltip = None;
        }
    }

    pub fn begin_drag_drop(
        &mut self,
        source: UiNodeId,
        target: UiNodeId,
        pointer_id: UiPointerId,
        session_id: Option<UiDragSessionId>,
        point: Option<UiPoint>,
        payload: Option<UiDragPayload>,
    ) -> Result<(), String> {
        if self.drag_drop.is_some() {
            return Err("drag session already active".to_string());
        }
        self.drag_drop = Some(UiSurfaceDragDropState {
            session_id: session_id.unwrap_or_else(|| UiDragSessionId::new(pointer_id.0)),
            source,
            target,
            pointer_id,
            point,
            payload,
            accepted: false,
        });
        Ok(())
    }

    pub fn update_drag_drop(
        &mut self,
        target: UiNodeId,
        pointer_id: UiPointerId,
        session_id: Option<UiDragSessionId>,
        point: Option<UiPoint>,
        payload: Option<UiDragPayload>,
    ) -> Result<(), String> {
        let drag = self
            .drag_drop
            .as_mut()
            .ok_or_else(|| "drag session is not active".to_string())?;
        validate_drag_owner(drag, pointer_id, session_id)?;
        drag.target = target;
        drag.point = point.or(drag.point);
        if payload.is_some() {
            drag.payload = payload;
        }
        Ok(())
    }

    pub fn accept_drag_drop(
        &mut self,
        target: UiNodeId,
        pointer_id: UiPointerId,
        session_id: Option<UiDragSessionId>,
    ) -> Result<(), String> {
        let drag = self
            .drag_drop
            .as_mut()
            .ok_or_else(|| "drag session is not active".to_string())?;
        validate_drag_owner(drag, pointer_id, session_id)?;
        drag.target = target;
        drag.accepted = true;
        Ok(())
    }

    pub fn reject_drag_drop(
        &mut self,
        target: UiNodeId,
        pointer_id: UiPointerId,
        session_id: Option<UiDragSessionId>,
    ) -> Result<(), String> {
        let drag = self
            .drag_drop
            .as_mut()
            .ok_or_else(|| "drag session is not active".to_string())?;
        validate_drag_owner(drag, pointer_id, session_id)?;
        drag.target = target;
        drag.accepted = false;
        Ok(())
    }

    pub fn end_drag_drop(
        &mut self,
        pointer_id: UiPointerId,
        session_id: Option<UiDragSessionId>,
    ) -> Result<Option<UiNodeId>, String> {
        let drag = self
            .drag_drop
            .as_ref()
            .ok_or_else(|| "drag session is not active".to_string())?;
        validate_drag_owner(drag, pointer_id, session_id)?;
        let source = drag.source;
        self.drag_drop = None;
        Ok(Some(source))
    }

    pub fn update_analog_control(&mut self, control: &str, value: f32) -> bool {
        const ANALOG_REPEAT_EPSILON: f32 = 0.001;

        let value = if value.is_finite() { value } else { 0.0 };
        match self.analog_controls.get_mut(control) {
            Some(state) if (state.value - value).abs() <= ANALOG_REPEAT_EPSILON => false,
            Some(state) => {
                state.value = value;
                true
            }
            None => {
                self.analog_controls
                    .insert(control.to_string(), UiSurfaceAnalogControlState { value });
                true
            }
        }
    }
}

fn validate_drag_owner(
    drag: &UiSurfaceDragDropState,
    pointer_id: UiPointerId,
    session_id: Option<UiDragSessionId>,
) -> Result<(), String> {
    if drag.pointer_id != pointer_id {
        return Err("drag pointer owner mismatch".to_string());
    }
    if session_id.is_some_and(|session_id| session_id != drag.session_id) {
        return Err("drag session owner mismatch".to_string());
    }
    Ok(())
}
