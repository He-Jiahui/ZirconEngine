use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::context::{build_slider_command_context, SliderCommandContext};
use super::sequence::push_ready_slider_commands;
use crate::ui::retained_host::host_contract::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_slider_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    match build_slider_command_context(node, rect) {
        SliderCommandContext::NotSlider => false,
        SliderCommandContext::Consumed => true,
        SliderCommandContext::Ready(context) => {
            if intersect(&context.rect, clip).is_none() {
                return true;
            }
            push_ready_slider_commands(commands, node, clip, order, opacity, context);
            true
        }
    }
}
