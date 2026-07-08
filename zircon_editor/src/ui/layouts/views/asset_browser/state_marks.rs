use crate::ui::layouts::views::ViewTemplateNodeData;

pub(super) fn mark_toggle_state(
    nodes: &mut [ViewTemplateNodeData],
    control_id: &str,
    active: bool,
) {
    if let Some(node) = nodes.iter_mut().find(|node| node.control_id == control_id) {
        node.selected = active;
        node.focused = false;
        node.surface_variant = if active { "inset".into() } else { "".into() };
        node.text_tone = if active {
            "default".into()
        } else {
            "subtle".into()
        };
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
        node.surface_variant = "".into();
        node.text_tone = if active {
            "default".into()
        } else {
            "subtle".into()
        };
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
