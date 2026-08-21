use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    component::{UiComponentState, UiValue},
    event_ui::UiNodeId,
    tree::UiTree,
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceComponentStateStore {
    states: BTreeMap<UiNodeId, UiComponentState>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiComponentStatePropertyChange {
    pub(crate) value_changed: bool,
    pub(crate) pseudo_state_changed: bool,
}

impl UiComponentStatePropertyChange {
    pub(crate) const fn any_changed(self) -> bool {
        self.value_changed || self.pseudo_state_changed
    }

    pub(crate) const fn merge(self, other: Self) -> Self {
        Self {
            value_changed: self.value_changed || other.value_changed,
            pseudo_state_changed: self.pseudo_state_changed || other.pseudo_state_changed,
        }
    }
}

impl UiSurfaceComponentStateStore {
    pub fn get(&self, node_id: UiNodeId) -> Option<&UiComponentState> {
        self.states.get(&node_id)
    }

    pub(crate) fn seed_from_tree_metadata(&mut self, tree: &UiTree) {
        for (node_id, node) in &tree.nodes {
            if let Some(metadata) = node.template_metadata.as_ref() {
                if bool_attribute(&metadata.attributes, "hovered")
                    || bool_attribute(&metadata.attributes, "hover")
                {
                    let _ = self.set_hovered(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "focused")
                    || bool_attribute(&metadata.attributes, "focus")
                {
                    let _ = self.set_focused(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "focus_visible")
                    || bool_attribute(&metadata.attributes, "focus-visible")
                    || bool_attribute(&metadata.attributes, "focusVisible")
                {
                    let _ = self.set_focus_visible(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "pressed")
                    || bool_attribute(&metadata.attributes, "active")
                {
                    let _ = self.set_pressed(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "dragging") {
                    let _ = self.set_dragging(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "drop_hovered") {
                    let _ = self.set_drop_hovered(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "active_drag_target") {
                    let _ = self.set_active_drag_target(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "checked") {
                    let _ = self.set_checked(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "disabled") {
                    let _ = self.set_disabled(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "expanded") {
                    let _ = self.set_expanded(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "popup_open")
                    || bool_attribute(&metadata.attributes, "open")
                {
                    let _ = self.set_popup_open(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "selected") {
                    let _ = self.set_selected(*node_id, true);
                }
                if bool_attribute(&metadata.attributes, "loading") {
                    let _ = self.set_loading(*node_id, true);
                }
            }
            if node.state_flags.pressed {
                let _ = self.set_pressed(*node_id, true);
            }
            if node.state_flags.checked {
                let _ = self.set_checked(*node_id, true);
            }
            if !node.state_flags.enabled {
                let _ = self.set_disabled(*node_id, true);
            }
        }
    }

    pub(crate) fn sync_from_property(
        &mut self,
        node_id: UiNodeId,
        property: &str,
        value: &UiValue,
    ) -> UiComponentStatePropertyChange {
        let value_changed = self.set_value(node_id, property.to_string(), value.clone());
        let UiValue::Bool(value) = value else {
            return UiComponentStatePropertyChange {
                value_changed,
                pseudo_state_changed: false,
            };
        };
        let pseudo_state_changed = match property {
            "hover" | "hovered" => self.set_hovered(node_id, *value),
            "focus" | "focused" => self.set_focused(node_id, *value),
            "focus_visible" | "focus-visible" | "focusVisible" => {
                self.set_focus_visible(node_id, *value)
            }
            "pressed" | "active" => self.set_pressed(node_id, *value),
            "dragging" => self.set_dragging(node_id, *value),
            "drop_hovered" => self.set_drop_hovered(node_id, *value),
            "active_drag_target" => self.set_active_drag_target(node_id, *value),
            "checked" => self.set_checked(node_id, *value),
            "enabled" => self.set_disabled(node_id, !*value),
            "disabled" => self.set_disabled(node_id, *value),
            "expanded" => self.set_expanded(node_id, *value),
            "popup_open" | "open" => self.set_popup_open(node_id, *value),
            "selected" => self.set_selected(node_id, *value),
            "loading" => self.set_loading(node_id, *value),
            _ => false,
        };
        UiComponentStatePropertyChange {
            value_changed,
            pseudo_state_changed,
        }
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

    pub(crate) fn set_focus_visible(&mut self, node_id: UiNodeId, focus_visible: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.focus_visible == focus_visible {
            return false;
        }
        state.flags.focus_visible = focus_visible;
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

    pub(crate) fn set_dragging(&mut self, node_id: UiNodeId, dragging: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.dragging == dragging {
            return false;
        }
        state.flags.dragging = dragging;
        true
    }

    pub(crate) fn set_drop_hovered(&mut self, node_id: UiNodeId, drop_hovered: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.drop_hovered == drop_hovered {
            return false;
        }
        state.flags.drop_hovered = drop_hovered;
        true
    }

    pub(crate) fn set_active_drag_target(
        &mut self,
        node_id: UiNodeId,
        active_drag_target: bool,
    ) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.active_drag_target == active_drag_target {
            return false;
        }
        state.flags.active_drag_target = active_drag_target;
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

    pub(crate) fn set_loading(&mut self, node_id: UiNodeId, loading: bool) -> bool {
        let state = self.states.entry(node_id).or_default();
        if state.flags.loading == loading {
            return false;
        }
        state.flags.loading = loading;
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

pub(crate) fn property_may_affect_runtime_pseudo_state(property: &str) -> bool {
    matches!(
        property,
        "checked"
            | "selected"
            | "disabled"
            | "enabled"
            | "pressed"
            | "active"
            | "dragging"
            | "drop_hovered"
            | "active_drag_target"
            | "expanded"
            | "popup_open"
            | "open"
            | "loading"
            | "focus"
            | "focused"
            | "focus_visible"
            | "focus-visible"
            | "focusVisible"
            | "hover"
            | "hovered"
    )
}

fn bool_attribute(values: &std::collections::BTreeMap<String, toml::Value>, key: &str) -> bool {
    values.get(key).and_then(toml::Value::as_bool) == Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_window_numeric_property_change_is_not_a_runtime_pseudo_state_change() {
        let mut states = UiSurfaceComponentStateStore::default();
        let node_id = UiNodeId::new(7);

        let change = states.sync_from_property(node_id, "viewport_start", &UiValue::Int(12));

        assert_eq!(
            change,
            UiComponentStatePropertyChange {
                value_changed: true,
                pseudo_state_changed: false,
            }
        );
        assert_eq!(
            states
                .get(node_id)
                .and_then(|state| state.value("viewport_start")),
            Some(&UiValue::Int(12))
        );
    }

    #[test]
    fn virtual_window_pseudo_state_change_reports_only_real_flag_transitions() {
        let mut states = UiSurfaceComponentStateStore::default();
        let node_id = UiNodeId::new(9);

        let first = states.sync_from_property(node_id, "hovered", &UiValue::Bool(true));
        let alias = states.sync_from_property(node_id, "hover", &UiValue::Bool(true));
        let unchanged = states.sync_from_property(node_id, "hover", &UiValue::Bool(true));

        assert_eq!(
            first,
            UiComponentStatePropertyChange {
                value_changed: true,
                pseudo_state_changed: true,
            }
        );
        assert_eq!(
            alias,
            UiComponentStatePropertyChange {
                value_changed: true,
                pseudo_state_changed: false,
            }
        );
        assert_eq!(unchanged, UiComponentStatePropertyChange::default());
    }
}
