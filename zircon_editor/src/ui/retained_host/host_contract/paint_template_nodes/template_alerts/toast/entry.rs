use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::select_workbench_toast_style;
use super::super::layout::{
    frame_is_within, toast_close_rect, toast_has_action, toast_icon_rect, toast_metrics,
};
use super::action::push_toast_action;
use super::icon::{push_toast_status_mark, toast_status_mark_size_for_metrics};
use super::surface::push_toast_surface;
use super::text::push_toast_text;
use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_toast(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let style = select_workbench_toast_style(node);
    let metrics = toast_metrics();
    let surface_radius = toast_surface_radius(node, metrics);
    push_toast_surface(
        commands,
        rect,
        clip,
        order,
        style.surface,
        style.border,
        metrics.border_width,
        surface_radius,
        opacity,
    );

    let icon = toast_icon_rect(
        rect,
        toast_status_mark_size_for_metrics(node, metrics),
        metrics,
    );
    let status_mark_size = toast_status_mark_size_for_metrics(node, metrics);
    let has_icon = !node.icon_name.is_empty();
    if has_icon
        && frame_is_within(&icon, rect)
        && icon.width >= status_mark_size
        && icon.height >= status_mark_size
    {
        push_toast_status_mark(commands, &icon, clip, order + 1, style.mark, opacity);
    }

    let close = toast_close_rect(rect, metrics);
    let has_action = toast_has_action(rect, metrics)
        && frame_is_within(&close, rect)
        && close.width >= metrics.close_size
        && close.height >= metrics.close_size;
    push_toast_text(
        commands,
        node,
        rect,
        if has_icon { Some(&icon) } else { None },
        &close,
        clip,
        order + 2,
        style.text,
        has_action,
        metrics,
        opacity,
    );

    if has_action {
        push_toast_action(
            commands,
            rect,
            &close,
            clip,
            order + 2,
            style.action,
            style.close,
            metrics,
            opacity,
        );
    }
}

fn toast_surface_radius(
    node: &TemplatePaneNodeData,
    metrics: super::super::layout::WorkbenchToastMetrics,
) -> f32 {
    if node.corner_radius.is_finite() && node.corner_radius > 0.0 {
        node.corner_radius
    } else {
        metrics.radius
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_prefers_the_projected_panel_radius() {
        let mut node = TemplatePaneNodeData::default();
        node.corner_radius = 14.0;

        assert_eq!(toast_surface_radius(&node, toast_metrics()), 14.0);
    }
}
