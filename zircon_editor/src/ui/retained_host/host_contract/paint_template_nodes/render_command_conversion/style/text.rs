use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextAlign};

use super::super::super::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn aligned_text_x(
    frame: &FrameRect,
    text: &str,
    style: &UiResolvedStyle,
) -> f32 {
    let estimated_width = text.chars().count() as f32 * (style.font_size.max(1.0) * 0.5);
    match style.text_align {
        UiTextAlign::Left => frame.x,
        UiTextAlign::Center => frame.x + (frame.width - estimated_width).max(0.0) * 0.5,
        UiTextAlign::Right => frame.x + (frame.width - estimated_width).max(0.0),
    }
}
