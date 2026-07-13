use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::style::{section_title_metrics, section_title_palette};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_section_title_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = section_title_metrics();
    let palette = section_title_palette();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(palette.header_surface),
        Some(palette.separator),
        metrics.separator_height,
        0.0,
        opacity,
    ));
}
