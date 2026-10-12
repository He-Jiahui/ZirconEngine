use super::super::super::super::super::{
    data::{FrameRect, TemplatePaneNodeData},
    paint_text::measure_runtime_text_width,
};
use super::metrics::{
    badge_centered_text_y, badge_root_available_text_width, badge_root_font_size,
    badge_root_text_x, badge_text_line_height, badge_text_width,
};
use super::model::BadgeTextFrame;

// 根标签按宿主字宽放入根框内边距；外侧计数层由后续独立绘制命令定位。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn badge_root_text_frame(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    label: &str,
) -> BadgeTextFrame {
    let font_size = badge_root_font_size(node, rect);
    let line_height = badge_text_line_height(font_size, rect);
    let available_width = badge_root_available_text_width(rect);
    let text_width = badge_text_width(
        measure_runtime_text_width(label, font_size),
        available_width,
    );
    BadgeTextFrame {
        rect: FrameRect {
            x: badge_root_text_x(rect),
            y: badge_centered_text_y(rect, line_height),
            width: text_width,
            height: line_height,
        },
        font_size,
        line_height,
    }
}

#[cfg(test)]
#[path = "tests/root_text.rs"]
mod tests;
