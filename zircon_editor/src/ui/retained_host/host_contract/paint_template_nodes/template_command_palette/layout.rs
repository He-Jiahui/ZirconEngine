mod common;
mod metrics;
mod panel;
mod rows;
mod search;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use common::pixel_aligned_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    FONT_SIZE, LINE_HEIGHT, PANEL_RADIUS, ROW_RADIUS, SEARCH_RADIUS,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use panel::empty_text_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rows::{
    row_label_rect, row_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use search::{
    search_rect, search_text_rect,
};
