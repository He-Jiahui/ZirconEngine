#[path = "../../divider/geometry/align.rs"]
mod align;
#[path = "../../divider/geometry/extents.rs"]
mod extents;
#[path = "../../divider/geometry/label_bounds.rs"]
mod label_bounds;
#[path = "../../divider/geometry/line_frames.rs"]
mod line_frames;
#[path = "../../divider/geometry/metrics.rs"]
mod metrics;
#[path = "../../divider/geometry/orientation.rs"]
mod orientation;
#[path = "../../divider/geometry/text_frame.rs"]
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
