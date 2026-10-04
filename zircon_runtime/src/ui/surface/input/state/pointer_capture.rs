use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerId, event_ui::UiNodeId, surface::UiPointerButton,
};

use super::UiSurfaceInputState;

/// 单个 pointer 的捕获 owner；surface 路由临时选择一个 owner，但各 pointer 的权限独立保留。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSurfacePointerCaptureState {
    pub owner: UiNodeId,
    /// None preserves capture acquired without a pointer-button event.
    #[serde(default)]
    pub(crate) button: Option<UiPointerButton>,
}

impl UiSurfaceInputState {
    pub(crate) fn can_capture_pointer_for_button(
        &self,
        pointer_id: UiPointerId,
        owner: UiNodeId,
        button: Option<UiPointerButton>,
    ) -> bool {
        self.pointer_captures
            .get(&pointer_id)
            .is_none_or(|capture| {
                capture.owner == owner || capture.button.is_none() || capture.button == button
            })
    }

    pub fn set_pointer_capture_for_id(&mut self, pointer_id: UiPointerId, owner: UiNodeId) {
        self.set_pointer_capture_for_button(pointer_id, owner, None);
    }

    pub(crate) fn set_pointer_capture_for_button(
        &mut self,
        pointer_id: UiPointerId,
        owner: UiNodeId,
        button: Option<UiPointerButton>,
    ) {
        match self.pointer_captures.get_mut(&pointer_id) {
            Some(capture) if capture.owner == owner => {
                if capture.button.is_none() {
                    capture.button = button;
                }
            }
            _ => {
                self.pointer_captures
                    .insert(pointer_id, UiSurfacePointerCaptureState { owner, button });
            }
        }
    }

    pub(crate) fn pointer_capture_matches(
        &self,
        pointer_id: UiPointerId,
        owner: UiNodeId,
        button: Option<UiPointerButton>,
    ) -> bool {
        self.pointer_captures
            .get(&pointer_id)
            .filter(|capture| capture.owner == owner)
            .is_none_or(|capture| capture.button.is_none() || capture.button == button)
    }

    pub fn pointer_capture_owner(&self, pointer_id: UiPointerId) -> Option<UiNodeId> {
        self.pointer_captures
            .get(&pointer_id)
            .map(|capture| capture.owner)
    }

    pub fn activate_pointer_capture_for_id(&self, pointer_id: UiPointerId) -> Option<UiNodeId> {
        self.pointer_capture_owner(pointer_id)
    }

    pub fn has_pointer_capture_for_owner(&self, owner: UiNodeId) -> bool {
        self.pointer_captures
            .values()
            .any(|capture| capture.owner == owner)
    }

    pub fn active_pointer_capture(&self) -> Option<(UiPointerId, UiNodeId)> {
        self.pointer_captures
            .iter()
            .next()
            .map(|(pointer_id, capture)| (*pointer_id, capture.owner))
    }

    pub fn activate_any_pointer_capture(&self) -> Option<UiNodeId> {
        self.active_pointer_capture()
            .map(|(_pointer_id, owner)| owner)
    }

    /// 仅释放匹配的 pointer/owner；该 owner 最后一份捕获释放时同时关闭高精度模式。
    pub fn clear_pointer_capture_id_for_owner(
        &mut self,
        pointer_id: UiPointerId,
        owner: UiNodeId,
    ) -> bool {
        if self.pointer_capture_owner(pointer_id) != Some(owner) {
            return false;
        }
        self.pointer_captures.remove(&pointer_id);
        if !self.has_pointer_capture_for_owner(owner) {
            self.clear_high_precision_for(owner);
        }
        true
    }

    pub fn clear_pointer_captures_for_owner(&mut self, owner: UiNodeId) {
        self.pointer_captures
            .retain(|_, capture| capture.owner != owner);
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
