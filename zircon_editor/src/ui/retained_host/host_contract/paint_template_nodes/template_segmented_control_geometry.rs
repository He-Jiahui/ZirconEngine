mod metrics;
mod segmented;
mod tabs;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    segment_font_size, segment_group_label_font_size, segment_group_label_line_height,
    segment_line_height, segment_radius, tab_font_size, tab_line_height,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use segmented::{
    segment_divider_rect, segment_label_rect, segment_rect, segmented_body_rect,
    segmented_group_label_rect, selected_segment_rect, selected_segment_underline_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use tabs::{
    tab_label_rect, tab_paint_rect, tab_underline_rect,
};
