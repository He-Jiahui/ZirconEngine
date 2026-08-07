use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::super::super::data::{HostPaneInteractionStateData, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn apply_template_hover_to_node(
    node: &mut TemplatePaneNodeData,
    interaction: &HostPaneInteractionStateData,
) {
    if interaction.hovered_template_control_id.is_empty()
        || node.control_id.as_str() != interaction.hovered_template_control_id.as_str()
    {
        return;
    }

    node.hovered = true;
    match interaction.hovered_template_dispatch_kind.as_str() {
        "workbench_option" => {
            apply_option_row_hover(node, interaction.hovered_template_value_text.as_str())
        }
        "workbench_menu_item" => {
            apply_menu_row_hover(node, interaction.hovered_template_action_id.as_str())
        }
        _ => {}
    }
}

fn apply_option_row_hover(node: &mut TemplatePaneNodeData, option_id: &str) {
    if option_id.is_empty() || node.structured_options.row_count() == 0 {
        return;
    }
    let mut changed = false;
    let options: Vec<_> = node
        .structured_options
        .iter()
        .cloned()
        .map(|mut option| {
            let hovered = !option.disabled && option.id.as_str() == option_id;
            if option.hovered != hovered || option.focused || option.pressed {
                option.hovered = hovered;
                option.focused = false;
                option.pressed = false;
                changed = true;
            }
            option
        })
        .collect();
    if changed {
        node.structured_options = ModelRc::from(Rc::new(VecModel::from(options)));
    }
}

fn apply_menu_row_hover(node: &mut TemplatePaneNodeData, action_id: &str) {
    if action_id.is_empty() || node.structured_menu_items.row_count() == 0 {
        return;
    }
    let mut changed = false;
    let items: Vec<_> = node
        .structured_menu_items
        .iter()
        .cloned()
        .map(|mut item| {
            let hovered = !item.disabled && !item.separator && item.action_id.as_str() == action_id;
            if item.hovered != hovered || item.focused || item.pressed {
                item.hovered = hovered;
                item.focused = false;
                item.pressed = false;
                changed = true;
            }
            item
        })
        .collect();
    if changed {
        node.structured_menu_items = ModelRc::from(Rc::new(VecModel::from(items)));
    }
}
