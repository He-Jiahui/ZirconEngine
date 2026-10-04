//! 阴影需有完整容器余量，气泡始终使用已通过布局检查的框；显式圆角优先于主题默认。

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::layers::bubble_order;
use super::layout::frame_is_within;
use super::metrics::tooltip_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tooltip_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    bubble: &FrameRect,
    clip: &FrameRect,
    order: i32,
    shadow: [u8; 4],
    surface: [u8; 4],
    border: [u8; 4],
    opacity: f32,
) {
    let metrics = tooltip_metrics();
    let radius = tooltip_surface_radius(node);
    let shadow_rect = FrameRect {
        x: bubble.x,
        y: bubble.y + metrics.shadow_offset_y,
        width: bubble.width,
        height: bubble.height,
    };
    if frame_is_within(rect, &shadow_rect) {
        commands.push(
            HostPaintCommand::quad(
                shadow_rect,
                Some(clip.clone()),
                order,
                Some(shadow),
                None,
                0.0,
                radius,
                opacity,
            )
            .with_box_shadow(0.0, metrics.shadow_offset_y, 0.0, 0.0, radius, false),
        );
    }
    commands.push(HostPaintCommand::quad(
        bubble.clone(),
        Some(clip.clone()),
        bubble_order(order),
        Some(surface),
        Some(border),
        metrics.border_width,
        radius,
        opacity,
    ));
}

pub(super) fn tooltip_surface_radius(node: &TemplatePaneNodeData) -> f32 {
    if node.corner_radius.is_finite() && node.corner_radius > 0.0 {
        node.corner_radius
    } else {
        tooltip_metrics().radius
    }
}
