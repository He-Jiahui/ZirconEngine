use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::geometry::intersect;
use super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const PREVIEW_SURFACE: [u8; 4] = [21, 48, 53, 255];
const PREVIEW_SURFACE_BLOCKED: [u8; 4] = [72, 32, 36, 255];
const PREVIEW_BORDER: [u8; 4] = [53, 199, 208, 255];
const PREVIEW_BORDER_BLOCKED: [u8; 4] = [239, 112, 102, 255];
const PREVIEW_TEXT: [u8; 4] = [206, 224, 226, 255];
const PREVIEW_RADIUS: f32 = 6.0;
const ICON_LEFT: f32 = 12.0;
const ICON_SIZE: f32 = 18.0;
const TEXT_LEFT_WITH_ICON: f32 = 38.0;
const TEXT_RIGHT_INSET: f32 = 12.0;
const FONT_SIZE: f32 = 12.0;
const LINE_HEIGHT: f32 = 14.4;
const INDICATOR_THICKNESS: f32 = 2.0;

pub(super) fn push_drag_overlay_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_drag_overlay(node) {
        return false;
    }
    if !node.popup_open && !node.dragging {
        return true;
    }

    let preview_rect = preview_frame(node, rect);
    if intersect(&preview_rect, clip).is_none() {
        return true;
    }

    let accent = if node.drop_allowed {
        PREVIEW_BORDER
    } else {
        PREVIEW_BORDER_BLOCKED
    };
    commands.push(HostPaintCommand::quad(
        preview_rect.clone(),
        Some(clip.clone()),
        order,
        Some(if node.drop_allowed {
            PREVIEW_SURFACE
        } else {
            PREVIEW_SURFACE_BLOCKED
        }),
        Some(accent),
        1.0,
        PREVIEW_RADIUS,
        opacity,
    ));

    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: preview_rect.x + ICON_LEFT,
            y: preview_rect.y + (preview_rect.height - ICON_SIZE).max(0.0) * 0.5,
            width: ICON_SIZE,
            height: ICON_SIZE,
        },
        Some(clip.clone()),
        order + 1,
        Some(accent),
        None,
        0.0,
        3.0,
        opacity,
    ));

    if let Some(label) = preview_label(node) {
        let text_left = preview_rect.x + TEXT_LEFT_WITH_ICON;
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: text_left,
                y: preview_rect.y + (preview_rect.height - LINE_HEIGHT).max(0.0) * 0.5,
                width: (preview_rect.x + preview_rect.width - TEXT_RIGHT_INSET - text_left)
                    .max(1.0),
                height: LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 2,
            label,
            PREVIEW_TEXT,
            FONT_SIZE,
            LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    if let Some(indicator) = indicator_frame(node) {
        commands.push(HostPaintCommand::quad(
            indicator,
            Some(clip.clone()),
            order + 3,
            Some(accent),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }

    true
}

fn is_drag_overlay(node: &TemplatePaneNodeData) -> bool {
    node.role.as_str() == "DragOverlay" || node.component_role.as_str() == "drag-overlay"
}

fn preview_frame(node: &TemplatePaneNodeData, fallback: &FrameRect) -> FrameRect {
    let width = node.drag_preview_width.max(0.0);
    let height = node.drag_preview_height.max(0.0);
    let width = if width > 0.0 { width } else { fallback.width };
    let height = if height > 0.0 {
        height
    } else {
        fallback.height
    };
    if node.has_drag_cursor {
        return FrameRect {
            x: node.drag_cursor_x + node.drag_offset_x,
            y: node.drag_cursor_y + node.drag_offset_y,
            width: width.max(1.0),
            height: height.max(1.0),
        };
    }
    FrameRect {
        x: fallback.x,
        y: fallback.y,
        width: width.max(1.0),
        height: height.max(1.0),
    }
}

fn indicator_frame(node: &TemplatePaneNodeData) -> Option<FrameRect> {
    if !node.has_drop_target {
        return None;
    }
    let width = node.drop_target_width.max(1.0);
    let height = node.drop_target_height.max(1.0);
    match node.drop_indicator_edge.as_str() {
        "top" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width,
            height: INDICATOR_THICKNESS,
        }),
        "bottom" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y + (height - INDICATOR_THICKNESS).max(0.0),
            width,
            height: INDICATOR_THICKNESS,
        }),
        "left" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width: INDICATOR_THICKNESS,
            height,
        }),
        "right" => Some(FrameRect {
            x: node.drop_target_x + (width - INDICATOR_THICKNESS).max(0.0),
            y: node.drop_target_y,
            width: INDICATOR_THICKNESS,
            height,
        }),
        "inside" => Some(FrameRect {
            x: node.drop_target_x,
            y: node.drop_target_y,
            width,
            height,
        }),
        _ => None,
    }
}

fn preview_label(node: &TemplatePaneNodeData) -> Option<String> {
    [
        node.drag_payload_label.as_str(),
        node.text.as_str(),
        node.drag_payload_reference.as_str(),
        node.value_text.as_str(),
    ]
    .into_iter()
    .map(str::trim)
    .find(|value| !value.is_empty())
    .map(str::to_string)
}
