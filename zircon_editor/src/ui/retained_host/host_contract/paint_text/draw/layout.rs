use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

use super::super::super::data::FrameRect;
use super::super::font::fallback_font;
use super::ellipsis::ellipsize_single_line;

pub(super) fn layout_text_run(
    rect: &FrameRect,
    text: &str,
    font_size: f32,
    line_height: f32,
) -> Layout {
    // 单行 chrome 标签:超宽时优雅末尾省略(`Scene…`)而非靠 max_width 折行/硬裁字形(`Sce`)。
    let available_width = rect.width.max(1.0);
    let display = ellipsize_single_line(text, font_size, available_width);
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        x: rect.x,
        y: rect.y + ((rect.height - line_height).max(0.0) * 0.5),
        max_height: Some(rect.height.max(1.0)),
        ..LayoutSettings::default()
    });
    layout.append(
        &[fallback_font()],
        &TextStyle::new(display.as_str(), font_size, 0),
    );
    layout
}
