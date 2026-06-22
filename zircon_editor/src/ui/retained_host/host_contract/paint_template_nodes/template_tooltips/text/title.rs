use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::{
    TOOLTIP_TEXT_LEFT, TOOLTIP_TITLE_FONT_SIZE, TOOLTIP_TITLE_LINE_HEIGHT, TOOLTIP_TITLE_TOP,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_tooltip_title(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text_width: f32,
    title_color: [u8; 4],
    opacity: f32,
) {
    let title = tooltip_title(node);
    if title.is_empty() {
        return;
    }

    commands.push(HostPaintCommand::text(
        FrameRect {
            x: bubble.x + TOOLTIP_TEXT_LEFT,
            y: bubble.y + TOOLTIP_TITLE_TOP,
            width: text_width,
            height: TOOLTIP_TITLE_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        title,
        title_color,
        TOOLTIP_TITLE_FONT_SIZE,
        TOOLTIP_TITLE_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn tooltip_title(node: &TemplatePaneNodeData) -> String {
    let text = node.text.as_str().trim();
    if text.is_empty() {
        "Tooltip".to_string()
    } else {
        text.to_string()
    }
}
