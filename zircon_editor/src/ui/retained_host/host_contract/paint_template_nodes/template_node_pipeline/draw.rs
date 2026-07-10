use crate::ui::retained_host::primitives::ModelRc;

use super::super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::render_commands::{draw_host_paint_commands, HostPaintCommand};
use super::super::template_nodes::push_template_node_commands;
use super::clip::effective_template_clip;
use super::transform::TemplateNodePaintTransform;

pub(in crate::ui::retained_host::host_contract) fn draw_template_nodes(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    draw_template_nodes_with_transform(frame, nodes, origin, clip, text_input_focus, None)
}

pub(in crate::ui::retained_host::host_contract) fn draw_template_nodes_with_transform(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
    transform: Option<&dyn TemplateNodePaintTransform>,
) -> bool {
    let Some(effective_clip) = effective_template_clip(frame, clip) else {
        return false;
    };

    let mut commands: Vec<HostPaintCommand> = Vec::new();
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_nodes_collect_commands");
        zircon_runtime::profile_counter!("editor", "template_node_count", nodes.row_count());
        for row in 0..nodes.row_count() {
            let Some(source_node) = nodes.row_data(row) else {
                continue;
            };
            let Some((node, node_clip)) = transform
                .map(|transform| transform.transform(source_node.clone(), effective_clip.clone()))
                .unwrap_or_else(|| Some((source_node, effective_clip.clone())))
            else {
                continue;
            };
            // Region repaint must avoid generating commands for off-damage nodes:
            // image commands can rasterize previews before the final primitive clip runs.
            push_template_node_commands(
                &mut commands,
                &node,
                origin,
                &node_clip,
                text_input_focus,
                row as i32,
            );
        }
    }
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_nodes_draw_commands");
        zircon_runtime::profile_counter!("editor", "template_command_count", commands.len());
        draw_host_paint_commands(frame, &commands)
    }
}

pub(in crate::ui::retained_host::host_contract) fn has_template_nodes(
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> bool {
    nodes.row_count() > 0
}
