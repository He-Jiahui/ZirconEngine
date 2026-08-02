mod frame;
mod pixel_rect;
mod rect_ops;

pub(in crate::ui::retained_host::host_contract) use frame::frame_from_template;
pub(in crate::ui::retained_host::host_contract) use frame::{frame_or, is_visible_frame};
pub(in crate::ui::retained_host::host_contract) use pixel_rect::PixelRect;
pub(in crate::ui::retained_host::host_contract) use rect_ops::{
    bounded_extent, corner_radius_for_frame, inset, intersect, inward_pixel_aligned_rect,
    translated,
};
