use super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn has_paintable_table_row_extent(
    rect: &FrameRect,
) -> bool {
    rect.x.is_finite()
        && rect.y.is_finite()
        && rect.width.is_finite()
        && rect.height.is_finite()
        && rect.width > 0.0
        && rect.height > 0.0
}
