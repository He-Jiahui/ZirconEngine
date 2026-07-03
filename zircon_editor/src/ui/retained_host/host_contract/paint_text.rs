mod blend;
mod clip;
mod draw;
mod font;
mod raster;
mod sync;

pub(in crate::ui::retained_host::host_contract) use draw::draw_text;
pub(in crate::ui::retained_host::host_contract) use draw::draw_text_with_size_and_style;
pub(in crate::ui::retained_host::host_contract) use font::{
    font_face_for_paint_style, font_request_for_face, measure_runtime_text_width,
    measure_runtime_text_width_with_style, HostTextFontFace,
};

#[cfg(test)]
#[path = "paint_text_tests.rs"]
mod tests;
