mod marks;
mod metrics;
mod radio;
mod toggle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use marks::{
    centered_square, label_rect_after_mark, leading_mark_rect, selection_label_gap,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::{
    RADIO_DOT_SIZE, SELECTION_MARK_INSET_X, SELECTION_TEXT_INSET_Y, TOGGLE_TRACK_WIDTH,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use radio::radio_dot_size;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use toggle::{
    toggle_thumb_rect, toggle_track_rect,
};
