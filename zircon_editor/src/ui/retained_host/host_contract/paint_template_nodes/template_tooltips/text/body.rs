use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::metrics::{
    TOOLTIP_BODY_FONT_SIZE, TOOLTIP_BODY_LINE_HEIGHT, TOOLTIP_BODY_TOP, TOOLTIP_TEXT_LEFT,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

pub(super) fn push_tooltip_body(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text_width: f32,
    body_color: [u8; 4],
    opacity: f32,
) {
    let body = tooltip_body(node);
    if body.is_empty() {
        return;
    }

    commands.push(HostPaintCommand::text(
        FrameRect {
            x: bubble.x + TOOLTIP_TEXT_LEFT,
            y: bubble.y + TOOLTIP_BODY_TOP,
            width: text_width,
            height: TOOLTIP_BODY_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        body,
        body_color,
        TOOLTIP_BODY_FONT_SIZE,
        TOOLTIP_BODY_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn tooltip_body(node: &TemplatePaneNodeData) -> String {
    let text = node.label_text.as_str().trim();
    if text.is_empty() {
        "This is a tooltip".to_string()
    } else {
        text.to_string()
    }
}
