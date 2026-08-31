use super::super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_surface::is_frame_only_node;
use super::fallback::push_template_fallback_commands;
use super::geometry::template_node_rect_and_clip;
use super::ordering::{template_node_paint_order, template_node_transition_opacity};
use super::specialized::push_specialized_template_node_commands;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_template_node_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
    order: i32,
) {
    let Some((rect, node_clip)) = template_node_rect_and_clip(node, origin, clip) else {
        return;
    };
    if is_frame_only_node(node) {
        return;
    }

    let order = template_node_paint_order(node, order);
    let opacity = template_node_transition_opacity(node);
    if opacity <= 0.0 {
        return;
    }

    let command_start = commands.len();
    if push_specialized_template_node_commands(
        commands,
        node,
        &rect,
        &node_clip,
        origin,
        clip,
        text_input_focus,
        order,
        opacity,
    ) {
        tag_commands_with_source(
            &mut commands[command_start..],
            node.surface_render_command_ref,
        );
        return;
    }

    push_template_fallback_commands(
        commands,
        node,
        &rect,
        &node_clip,
        origin,
        clip,
        order,
        opacity,
        text_input_focus,
    );
    tag_commands_with_source(
        &mut commands[command_start..],
        node.surface_render_command_ref,
    );
}

fn tag_commands_with_source(
    commands: &mut [HostPaintCommand],
    command_ref: Option<zircon_runtime_interface::ui::surface::UiRenderFrameCommandRef>,
) {
    for command in commands {
        command.source_render_command_ref = command_ref;
    }
}
