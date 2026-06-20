use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::style_selector::WorkbenchIconButtonStyle;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_icon_button_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchIconButtonStyle,
    opacity: f32,
) {
    let Some(background) = style.background else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        style.border,
        style.border_width,
        style.radius,
        opacity,
    ));
}
