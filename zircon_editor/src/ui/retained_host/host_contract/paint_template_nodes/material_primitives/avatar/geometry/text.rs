use crate::ui::retained_host::host_contract::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};

use super::metrics::{
    avatar_centered_text_x, avatar_centered_text_y, avatar_font_size, avatar_text_line_height,
    avatar_text_width,
};

// 文字绘制使用运行时字宽而非字符数估计；返回框与字号共同决定居中和裁剪。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_text_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> (FrameRect, f32, f32) {
    let font_size = avatar_font_size(node, rect);
    let line_height = avatar_text_line_height(font_size);
    let text_width = avatar_text_width(measure_runtime_text_width(label, font_size), rect.width);
    (
        FrameRect {
            x: avatar_centered_text_x(rect, text_width),
            y: avatar_centered_text_y(rect, line_height),
            width: text_width,
            height: line_height,
        },
        font_size,
        line_height,
    )
}

#[cfg(test)]
#[path = "tests/text.rs"]
mod tests;
