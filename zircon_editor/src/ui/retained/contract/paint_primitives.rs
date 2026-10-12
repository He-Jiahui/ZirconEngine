#[path = "paint_primitives/clip.rs"]
mod clip;
#[path = "paint_primitives/image.rs"]
mod image;
#[path = "paint_primitives/lines.rs"]
mod lines;
#[path = "paint_primitives/pixels.rs"]
mod pixels;
#[path = "paint_primitives/shapes.rs"]
mod shapes;
#[path = "paint_primitives/text_markers.rs"]
mod text_markers;

#[cfg(test)]
#[path = "paint_primitives/tests/cases.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract) use image::{
    draw_gpu_image_clipped_with_resource_key, draw_rgba_image_clipped_with_atlas,
    draw_rgba_image_clipped_with_resource_key, draw_shared_rgba_image_clipped_with_resource_key,
};
pub(in crate::ui::retained_host::host_contract) use lines::draw_separator_line;
pub(in crate::ui::retained_host::host_contract) use shapes::{
    draw_border, draw_border_clipped, draw_rect, draw_rect_clipped, draw_rounded_border_clipped,
    draw_rounded_box_clipped, draw_rounded_rect_clipped,
};
pub(in crate::ui::retained_host::host_contract) use text_markers::{
    draw_label_marker, draw_text_bars, draw_text_bars_clipped,
};
