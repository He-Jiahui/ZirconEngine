mod blend;
mod clip;
mod draw;
mod font;
mod raster;
mod sync;

pub(in crate::ui::retained_host::host_contract) use draw::draw_text;
pub(in crate::ui::retained_host::host_contract) use draw::draw_text_with_size_and_style;
pub(crate) use font::measure_runtime_text_width;
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use font::runtime_font_family_for_face;
pub(in crate::ui::retained_host::host_contract) use font::{
    HostTextFontFace, font_face_for_paint_style, font_request_for_face,
    measure_runtime_text_width_with_style, runtime_text_style_for_face,
};

#[cfg(test)]
#[path = "paint_text_tests.rs"]
mod tests;
