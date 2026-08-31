use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    dispatch::UiTransientDismissalTarget, event_ui::UiNodeId, layout::UiPoint,
};

use super::UiSurfaceInputState;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSurfacePopupState {
    pub popup_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<UiNodeId>,
    /// Declarative popup states retain their source tree node for precise
    /// invalidation; imperative pointer popups do not have one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub popup_node: Option<UiNodeId>,
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
    pub(crate) fn set_popup_anchor_point(&mut self, popup_node: UiNodeId, point: UiPoint) {
        self.popup_anchor_points.insert(popup_node, point);
        if let Some(popup) = self
            .popup_stack
            .iter_mut()
            .find(|popup| popup.popup_node == Some(popup_node))
        {
            popup.anchor = Some(point);
        }
    }

    pub(crate) fn popup_anchor_point(&self, popup_node: UiNodeId) -> Option<UiPoint> {
        self.popup_anchor_points.get(&popup_node).copied()
    }

    pub fn open_popup(
        &mut self,
        popup_id: String,
        owner: Option<UiNodeId>,
        anchor: Option<UiPoint>,
    ) {
        self.open_popup_with_node(popup_id, owner, None, anchor);
    }

    pub(crate) fn open_popup_with_node(
        &mut self,
        popup_id: String,
        owner: Option<UiNodeId>,
        popup_node: Option<UiNodeId>,
        anchor: Option<UiPoint>,
    ) {
        self.close_popup(popup_id.as_str());
        self.popup_stack.push(UiSurfacePopupState {
            popup_id,
            owner,
            popup_node,
            anchor,
        });
    }

    /// Refresh an already-open declarative popup without discarding entries above
    /// it in the nested popup stack. When a missing parent is restored before an
    /// existing declarative child, place it before that child; otherwise retain
    /// `open_popup`'s replacement semantics for a genuinely new popup.
    pub(crate) fn synchronize_popup_with_node(
        &mut self,
        popup_id: String,
        owner: Option<UiNodeId>,
        popup_node: UiNodeId,
        anchor: Option<UiPoint>,
        insert_before: Option<UiNodeId>,
    ) {
        if let Some(existing) = self
            .popup_stack
            .iter_mut()
            .find(|popup| popup.popup_node == Some(popup_node))
        {
            existing.popup_id = popup_id;
            existing.owner = owner;
            existing.anchor = anchor;
            return;
        }
        if let Some(index) = insert_before.and_then(|insert_before| {
            self.popup_stack
                .iter()
                .position(|popup| popup.popup_node == Some(insert_before))
        }) {
            self.popup_stack.insert(
                index,
                UiSurfacePopupState {
                    popup_id,
                    owner,
                    popup_node: Some(popup_node),
                    anchor,
                },
            );
            return;
        }
        self.open_popup_with_node(popup_id, owner, Some(popup_node), anchor);
    }

    pub(crate) fn close_popup_with_node(&mut self, popup_node: UiNodeId, popup_id: &str) -> bool {
        self.popup_anchor_points.remove(&popup_node);
        let index = self
            .popup_stack
            .iter()
            .position(|popup| popup.popup_node == Some(popup_node))
            .or_else(|| {
                self.popup_stack
                    .iter()
                    .position(|popup| popup.popup_id.as_str() == popup_id)
            });
        let Some(index) = index else {
            return false;
        };
        for popup in &self.popup_stack[index..] {
            if let Some(popup_node) = popup.popup_node {
                self.popup_anchor_points.remove(&popup_node);
            }
        }
        self.popup_stack.truncate(index);
        true
    }

    pub fn close_popup(&mut self, popup_id: &str) -> bool {
        let Some(index) = self
            .popup_stack
            .iter()
            .position(|popup| popup.popup_id.as_str() == popup_id)
        else {
            return false;
        };
        for popup in &self.popup_stack[index..] {
            if let Some(popup_node) = popup.popup_node {
                self.popup_anchor_points.remove(&popup_node);
            }
        }
        self.popup_stack.truncate(index);
        true
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

    pub fn dismiss_transient_ui(&mut self, target: UiTransientDismissalTarget) -> Option<UiNodeId> {
        let route_owner = self.transient_dismissal_owner(target);
        match target {
            UiTransientDismissalTarget::All => {
                self.popup_stack.clear();
                self.tooltip = None;
            }
            UiTransientDismissalTarget::PopupStack => {
                self.popup_stack.clear();
            }
            UiTransientDismissalTarget::Tooltip => {
                self.tooltip = None;
            }
        }
        route_owner
    }

    fn transient_dismissal_owner(&self, target: UiTransientDismissalTarget) -> Option<UiNodeId> {
        match target {
            UiTransientDismissalTarget::All => self
                .popup_stack
                .iter()
                .rev()
                .find_map(|popup| popup.owner)
                .or_else(|| self.tooltip.as_ref().and_then(|tooltip| tooltip.owner)),
            UiTransientDismissalTarget::PopupStack => {
                self.popup_stack.iter().rev().find_map(|popup| popup.owner)
            }
            UiTransientDismissalTarget::Tooltip => {
                self.tooltip.as_ref().and_then(|tooltip| tooltip.owner)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiPoint};

    use super::UiSurfaceInputState;

    #[test]
    fn closing_parent_popup_also_closes_nested_popup_tail() {
        let mut input = UiSurfaceInputState::default();
        input.open_popup(
            "menu.file".to_string(),
            Some(UiNodeId::new(1)),
            Some(UiPoint::new(8.0, 12.0)),
        );
        input.open_popup(
            "menu.file.recent".to_string(),
            Some(UiNodeId::new(2)),
            Some(UiPoint::new(24.0, 12.0)),
        );
        input.open_popup(
            "menu.file.recent.project".to_string(),
            Some(UiNodeId::new(3)),
            Some(UiPoint::new(40.0, 12.0)),
        );

        assert!(input.close_popup("menu.file.recent"));
        assert_eq!(
            input
                .popup_stack
                .iter()
                .map(|popup| popup.popup_id.as_str())
                .collect::<Vec<_>>(),
            vec!["menu.file"]
        );
    }

    #[test]
    fn closing_unknown_popup_preserves_stack() {
        let mut input = UiSurfaceInputState::default();
        input.open_popup("menu.file".to_string(), Some(UiNodeId::new(1)), None);

        assert!(!input.close_popup("menu.edit"));
        assert_eq!(input.popup_stack.len(), 1);
    }
}
