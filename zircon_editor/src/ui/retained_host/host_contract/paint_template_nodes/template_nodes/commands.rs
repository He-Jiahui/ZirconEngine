use super::super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::super::super::template_component_family::{
    is_component_family, TemplateComponentFamily,
};
use super::super::render_commands::HostPaintCommand;
use super::super::template_node_surface::is_frame_only_node;
use super::super::template_node_text::{
    focused_text_edit_feedback_for_text_command, push_template_text_fallback_command,
};
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

    let focused_input = text_input_focus.filter(|focus| {
        focus.accepts_text_input()
            && focus.control_id.as_str() == node.control_id.as_str()
            && is_component_family(node, TemplateComponentFamily::TextInput)
    });
    let focused_node = focused_input.map(|_| {
        let mut draft = node.clone();
        draft.focused = true;
        draft
    });
    let node = focused_node.as_ref().unwrap_or(node);

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
        if focused_input.is_some() {
            // Specialized fields paint committed values without focus feedback.
            // Keep their surface and text slot, and reuse shared edit feedback.
            if let Some(text_index) = (command_start..commands.len())
                .rev()
                .find(|index| commands[*index].text.is_some())
            {
                let focus = focused_input.unwrap();
                commands[text_index].text = Some(focus.value_text.to_string());
                commands[text_index].z_index = commands[text_index].z_index.saturating_add(1);
                if let Some((selection, caret)) = focused_text_edit_feedback_for_text_command(
                    node,
                    &commands[text_index],
                    text_input_focus,
                    opacity,
                ) {
                    if let Some(selection) = selection {
                        commands.push(selection);
                    }
                    commands.push(caret);
                }
            } else {
                let text_order = commands[command_start..]
                    .iter()
                    .map(|command| command.z_index)
                    .max()
                    .unwrap_or(order)
                    .saturating_add(2);
                push_template_text_fallback_command(
                    commands,
                    node,
                    &rect,
                    &node_clip,
                    text_order,
                    text_input_focus,
                    false,
                    false,
                    opacity,
                );
            }
        }
        tag_commands_with_source(&mut commands[command_start..], node);
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
    tag_commands_with_source(&mut commands[command_start..], node);
}

fn tag_commands_with_source(commands: &mut [HostPaintCommand], node: &TemplatePaneNodeData) {
    for command in commands {
        command.source_render_command_ref = node.surface_render_command_ref;
        command.source_surface_frame = node.source_surface_frame.clone();
    }
}

#[cfg(test)]
#[path = "commands/tests/focused_input_tests.rs"]
mod focused_input_tests;
