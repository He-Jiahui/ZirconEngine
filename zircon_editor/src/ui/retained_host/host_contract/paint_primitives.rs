mod clip;
mod image;
mod lines;
mod pixels;
mod shapes;
mod text_markers;

#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract) use image::{
    draw_gpu_image_clipped_with_resource_key, draw_rgba_image_clipped_with_atlas,
    draw_rgba_image_clipped_with_resource_key, draw_shared_rgba_image_clipped_with_resource_key,
};
pub(in crate::ui::retained_host::host_contract) use lines::draw_separator_line;
pub(in crate::ui::retained_host::host_contract) use shapes::{
    draw_border, draw_border_clipped, draw_rect, draw_rect_clipped, draw_rounded_border_clipped,
    draw_rounded_rect_clipped,
};
pub(in crate::ui::retained_host::host_contract) use text_markers::{
    draw_label_marker, draw_text_bars, draw_text_bars_clipped,
};
