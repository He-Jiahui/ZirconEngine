mod common;
mod metrics;
mod panel;
mod row;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use common::pixel_aligned_rect;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    HEADER_FONT_SIZE, HEADER_LINE_HEIGHT, MARK_RADIUS, MESSAGE_FONT_SIZE, MESSAGE_LINE_HEIGHT,
    PANEL_RADIUS, ROW_RADIUS, TITLE_FONT_SIZE, TITLE_LINE_HEIGHT,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use panel::{
    empty_text_rect, header_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use row::{
    mark_rect, message_rect, row_rect, row_text_width, title_rect,
};
