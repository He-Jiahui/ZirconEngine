use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{dispatch::UiPointerId, event_ui::UiNodeId};

use super::UiSurfaceInputState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSurfacePointerCaptureState {
    pub owner: UiNodeId,
}

impl UiSurfaceInputState {
    pub fn set_pointer_capture_for_id(&mut self, pointer_id: UiPointerId, owner: UiNodeId) {
        self.captured_pointer_id = Some(pointer_id);
        self.pointer_captures
            .insert(pointer_id, UiSurfacePointerCaptureState { owner });
    }

    pub fn pointer_capture_owner(&self, pointer_id: UiPointerId) -> Option<UiNodeId> {
        self.pointer_captures
            .get(&pointer_id)
            .map(|capture| capture.owner)
    }

    pub fn activate_pointer_capture_for_id(&mut self, pointer_id: UiPointerId) -> Option<UiNodeId> {
        let owner = self.pointer_capture_owner(pointer_id)?;
        self.captured_pointer_id = Some(pointer_id);
        Some(owner)
    }

    pub fn has_pointer_capture_for_owner(&self, owner: UiNodeId) -> bool {
        self.pointer_captures
            .values()
            .any(|capture| capture.owner == owner)
    }

    pub fn has_legacy_or_indexed_pointer_capture_for_owner(&self, owner: UiNodeId) -> bool {
        self.has_pointer_capture_for_owner(owner)
            || (self.pointer_captures.is_empty() && self.captured_pointer_id.is_some())
    }

    pub fn active_pointer_capture(&self) -> Option<(UiPointerId, UiNodeId)> {
        self.captured_pointer_id
            .and_then(|pointer_id| {
                self.pointer_capture_owner(pointer_id)
                    .map(|owner| (pointer_id, owner))
            })
            .or_else(|| {
                self.pointer_captures
                    .iter()
                    .next()
                    .map(|(pointer_id, capture)| (*pointer_id, capture.owner))
            })
    }

    pub fn activate_any_pointer_capture(&mut self) -> Option<UiNodeId> {
        let (pointer_id, owner) = self.active_pointer_capture()?;
        self.captured_pointer_id = Some(pointer_id);
        Some(owner)
    }

    pub fn clear_pointer_capture_id_for_owner(
        &mut self,
        pointer_id: UiPointerId,
        owner: UiNodeId,
    ) -> bool {
        if self.pointer_capture_owner(pointer_id) != Some(owner)
            && self.captured_pointer_id != Some(pointer_id)
        {
            return false;
        }
        self.pointer_captures.remove(&pointer_id);
        if self.captured_pointer_id == Some(pointer_id) {
            self.captured_pointer_id = None;
        }
        if !self.has_pointer_capture_for_owner(owner) {
            self.clear_high_precision_for(owner);
        }
        true
    }

    pub fn clear_pointer_captures_for_owner(&mut self, owner: UiNodeId) {
        let pointer_ids = self
            .pointer_captures
            .iter()
            .filter_map(|(pointer_id, capture)| (capture.owner == owner).then_some(*pointer_id))
            .collect::<Vec<_>>();
        for pointer_id in pointer_ids {
            self.pointer_captures.remove(&pointer_id);
        }
        if self
            .captured_pointer_id
            .is_some_and(|pointer_id| self.pointer_capture_owner(pointer_id).is_none())
        {
            self.captured_pointer_id = None;
        }
        self.clear_high_precision_for(owner);
    }

    pub fn restore_pointer_capture(
        &mut self,
        pointer_id: UiPointerId,
        capture: UiSurfacePointerCaptureState,
    ) {
        self.pointer_captures.entry(pointer_id).or_insert(capture);
    }
}
