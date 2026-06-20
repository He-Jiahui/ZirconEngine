mod labels;
mod metrics;
mod rects;
mod values;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use labels::{
    slider_label, slider_range_min_label, slider_value_label,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    SLIDER_FONT_SIZE, SLIDER_HORIZONTAL_INSET, SLIDER_LABEL_GAP, SLIDER_LABEL_WIDTH,
    SLIDER_LINE_HEIGHT, SLIDER_THUMB_HALO_SIZE, SLIDER_THUMB_SIZE, SLIDER_TRACK_HEIGHT,
    SLIDER_TRACK_RADIUS, SLIDER_VALUE_GAP, SLIDER_VALUE_WIDTH,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rects::{
    centered_rect, pixel_aligned_rect, slider_range_min_value_rect, slider_track_rect,
    slider_value_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use values::{
    slider_fill_span, slider_percent, slider_range_min_percent, slider_thumb_size,
    slider_tick_count,
};
