use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

mod frame;
mod identity;
mod separators;
mod surface;

use identity::shell_panel_kind;
use surface::push_shell_panel_surface;

#[cfg(test)]
#[path = "template_shell_panels_tests/mod.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_shell_panel_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    let Some(kind) = shell_panel_kind(node) else {
        return false;
    };
    let rect = separators::pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    push_shell_panel_surface(commands, node, kind, &rect, clip, order, opacity);
    true
}
