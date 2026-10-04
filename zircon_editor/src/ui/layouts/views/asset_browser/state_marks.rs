use crate::ui::layouts::views::ViewTemplateNodeData;
use crate::ui::retained_host::primitives::SharedString;

pub(super) fn mark_toggle_state(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    active: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = false;
        assign_shared_string_if_changed(
            &mut node.surface_variant,
            if active { "inset" } else { "" },
        );
        assign_shared_string_if_changed(
            &mut node.text_tone,
            if active { "default" } else { "subtle" },
        );
    }
}

pub(super) fn mark_utility_tab_state(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    active: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = false;
        assign_shared_string_if_changed(&mut node.surface_variant, "");
        assign_shared_string_if_changed(
            &mut node.text_tone,
            if active { "default" } else { "subtle" },
        );
    }
}

fn assign_shared_string_if_changed(target: &mut SharedString, value: &str) {
    if target.as_str() != value {
        *target = value.into();
    }
}

pub(super) fn mark_panel_selected(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    selected: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = selected;
        node.focused = false;
    }
}

pub(super) fn mark_panel_group_selected(
    nodes: &mut [ViewTemplateNodeData],
    control_ids: &[&str],
    selected: bool,
) {
    for control_id in control_ids {
        mark_panel_selected(nodes, control_id, selected);
    }
}

#[cfg(test)]
#[path = "tests/state_marks.rs"]
mod tests;
