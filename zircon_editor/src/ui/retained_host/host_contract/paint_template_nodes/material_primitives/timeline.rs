use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use style::{
    timeline_connector_color, timeline_dot_background_color, timeline_dot_border_color,
    timeline_dot_border_width, timeline_dot_is_outlined, timeline_dot_tone_color,
};

const TIMELINE_CONNECTOR_WIDTH: f32 = 2.0;

mod style;

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
