use crate::ui::retained_host::host_contract::paint_theme::METRICS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_font_size() -> f32
{
    METRICS.font_body
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_text_inset_x(
) -> f32 {
    METRICS.input_pad[0]
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_text_inset_y(
) -> f32 {
    METRICS.segment_text_inset_y
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_radius() -> f32 {
    METRICS.radius_control
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_group_label_font_size(
) -> f32 {
    METRICS.font_body
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_group_label_height(
) -> f32 {
    14.0
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_font_size() -> f32 {
    METRICS.font_body
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_underline_height(
) -> f32 {
    METRICS.tab_underline_height
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_group_label_gap(
) -> f32 {
    METRICS.gap_s
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_selected_inset(
) -> f32 {
    METRICS.segment_selected_inset
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_text_inset_x() -> f32 {
    METRICS.button_pad_x
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_line_height() -> f32
{
    METRICS.line_height(segment_font_size())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segment_group_label_line_height(
) -> f32 {
    METRICS.line_height(segment_group_label_font_size())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn tab_line_height() -> f32 {
    METRICS.line_height(tab_font_size())
}
