use crate::ui::retained_host::host_contract::paint_theme::{
    HostControlMetrics, current_host_metrics,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchSliderMetrics
{
    pub track_height: f32,
    pub track_radius: f32,
    pub thumb_size: f32,
    pub thumb_halo_size: f32,
    pub horizontal_inset: f32,
    pub label_width: f32,
    pub label_gap: f32,
    pub value_width: f32,
    pub value_gap: f32,
    pub value_min_width: f32,
    pub value_height_pad: f32,
    pub value_min_height: f32,
    pub value_max_height: f32,
    pub value_text_inset_x: f32,
    pub value_radius: f32,
    pub range_value_min_height: f32,
    pub range_value_y_offset: f32,
    pub range_value_height: f32,
    pub tick_offset_y: f32,
    pub tick_width: f32,
    pub tick_height: f32,
    pub font_size: f32,
    pub line_height: f32,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_slider_metrics()
-> WorkbenchSliderMetrics {
    workbench_slider_metrics_from_host(current_host_metrics())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn workbench_slider_metrics_from_host(
    metrics: HostControlMetrics,
) -> WorkbenchSliderMetrics {
    let track_height = (metrics.border_width * 4.0).max(1.0);
    let thumb_size = metrics.gap_m.max(track_height);
    let value_width = metrics.button_pad_x * 3.0 + metrics.gap_m;
    let value_gap = (metrics.gap_l - metrics.border_width * 2.0).max(metrics.gap_s);
    let font_size = metrics.font_body + metrics.border_width;
    let line_height = metrics.line_height(font_size);
    let value_height_pad = (metrics.gap_m - metrics.border_width * 2.0).max(0.0);
    let value_min_height =
        (metrics.row_height - metrics.gap_m + metrics.border_width * 2.0).max(line_height);
    let value_max_height = metrics.row_height.max(value_min_height);
    WorkbenchSliderMetrics {
        track_height,
        track_radius: track_height * 0.5,
        thumb_size,
        thumb_halo_size: thumb_size * 2.0,
        horizontal_inset: metrics.gap_m,
        label_width: metrics.button_pad_x * 4.0 + metrics.border_width * 2.0,
        label_gap: metrics.gap_l,
        value_width,
        value_gap,
        value_min_width: value_width * 3.0,
        value_height_pad,
        value_min_height,
        value_max_height,
        value_text_inset_x: metrics.text_clip_guard,
        value_radius: metrics.radius_control,
        range_value_min_height: metrics.row_height + metrics.gap_l + metrics.gap_m
            - metrics.border_width * 2.0,
        range_value_y_offset: value_gap,
        range_value_height: (metrics.row_height - metrics.gap_s).max(line_height),
        tick_offset_y: metrics.gap_m,
        tick_width: metrics.border_width.max(1.0),
        tick_height: track_height,
        font_size,
        line_height,
    }
}
