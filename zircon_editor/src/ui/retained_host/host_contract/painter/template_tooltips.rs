use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::style_selector::select_workbench_tooltip_style;
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
const TOOLTIP_ARROW_SIZE: f32 = 8.0;
const TOOLTIP_ICON_SIZE: f32 = 18.0;
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

fn tooltip_arrow_size(node: &TemplatePaneNodeData) -> f32 {
    let size = if node.value_number > 0.0 {
        node.value_number
    } else {
        TOOLTIP_ARROW_SIZE
    };
    size.clamp(4.0, 14.0)
}

fn tooltip_icon_size(node: &TemplatePaneNodeData) -> f32 {
    let size = if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        TOOLTIP_ICON_SIZE
    };
    size.clamp(10.0, 24.0)
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

fn push_tooltip_arrow(
    commands: &mut Vec<HostPaintCommand>,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    arrow_size: f32,
    fill: [u8; 4],
    border: [u8; 4],
    opacity: f32,
) {
    let size = arrow_size.round().max(4.0) as u32;
    let x = bubble.x + bubble.width * 0.5 - size as f32 * 0.5;
    let y = bubble.y + bubble.height - 1.0;
    push_diamond(commands, x, y, size, clip, order, border, opacity);

    let fill_size = size.saturating_sub(2).max(2);
    push_diamond(
        commands,
        x + 1.0,
        y + 1.0,
        fill_size,
        clip,
        order + 1,
        fill,
        opacity,
    );
}

fn push_diamond(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    y: f32,
    size: u32,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let size = size.max(2);
    let center = (size as f32 - 1.0) * 0.5;
    for row in 0..size {
        let distance = (row as f32 - center).abs();
        let row_width = (size as f32 - distance * 2.0).ceil().max(1.0);
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: x + (size as f32 - row_width) * 0.5,
                y: y + row as f32,
                width: row_width,
                height: 1.0,
            },
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn push_tooltip_info_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    icon_size: f32,
    color: [u8; 4],
    opacity: f32,
) {
    let y = if node.layout_content_offset_y > 0.0 {
        rect.y + node.layout_content_offset_y
    } else {
        rect.y + rect.height - icon_size
    };
    let icon = FrameRect {
        x: rect.x + (rect.width - icon_size).max(0.0) * 0.5,
        y,
        width: icon_size,
        height: icon_size,
    };
    commands.push(HostPaintCommand::quad(
        icon.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        1.0,
        icon_size * 0.5,
        opacity,
    ));

    let stem_width = (icon_size * 0.12).max(2.0);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: icon.x + (icon.width - stem_width) * 0.5,
            y: icon.y + icon.height * 0.45,
            width: stem_width,
            height: icon.height * 0.33,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        1.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: icon.x + (icon.width - stem_width) * 0.5,
            y: icon.y + icon.height * 0.25,
            width: stem_width,
            height: stem_width,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        stem_width * 0.5,
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

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::style_selector::{
        WORKBENCH_TOOLTIP_BODY, WORKBENCH_TOOLTIP_BORDER, WORKBENCH_TOOLTIP_ICON,
        WORKBENCH_TOOLTIP_SURFACE,
    };
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::primitives::Color;
    use zircon_runtime_interface::ui::style::UiPainterResolvedState;

    #[test]
    fn workbench_tooltip_paints_declared_bubble_arrow_and_info_icon() {
        let mut node = tooltip_node();
        node.value_number = 8.0;
        node.value_color = Color::from_rgb_u8(23, 28, 32);
        node.label_color = Color::from_rgb_u8(168, 179, 184);
        node.icon_color = Color::from_rgb_u8(37, 156, 167);

        let style = select_workbench_tooltip_style(&node);
        assert_eq!(tooltip_arrow_size(&node), 8.0);
        assert_eq!(style.arrow, WORKBENCH_TOOLTIP_SURFACE);
        assert_eq!(style.body, WORKBENCH_TOOLTIP_BODY);
        assert_eq!(style.icon, WORKBENCH_TOOLTIP_ICON);

        let bytes = paint_template_nodes_for_test(128, 96, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 128, 64, 12), WORKBENCH_TOOLTIP_SURFACE);
        assert_eq!(pixel_at(&bytes, 128, 64, 8), WORKBENCH_TOOLTIP_BORDER);
        assert_eq!(pixel_at(&bytes, 128, 63, 56), WORKBENCH_TOOLTIP_SURFACE);
        assert_eq!(pixel_at(&bytes, 128, 63, 69), WORKBENCH_TOOLTIP_ICON);
        assert!(changed_pixel_count(&bytes, 128, 22, 14, 50, 14) > 0);
        assert!(changed_pixel_count(&bytes, 128, 22, 29, 72, 14) > 0);
    }

    #[test]
    fn workbench_tooltip_style_uses_shared_state_priority() {
        let mut node = tooltip_node();
        node.hovered = true;
        node.focused = true;
        node.pressed = true;
        node.disabled = true;

        let disabled = select_workbench_tooltip_style(&node);
        assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
        assert_ne!(disabled.border, WORKBENCH_TOOLTIP_BORDER);

        node.disabled = false;
        let pressed = select_workbench_tooltip_style(&node);
        assert_eq!(pressed.state, UiPainterResolvedState::Pressed);

        node.pressed = false;
        let focused = select_workbench_tooltip_style(&node);
        assert_eq!(focused.state, UiPainterResolvedState::Focused);
    }

    fn tooltip_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "WorkbenchTooltipRoot".into(),
            role: "Tooltip".into(),
            component_role: "tooltip".into(),
            surface_variant: "workbench-tooltip".into(),
            text: "Tooltip".into(),
            label_text: "This is a tooltip".into(),
            layout_icon_size: 18.0,
            layout_content_offset_y: 56.0,
            frame: TemplateNodeFrameData {
                x: 8.0,
                y: 8.0,
                width: 110.0,
                height: 78.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn changed_pixel_count(
        bytes: &[u8],
        frame_width: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> usize {
        let mut changed = 0;
        for row in y..(y + height) {
            for column in x..(x + width) {
                if pixel_at(bytes, frame_width, column, row) != [0, 0, 0, 255] {
                    changed += 1;
                }
            }
        }
        changed
    }

    fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y as usize * frame_width as usize) + x as usize) * 4;
        [
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]
    }
}
