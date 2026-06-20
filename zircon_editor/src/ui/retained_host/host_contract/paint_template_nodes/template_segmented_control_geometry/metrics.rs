pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_FONT_SIZE: f32 =
    11.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_TEXT_INSET_X:
    f32 = 8.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_TEXT_INSET_Y:
    f32 = 5.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_RADIUS: f32 =
    5.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_GROUP_LABEL_FONT_SIZE:
    f32 = 11.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_GROUP_LABEL_HEIGHT:
    f32 = 14.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TAB_FONT_SIZE: f32 =
    12.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TAB_UNDERLINE_HEIGHT:
    f32 = 2.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_GROUP_LABEL_GAP: f32 = 4.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const SEGMENT_SELECTED_INSET: f32 = 2.0;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const TAB_TEXT_INSET_X: f32 =
    12.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_line_height() -> f32
{
    SEGMENT_FONT_SIZE * 1.2
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_group_label_line_height(
) -> f32 {
    SEGMENT_GROUP_LABEL_FONT_SIZE * 1.2
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_line_height() -> f32 {
    TAB_FONT_SIZE * 1.2
}
