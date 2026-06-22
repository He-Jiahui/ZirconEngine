use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

use super::super::super::data::FrameRect;
use super::super::font::fallback_font;

pub(super) fn layout_text_run(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
) -> Layout {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: rect.x,
        y: rect.y + ((rect.height - line_height).max(0.0) * 0.5),
        max_width: Some(rect.width.max(1.0)),
        max_height: Some(rect.height.max(1.0)),
        ..LayoutSettings::default()
    });
    layout.append(&[fallback_font()], &TextStyle::new(text, font_size, 0));
    layout
}
