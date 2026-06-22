use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::super::super::data::{HostPaneInteractionStateData, TemplatePaneNodeData};

pub(super) fn apply_template_row_hover(
    node: &mut TemplatePaneNodeData,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    match interaction.hovered_template_dispatch_kind.as_str() {
        "workbench_option" => {
            apply_option_row_hover(node, interaction.hovered_template_value_text.as_str())
        }
        "workbench_menu_item" => {
            apply_menu_row_hover(node, interaction.hovered_template_action_id.as_str())
        }
        _ => false,
    }
}

fn apply_option_row_hover(node: &mut TemplatePaneNodeData, option_id: &str) -> bool {
    if option_id.is_empty() || node.structured_options.row_count() == 0 {
        return false;
    }
    let mut changed = false;
    let options: Vec<_> = (0..node.structured_options.row_count())
        .filter_map(|row| node.structured_options.row_data(row))
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
    changed
}

fn apply_menu_row_hover(node: &mut TemplatePaneNodeData, action_id: &str) -> bool {
    if action_id.is_empty() || node.structured_menu_items.row_count() == 0 {
        return false;
    }
    let mut changed = false;
    let items: Vec<_> = (0..node.structured_menu_items.row_count())
        .filter_map(|row| node.structured_menu_items.row_data(row))
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
    changed
}
