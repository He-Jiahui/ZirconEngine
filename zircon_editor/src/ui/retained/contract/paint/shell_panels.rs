use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;
use super::render_commands::HostPaintCommand;

#[path = "shell_panels/frame.rs"]
mod frame;
#[path = "shell_panels/identity.rs"]
mod identity;
#[path = "shell_panels/separators.rs"]
mod separators;
#[path = "shell_panels/surface.rs"]
mod surface;

use identity::shell_panel_kind;
use surface::push_shell_panel_surface;

#[cfg(test)]
#[path = "shell_panels_tests/tests/mod.rs"]
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
    let Some(clip) = intersect(rect, clip) else {
        return true;
    };

    push_shell_panel_surface(commands, node, kind, rect, &clip, order, opacity);
    true
}
