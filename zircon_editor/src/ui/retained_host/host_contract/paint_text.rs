mod blend;
mod clip;
mod draw;
mod font;
mod raster;

pub(in crate::ui::retained_host::host_contract) use draw::draw_text_with_size_and_style;
pub(in crate::ui::retained_host::host_contract) use draw::{draw_text, draw_text_with_size};

#[cfg(test)]
#[path = "paint_text_tests.rs"]
mod tests;
