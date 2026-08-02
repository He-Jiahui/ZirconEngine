mod common;
mod indicator;
mod metrics;
mod panel;
mod rows;
mod search;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use common::{
    min_frame_extent, pixel_aligned_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use indicator::{
    match_indicator_radius, match_indicator_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    WorkbenchCommandPaletteMetrics, command_palette_metrics, command_palette_metrics_from_host,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use panel::empty_text_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rows::{
    row_detail_rect, row_label_rect, row_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use search::{
    search_icon_rect, search_rect, search_text_rect,
};
