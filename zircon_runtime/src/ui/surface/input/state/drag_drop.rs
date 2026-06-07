use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    component::UiDragPayload,
    dispatch::{UiDragSessionId, UiPointerId},
    event_ui::UiNodeId,
    layout::UiPoint,
};

use super::UiSurfaceInputState;

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

impl UiSurfaceInputState {
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

    pub fn drag_drop_matches(
        &self,
        pointer_id: UiPointerId,
        session_id: Option<UiDragSessionId>,
    ) -> bool {
        self.drag_drop
            .as_ref()
            .is_some_and(|drag| validate_drag_owner(drag, pointer_id, session_id).is_ok())
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
