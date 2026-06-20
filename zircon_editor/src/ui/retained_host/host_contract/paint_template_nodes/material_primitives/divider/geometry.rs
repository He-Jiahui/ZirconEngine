mod align;
mod extents;
mod label_bounds;
mod line_frames;
mod metrics;
mod orientation;
mod text_frame;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use extents::{
    horizontal_divider_extent, vertical_divider_extent,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use label_bounds::{
    horizontal_label_bounds, vertical_label_bounds,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use line_frames::{
    horizontal_line_frame, horizontal_line_y, vertical_line_frame, vertical_line_x,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use orientation::divider_is_vertical;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text_frame::{
    horizontal_label_text_frame, vertical_label_text_frame,
};
