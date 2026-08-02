mod marks;
mod metrics;
mod radio;
mod toggle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use marks::{
    centered_square, frame_is_within, has_paintable_selection_control_extent,
    label_rect_after_mark, leading_mark_rect, selection_label_gap,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::workbench_selection_control_metrics;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    RADIO_DOT_SIZE, TOGGLE_THUMB_SIZE, TOGGLE_TRACK_WIDTH,
    workbench_selection_control_metrics_from_host,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use radio::radio_dot_size;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use toggle::{
    toggle_thumb_rect, toggle_track_rect,
};
