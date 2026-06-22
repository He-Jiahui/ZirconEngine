use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::search::push_command_palette_search_field;
use super::surface::push_command_palette_panel_surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_palette_panel_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_command_palette_panel_surface(commands, rect, clip, order, opacity);
    push_command_palette_search_field(commands, node, rect, clip, order + 1, opacity);
}
