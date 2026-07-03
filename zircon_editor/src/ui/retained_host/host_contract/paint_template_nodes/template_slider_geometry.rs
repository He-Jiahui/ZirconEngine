mod labels;
mod metrics;
mod rects;
mod values;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use labels::{
    slider_label, slider_range_min_label, slider_value_label,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    workbench_slider_metrics, workbench_slider_metrics_from_host, WorkbenchSliderMetrics,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rects::{
    centered_rect, pixel_aligned_rect, slider_range_min_value_rect, slider_track_rect,
    slider_value_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use values::{
    slider_fill_span, slider_percent, slider_range_min_percent, slider_thumb_size,
    slider_tick_count,
};
