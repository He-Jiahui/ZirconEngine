use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::frame_geometry::contains_point;
use super::super::super::super::paint_geometry::frame_from_template;
use super::super::super::super::template_popup_layout::{menu_item_row_at_y, menu_item_row_frame};
use super::super::TemplateNodePointerMoveKind;
use super::hit::TemplatePopupRowTarget;

pub(super) fn hit_test_template_menu_row_target<'a>(
    node: &'a TemplatePaneNodeData,
    origin: &FrameRect,
    x: f32,
    y: f32,
) -> Option<TemplatePopupRowTarget<'a>> {
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
    if let Some(row) = menu_item_row_at_y(node, &menu_frame, row_count, y) {
        let item = node.structured_menu_items.get(row)?;
        if !item.disabled && !item.separator && !item.action_id.is_empty() {
            let row_frame = menu_item_row_frame(node, &menu_frame, row_count, row)?;
            if contains_point(&row_frame, x, y) {
                return Some(TemplatePopupRowTarget::Hit {
                    kind: TemplateNodePointerMoveKind::MenuItem,
                    action_id: item.action_id.as_str(),
                    value_text: item.label.as_str(),
                    frame: row_frame,
                });
            }
        }
    }
    if contains_point(&menu_frame, x, y) {
        return Some(TemplatePopupRowTarget::Blocked);
    }
    None
}
