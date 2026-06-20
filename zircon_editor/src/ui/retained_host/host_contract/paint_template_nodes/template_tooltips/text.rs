use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const TOOLTIP_TEXT_LEFT: f32 = 8.0;
const TOOLTIP_TITLE_TOP: f32 = 7.0;
const TOOLTIP_BODY_TOP: f32 = 23.0;
const TOOLTIP_TITLE_FONT_SIZE: f32 = 12.0;
const TOOLTIP_TITLE_LINE_HEIGHT: f32 = 14.0;
const TOOLTIP_BODY_FONT_SIZE: f32 = 11.0;
const TOOLTIP_BODY_LINE_HEIGHT: f32 = 13.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tooltip_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    title_color: [u8; 4],
    body_color: [u8; 4],
    opacity: f32,
) {
    let text_width = (bubble.width - TOOLTIP_TEXT_LEFT * 2.0).max(1.0);
    let title = tooltip_title(node);
    if !title.is_empty() {
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

    let body = tooltip_body(node);
    if !body.is_empty() {
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: bubble.x + TOOLTIP_TEXT_LEFT,
                y: bubble.y + TOOLTIP_BODY_TOP,
                width: text_width,
                height: TOOLTIP_BODY_LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 1,
            body,
            body_color,
            TOOLTIP_BODY_FONT_SIZE,
            TOOLTIP_BODY_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}

fn tooltip_title(node: &TemplatePaneNodeData) -> String {
    let text = node.text.as_str().trim();
    if text.is_empty() {
        "Tooltip".to_string()
    } else {
        text.to_string()
    }
}

fn tooltip_body(node: &TemplatePaneNodeData) -> String {
    let text = node.label_text.as_str().trim();
    if text.is_empty() {
        "This is a tooltip".to_string()
    } else {
        text.to_string()
    }
}
