use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::material_state_layer::push_state_layer_commands;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::{
    is_asset_browser_toolbar_chip_button, is_tab_like_workbench_button, WorkbenchButtonKind,
};
use super::geometry::frame_is_within;
use super::layers::{content_order, surface_overlay_order};
use super::surface_indicator::{button_surface_indicator_palette, button_surface_indicator_rect};

mod style;

use style::button_surface_command_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_button_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: WorkbenchButtonKind,
    opacity: f32,
) {
    let command_style = button_surface_command_style(node, rect, kind);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(command_style.fill),
        Some(command_style.border),
        command_style.border_width,
        command_style.radius,
        opacity,
    ));
    push_state_layer_commands(
        commands,
        node,
        rect,
        clip,
        command_style.radius,
        surface_overlay_order(order),
        opacity,
    );
    if should_paint_tab_like_indicator(node) {
        let palette = button_surface_indicator_palette();
        let indicator = button_surface_indicator_rect(node, rect);
        if frame_is_within(&indicator, rect) {
            commands.push(HostPaintCommand::quad(
                indicator,
                Some(clip.clone()),
                content_order(order),
                Some(palette.underline),
                None,
                0.0,
                0.0,
                opacity,
            ));
        }
    }
}

fn should_paint_tab_like_indicator(node: &TemplatePaneNodeData) -> bool {
    is_tab_like_workbench_button(node)
        && !is_asset_browser_toolbar_chip_button(node)
        && (node.selected || node.checked)
}
