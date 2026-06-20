use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchButtonKind;
use super::geometry::button_radius;
use super::style::button_style;

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
}
