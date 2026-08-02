use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::context::{SliderCommandContext, build_slider_command_context};
use super::sequence::push_ready_slider_commands;

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
            push_ready_slider_commands(commands, node, clip, order, opacity, context);
            true
        }
    }
}
