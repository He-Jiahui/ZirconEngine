use super::super::super::data::FrameRect;
use super::super::template_tree_row_geometry::tree_action_button_rect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_tree_row_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_contains(
    row: &FrameRect,
    child: &FrameRect,
) -> bool {
    has_paintable_tree_row_extent(row)
        && child.x.is_finite()
        && child.y.is_finite()
        && child.width.is_finite()
        && child.height.is_finite()
        && child.width > 0.0
        && child.height > 0.0
        && child.x >= row.x
        && child.y >= row.y
        && child.x + child.width <= row.x + row.width
        && child.y + child.height <= row.y + row.height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tree_row_has_action_space(
    rect: &FrameRect,
) -> bool {
    tree_row_contains(rect, &tree_action_button_rect(rect, 1))
}
