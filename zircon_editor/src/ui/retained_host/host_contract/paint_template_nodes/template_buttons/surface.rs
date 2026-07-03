use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::{
    is_asset_browser_tab_like_button, is_asset_browser_toolbar_chip_button,
    is_tab_like_workbench_button, WorkbenchButtonKind,
};
use super::geometry::button_radius;
use super::style::button_style;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, current_host_palette,
};

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
    if should_paint_tab_like_indicator(node) {
        commands.push(HostPaintCommand::quad(
            tab_like_indicator_rect(node, rect),
            Some(clip.clone()),
            order + 1,
            Some(current_host_palette().accent),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn should_paint_tab_like_indicator(node: &TemplatePaneNodeData) -> bool {
    is_tab_like_workbench_button(node)
        && !is_asset_browser_toolbar_chip_button(node)
        && (node.selected || node.checked || node.focused)
}

fn tab_like_indicator_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let metrics = current_host_metrics();
    let height = metrics.tab_underline_height.min(rect.height).max(1.0);
    let inset = if is_asset_browser_tab_like_button(node) {
        metrics.button_pad_x.min(rect.width * 0.24).max(0.0)
    } else {
        0.0
    };
    FrameRect {
        x: rect.x + inset,
        y: rect.y + (rect.height - height).max(0.0),
        width: (rect.width - inset * 2.0).max(1.0),
        height,
    }
}
