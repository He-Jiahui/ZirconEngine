use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::style::{list_row_background, list_row_border, list_row_border_width};

const LIST_ROW_RADIUS: f32 = 4.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_list_row_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let Some(background) = list_row_background(node) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        list_row_border(node),
        list_row_border_width(node),
        LIST_ROW_RADIUS,
        opacity,
    ));
}
