use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::{is_tab_like_workbench_button, WorkbenchButtonKind};
use super::geometry::button_radius;
use super::style::button_style;
use crate::ui::retained_host::host_contract::paint_theme::{METRICS, PALETTE};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_button_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: WorkbenchButtonKind,
    opacity: f32,
) {
    let style = button_style(node, kind);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        style.border_width,
        button_radius(node, rect),
        opacity,
    ));
    if is_tab_like_workbench_button(node) && (node.selected || node.checked || node.focused) {
        commands.push(HostPaintCommand::quad(
            tab_like_indicator_rect(rect),
            Some(clip.clone()),
            order + 1,
            Some(PALETTE.accent),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn tab_like_indicator_rect(rect: &FrameRect) -> FrameRect {
    let height = METRICS.tab_underline_height.min(rect.height).max(1.0);
    FrameRect {
        x: rect.x,
        y: rect.y + (rect.height - height).max(0.0),
        width: rect.width,
        height,
    }
}
