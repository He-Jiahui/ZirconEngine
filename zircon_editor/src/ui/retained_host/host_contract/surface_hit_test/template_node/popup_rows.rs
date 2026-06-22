mod action_id;
mod hit;
mod menu;
mod option;

use crate::ui::retained_host::primitives::ModelRc;

use self::menu::hit_test_template_menu_rows;
use self::option::hit_test_template_option_rows;
use super::super::super::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract) use self::hit::TemplatePopupRowHit;

pub(in crate::ui::retained_host::host_contract) fn hit_test_template_popup_rows(
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    for row in (0..nodes.row_count()).rev() {
        let Some(node) = nodes.row_data(row) else {
            continue;
        };
        if !node.popup_open || node.disabled || node.control_id.is_empty() {
            continue;
        }
        if let Some(hit) = hit_test_template_menu_rows(&node, origin, x, y) {
            return Some(hit);
        }
        if let Some(hit) = hit_test_template_option_rows(&node, origin, x, y) {
            return Some(hit);
        }
    }
    None
}
