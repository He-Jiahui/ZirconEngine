use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchIconButtonStyle;
use super::geometry::icon_button_surface_radius;

mod style;

use style::icon_button_surface_command_style;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_button_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchIconButtonStyle,
    opacity: f32,
) {
    let Some(command_style) = icon_button_surface_command_style(style) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        command_style.background,
        command_style.border,
        command_style.border_width,
        icon_button_surface_radius(rect, command_style.radius),
        opacity,
    ));
}
