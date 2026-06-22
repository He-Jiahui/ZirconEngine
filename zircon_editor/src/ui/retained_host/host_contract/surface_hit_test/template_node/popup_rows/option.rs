use crate::ui::retained_host::primitives::SharedString;

use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::frame_geometry::contains_point;
use super::super::super::super::paint_geometry::frame_from_template;
use super::super::super::super::template_popup_layout::{
    template_option_popup_frame_within, template_option_row_frame_within,
};
use super::hit::{template_popup_row_hit, TemplatePopupRowHit};

pub(super) fn hit_test_template_option_rows(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    let row_count = node.structured_options.row_count();
    if row_count == 0 {
        return None;
    }
    let action_id = option_action_id(node);
    if action_id.is_empty() {
        return None;
    }

    let local = frame_from_template(&node.frame);
    let control_frame = FrameRect {
        x: origin.x + local.x,
        y: origin.y + local.y,
        width: local.width,
        height: local.height,
    };
    let popup_frame = template_option_popup_frame_within(node, &control_frame, row_count, origin)?;
    for row in 0..row_count {
        let option = node.structured_options.row_data(row)?;
        if option.disabled {
            continue;
        }
        let row_frame =
            template_option_row_frame_within(node, &control_frame, row_count, row, origin)?;
        if contains_point(&row_frame, x, y) {
            return Some(TemplatePopupRowHit::Hit(template_popup_row_hit(
                node,
                row_frame,
                "workbench_option",
                action_id,
                option.id,
            )));
        }
    }
    if contains_point(&popup_frame, x, y) {
        return Some(TemplatePopupRowHit::Blocked);
    }
    None
}

fn option_action_id(node: &TemplatePaneNodeData) -> SharedString {
    if node.edit_action_id.is_empty() {
        node.action_id.clone()
    } else {
        node.edit_action_id.clone()
    }
}
