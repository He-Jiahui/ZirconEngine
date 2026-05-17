use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    component::{UiComponentState, UiValue},
    event_ui::UiNodeId,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceComponentStateStore {
    states: BTreeMap<UiNodeId, UiComponentState>,
}

impl UiSurfaceComponentStateStore {
    pub fn get(&self, node_id: UiNodeId) -> Option<&UiComponentState> {
        self.states.get(&node_id)
    }

    pub(crate) fn set_hovered(&mut self, node_id: UiNodeId, hovered: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.hovered == hovered {
            return false;
        }
        state.flags.hovered = hovered;
        true
    }

    pub(crate) fn set_focused(&mut self, node_id: UiNodeId, focused: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.focused == focused {
            return false;
        }
        state.flags.focused = focused;
        true
    }

    pub(crate) fn set_pressed(&mut self, node_id: UiNodeId, pressed: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.pressed == pressed {
            return false;
        }
        state.flags.pressed = pressed;
        true
    }

    pub(crate) fn set_checked(&mut self, node_id: UiNodeId, checked: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.checked == checked {
            return false;
        }
        state.flags.checked = checked;
        true
    }

    pub(crate) fn set_disabled(&mut self, node_id: UiNodeId, disabled: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.disabled == disabled {
            return false;
        }
        state.flags.disabled = disabled;
        true
    }

    pub(crate) fn set_expanded(&mut self, node_id: UiNodeId, expanded: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.expanded == expanded {
            return false;
        }
        state.flags.expanded = expanded;
        true
    }

    pub(crate) fn set_popup_open(&mut self, node_id: UiNodeId, popup_open: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.popup_open == popup_open {
            return false;
        }
        state.flags.popup_open = popup_open;
        true
    }

    pub(crate) fn set_selected(&mut self, node_id: UiNodeId, selected: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.selected == selected {
            return false;
        }
        state.flags.selected = selected;
        true
    }

    pub(crate) fn set_value(
        &mut self,
        node_id: UiNodeId,
        property: impl Into<String>,
        value: UiValue,
    ) -> bool {
        let state = self.states.entry(node_id).or_default();
        let property = property.into();
        if state.values.get(&property) == Some(&value) {
            return false;
        }
        // A direct runtime value write supersedes any drag/drop provenance for that property.
        state.reference_sources.remove(&property);
        state.values.insert(property, value);
        true
    }

    pub(crate) fn clear_nodes(&mut self, node_ids: &[UiNodeId]) {
        for node_id in node_ids {
            self.states.remove(node_id);
        }
    }
}
