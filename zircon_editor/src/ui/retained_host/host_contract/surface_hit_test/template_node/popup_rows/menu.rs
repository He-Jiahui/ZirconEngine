use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::frame_geometry::contains_point;
use super::super::super::super::paint_geometry::frame_from_template;
use super::super::super::super::template_popup_layout::menu_item_row_frame;
use super::action_id::normalized_menu_row_action_id;
use super::hit::{template_popup_row_hit, TemplatePopupRowHit};
use super::{next_uniform_popup_row_at_boundary, uniform_popup_row_at_y};

pub(super) fn hit_test_template_menu_rows(
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowHit> {
    let row_count = node.structured_menu_items.row_count();
    if row_count == 0 {
        return None;
    }

    let local = frame_from_template(&node.frame);
    let menu_frame = FrameRect {
        x: origin.x + local.x,
        y: origin.y + local.y,
        width: local.width,
        height: local.height,
    };
    let row_height = menu_item_row_frame(&menu_frame, row_count, 0)?.height;
    let rows_frame = FrameRect {
        height: row_height * row_count as f32,
        ..menu_frame.clone()
    };
    if contains_point(&rows_frame, x, y) {
        let row = uniform_popup_row_at_y(y, menu_frame.y, row_height, row_count)?;
        for candidate in [
            Some(row),
            next_uniform_popup_row_at_boundary(y, menu_frame.y, row_height, row, row_count),
        ]
        .into_iter()
        .flatten()
        {
            let item = node.structured_menu_items.get(candidate)?;
            if item.disabled || item.separator || item.action_id.is_empty() {
                continue;
            }
            let row_frame = menu_item_row_frame(&menu_frame, row_count, candidate)?;
            return Some(TemplatePopupRowHit::Hit(template_popup_row_hit(
                node,
                row_frame,
                "workbench_menu_item",
                normalized_menu_row_action_id(item.action_id.as_str(), item.label.as_str()),
                item.label.clone(),
            )));
        }
    }
    if contains_point(&menu_frame, x, y) {
        return Some(TemplatePopupRowHit::Blocked);
    }
    None
}
