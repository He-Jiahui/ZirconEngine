use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::theme::PALETTE;
use super::{component_variant_contains, resolved_style_color};

const TIMELINE_DOT_BORDER_WIDTH: f32 = 2.0;
const TIMELINE_CONNECTOR_WIDTH: f32 = 2.0;
const MUI_GREY_400: [u8; 4] = [189, 189, 189, 255];
const MUI_SECONDARY_MAIN: [u8; 4] = [156, 39, 176, 255];

pub(super) fn push_timeline_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match timeline_primitive_kind(node) {
        Some(TimelinePrimitiveKind::Dot) => {
            push_timeline_dot(commands, node, rect, clip, order, opacity);
        }
        Some(TimelinePrimitiveKind::Connector) => {
            push_timeline_connector(commands, node, rect, clip, order, opacity);
        }
        Some(TimelinePrimitiveKind::Separator) => {}
        None => return false,
    }
    true
}

enum TimelinePrimitiveKind {
    Dot,
    Connector,
    Separator,
}

fn timeline_primitive_kind(node: &TemplatePaneNodeData) -> Option<TimelinePrimitiveKind> {
    let component_role = node.component_role.as_str();
    let role = node.role.as_str();
    if matches_timeline_role(component_role, role, &["timeline-dot", "TimelineDot"]) {
        Some(TimelinePrimitiveKind::Dot)
    } else if matches_timeline_role(
        component_role,
        role,
        &["timeline-connector", "TimelineConnector"],
    ) {
        Some(TimelinePrimitiveKind::Connector)
    } else if matches_timeline_role(
        component_role,
        role,
        &["timeline-separator", "TimelineSeparator"],
    ) {
        Some(TimelinePrimitiveKind::Separator)
    } else {
        None
    }
}

fn matches_timeline_role(component_role: &str, role: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|candidate| component_role == *candidate || role == *candidate)
}

fn push_timeline_dot(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let dot = centered_square(rect);
    if dot.width <= 0.0 || dot.height <= 0.0 {
        return;
    }

    let outlined = timeline_dot_is_outlined(node);
    let tone = timeline_dot_tone_color(node);
    let background = timeline_dot_background_color(node, outlined, tone);
    let border_color = timeline_dot_border_color(node, outlined, tone);
    let border_width = timeline_dot_border_width(node, outlined, border_color.is_some());
    commands.push(HostPaintCommand::quad(
        dot.clone(),
        Some(clip.clone()),
        order,
        background,
        border_color,
        border_width,
        dot.width.min(dot.height) * 0.5,
        opacity,
    ));
}

fn push_timeline_connector(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let width = rect.width.min(TIMELINE_CONNECTOR_WIDTH).max(0.0);
    if width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let x = (rect.x + (rect.width - width).max(0.0) * 0.5).round();
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x,
            y: rect.y,
            width,
            height: rect.height,
        },
        Some(clip.clone()),
        order,
        Some(timeline_connector_color(node)),
        None,
        0.0,
        width * 0.5,
        opacity,
    ));
}

fn centered_square(rect: &FrameRect) -> FrameRect {
    let size = rect.width.min(rect.height).max(0.0);
    FrameRect {
        x: (rect.x + (rect.width - size).max(0.0) * 0.5).round(),
        y: (rect.y + (rect.height - size).max(0.0) * 0.5).round(),
        width: size.round().max(1.0),
        height: size.round().max(1.0),
    }
}

fn timeline_dot_is_outlined(node: &TemplatePaneNodeData) -> bool {
    component_variant_contains(node, "outlined")
}

fn timeline_dot_background_color(
    node: &TemplatePaneNodeData,
    outlined: bool,
    tone: [u8; 4],
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref()).or_else(|| {
        if outlined {
            None
        } else if timeline_dot_color_token(node) == "grey" {
            Some(MUI_GREY_400)
        } else {
            Some(tone)
        }
    })
}

fn timeline_dot_border_color(
    node: &TemplatePaneNodeData,
    outlined: bool,
    tone: [u8; 4],
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        if outlined {
            Some(tone)
        } else {
            None
        }
    })
}

fn timeline_dot_border_width(node: &TemplatePaneNodeData, outlined: bool, has_border: bool) -> f32 {
    if !has_border {
        return 0.0;
    }
    let style_width = node.button_style.element.border_width;
    if style_width.is_finite() && style_width > 0.0 {
        style_width
    } else if node.border_width.is_finite() && node.border_width > 0.0 {
        node.border_width.max(if outlined {
            TIMELINE_DOT_BORDER_WIDTH
        } else {
            1.0
        })
    } else if outlined {
        TIMELINE_DOT_BORDER_WIDTH
    } else {
        1.0
    }
}

fn timeline_connector_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .or_else(|| resolved_style_color(node.button_style.element.foreground_color.as_ref()))
        .or_else(|| resolved_style_color(node.button_style.element.border_color.as_ref()))
        .unwrap_or(MUI_GREY_400)
}

fn timeline_dot_tone_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    match timeline_dot_color_token(node) {
        "secondary" => MUI_SECONDARY_MAIN,
        "grey" => MUI_GREY_400,
        "inherit" | "muted" | "subtle" => PALETTE.text_muted,
        "warning" => PALETTE.warning,
        "error" | "danger" => PALETTE.error,
        "success" => PALETTE.success,
        "info" => PALETTE.info,
        "primary" | "accent" | "default" => PALETTE.accent,
        _ => PALETTE.accent,
    }
}

fn timeline_dot_color_token(node: &TemplatePaneNodeData) -> &str {
    for token in [
        "secondary",
        "primary",
        "grey",
        "inherit",
        "warning",
        "error",
        "danger",
        "success",
        "info",
    ] {
        if component_variant_contains(node, token) {
            return token;
        }
    }
    match node.text_tone.as_str() {
        "" => "grey",
        "inverse" | "on-dark" => "inherit",
        other => other,
    }
}
