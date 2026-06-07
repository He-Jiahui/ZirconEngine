use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiPoint};

use super::UiSurfaceInputState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfacePopupState {
    pub popup_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<UiNodeId>,
    pub anchor: Option<UiPoint>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceTooltipState {
    pub tooltip_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<UiNodeId>,
    pub visible: bool,
}

impl UiSurfaceInputState {
    pub fn open_popup(
        &mut self,
        popup_id: String,
        owner: Option<UiNodeId>,
        anchor: Option<UiPoint>,
    ) {
        self.close_popup(popup_id.as_str());
        self.popup_stack.push(UiSurfacePopupState {
            popup_id,
            owner,
            anchor,
        });
    }

    pub fn close_popup(&mut self, popup_id: &str) -> bool {
        let previous_len = self.popup_stack.len();
        self.popup_stack
            .retain(|popup| popup.popup_id.as_str() != popup_id);
        previous_len != self.popup_stack.len()
    }

    pub fn toggle_popup(
        &mut self,
        popup_id: String,
        owner: Option<UiNodeId>,
        anchor: Option<UiPoint>,
    ) {
        if !self.close_popup(popup_id.as_str()) {
            self.open_popup(popup_id, owner, anchor);
        }
    }

    pub fn popup_owner(&self, popup_id: &str) -> Option<UiNodeId> {
        self.popup_stack
            .iter()
            .rev()
            .find(|popup| popup.popup_id.as_str() == popup_id)
            .and_then(|popup| popup.owner)
    }

    pub fn popup_matches(&self, popup_id: &str, owner: Option<UiNodeId>) -> bool {
        self.popup_stack.iter().rev().any(|popup| {
            let owner_matches = match owner {
                Some(owner) => popup.owner == Some(owner),
                None => true,
            };
            popup.popup_id.as_str() == popup_id && owner_matches
        })
    }

    pub fn arm_tooltip(&mut self, tooltip_id: String, owner: Option<UiNodeId>) {
        self.tooltip = Some(UiSurfaceTooltipState {
            tooltip_id,
            owner,
            visible: false,
        });
    }

    pub fn show_tooltip(&mut self, tooltip_id: String, owner: Option<UiNodeId>) {
        self.tooltip = Some(UiSurfaceTooltipState {
            tooltip_id,
            owner,
            visible: true,
        });
    }

    pub fn tooltip_matches(&self, tooltip_id: &str, owner: Option<UiNodeId>) -> bool {
        self.tooltip.as_ref().is_some_and(|tooltip| {
            let owner_matches = match owner {
                Some(owner) => tooltip.owner == Some(owner),
                None => true,
            };
            tooltip.tooltip_id.as_str() == tooltip_id && owner_matches
        })
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

    pub fn tooltip_owner(&self, tooltip_id: &str) -> Option<UiNodeId> {
        self.tooltip
            .as_ref()
            .filter(|tooltip| tooltip.tooltip_id.as_str() == tooltip_id)
            .and_then(|tooltip| tooltip.owner)
    }
}
