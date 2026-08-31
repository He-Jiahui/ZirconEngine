mod border;
mod fill;
mod geometry;
mod span;

pub(in crate::ui::retained_host::host_contract) use border::{
    fill_rect_border_pixels, fill_rounded_border_pixels,
};
pub(in crate::ui::retained_host::host_contract) use fill::{
    fill_rect_pixel_coverage, fill_rounded_box_pixels, fill_rounded_pixel_rect,
};
pub(in crate::ui::retained_host::host_contract) use geometry::{
    clamped_corner_radius, inset_frame, rounded_rect_contains_pixel,
};
pub(in crate::ui::retained_host::host_contract) use span::{fill_pixel_span, write_pixel};

#[cfg(test)]
mod tests;
