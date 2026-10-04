use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerId, event_ui::UiNodeId, surface::UiPointerButton,
};

use super::UiSurfaceInputState;

/// The press that may activate a node is separate from the pointer's capture owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct UiSurfacePointerPressState {
    pub(crate) owner: UiNodeId,
    pub(crate) button: Option<UiPointerButton>,
}

impl UiSurfaceInputState {
    pub(crate) fn routed_pointer_id(&self) -> UiPointerId {
        self.pointer_route_id.unwrap_or_default()
    }

    pub(crate) fn pointer_press_owner(&self, pointer_id: UiPointerId) -> Option<UiNodeId> {
        self.pointer_presses
            .get(&pointer_id)
            .map(|press| press.owner)
    }

    pub(crate) fn pointer_press_matches(
        &self,
        pointer_id: UiPointerId,
        owner: UiNodeId,
        button: Option<UiPointerButton>,
    ) -> bool {
        self.pointer_presses
            .get(&pointer_id)
            .filter(|press| press.owner == owner)
            .is_none_or(|press| press.button.is_none() || press.button == button)
    }

    pub(crate) fn record_pointer_press(
        &mut self,
        owner: Option<UiNodeId>,
        button: Option<UiPointerButton>,
    ) {
        let pointer_id = self.routed_pointer_id();
        if let Some(owner) = owner {
            self.pointer_presses
                .insert(pointer_id, UiSurfacePointerPressState { owner, button });
        } else {
            self.pointer_presses.remove(&pointer_id);
        }
    }

    pub(crate) fn clear_pointer_press(&mut self, pointer_id: UiPointerId) {
        self.pointer_presses.remove(&pointer_id);
    }

    pub(crate) fn clear_pointer_presses_for_owner(&mut self, owner: UiNodeId) {
        self.pointer_presses.retain(|_, press| press.owner != owner);
    }

    pub(crate) fn any_pointer_press_owner(&self) -> Option<UiNodeId> {
        self.pointer_presses
            .values()
            .next()
            .map(|press| press.owner)
    }

    pub(crate) fn has_primary_pointer_press(&self, owner: UiNodeId) -> bool {
        self.pointer_presses
            .values()
            .any(|press| press.owner == owner && press.button == Some(UiPointerButton::Primary))
    }
}
