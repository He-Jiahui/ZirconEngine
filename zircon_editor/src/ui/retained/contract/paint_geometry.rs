#[path = "paint_geometry/frame.rs"]
mod frame;
#[path = "paint_geometry/pixel_rect.rs"]
mod pixel_rect;
#[path = "paint_geometry/rect_ops.rs"]
mod rect_ops;

pub(in crate::ui::retained_host::host_contract) use frame::frame_from_template;
pub(in crate::ui::retained_host::host_contract) use frame::{frame_or, is_visible_frame};
pub(in crate::ui::retained_host::host_contract) use pixel_rect::PixelRect;
pub(in crate::ui::retained_host::host_contract) use rect_ops::{
    bounded_extent, corner_radius_for_frame, inset, intersect, inward_pixel_aligned_rect,
    translated,
};
