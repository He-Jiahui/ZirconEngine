use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::style_selector::select_workbench_tooltip_style;
#[cfg(test)]
#[path = "template_tooltips_tests.rs"]
mod tests;
use super::template_tooltip_glyphs::{
    push_tooltip_arrow, push_tooltip_info_icon, tooltip_arrow_size, tooltip_icon_size,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const TOOLTIP_BUBBLE_WIDTH: f32 = 96.0;
const TOOLTIP_BUBBLE_HEIGHT: f32 = 45.0;
const TOOLTIP_RADIUS: f32 = 4.0;
const TOOLTIP_BORDER_WIDTH: f32 = 1.0;
const TOOLTIP_TEXT_LEFT: f32 = 8.0;
const TOOLTIP_TITLE_TOP: f32 = 7.0;
const TOOLTIP_BODY_TOP: f32 = 23.0;
const TOOLTIP_TITLE_FONT_SIZE: f32 = 12.0;
const TOOLTIP_TITLE_LINE_HEIGHT: f32 = 14.0;
const TOOLTIP_BODY_FONT_SIZE: f32 = 11.0;
const TOOLTIP_BODY_LINE_HEIGHT: f32 = 13.0;
const TOOLTIP_SHADOW_OFFSET_Y: f32 = 8.0;

pub(super) fn push_tooltip_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_tooltip(node) {
        return false;
    }

    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    let style = select_workbench_tooltip_style(node);
    let bubble = tooltip_bubble_rect(node, &rect);
    let arrow_size = tooltip_arrow_size(node);
    let icon_size = tooltip_icon_size(node);

    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: bubble.x,
            y: bubble.y + TOOLTIP_SHADOW_OFFSET_Y,
            width: bubble.width,
            height: bubble.height,
        },
        Some(clip.clone()),
        order,
        Some(style.shadow),
        None,
        0.0,
        TOOLTIP_RADIUS,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        bubble.clone(),
        Some(clip.clone()),
        order + 1,
        Some(style.surface),
        Some(style.border),
        TOOLTIP_BORDER_WIDTH,
        TOOLTIP_RADIUS,
        opacity,
    ));

    push_tooltip_text(
        commands,
        node,
        &bubble,
        clip,
        order + 2,
        style.title,
        style.body,
        opacity,
    );
    push_tooltip_arrow(
        commands,
        &bubble,
        clip,
        order + 3,
        arrow_size,
        style.arrow,
        style.border,
        opacity,
    );
    push_tooltip_info_icon(
        commands,
        node,
        &rect,
        clip,
        order + 4,
        icon_size,
        style.icon,
        opacity,
    );

    true
}

fn is_workbench_tooltip(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str() == "WorkbenchTooltipRoot"
        || node.surface_variant.as_str() == "workbench-tooltip"
        || (node.control_id.as_str().starts_with("Workbench")
            && (node.role.as_str() == "Tooltip" || node.component_role.as_str() == "tooltip"))
}

fn tooltip_bubble_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + (rect.width - TOOLTIP_BUBBLE_WIDTH).max(0.0) * 0.5 + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: TOOLTIP_BUBBLE_WIDTH.min(rect.width.max(1.0)),
        height: TOOLTIP_BUBBLE_HEIGHT,
    }
}

fn push_tooltip_text(
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

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round(),
        height: rect.height.round(),
    }
}
