use crate::ui::retained_host::primitives::ModelRc;

use super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::{intersect, is_visible_frame};
use super::render_commands::{draw_host_paint_commands, HostPaintCommand};
use super::template_nodes::push_template_node_commands;

pub(in crate::ui::retained_host::host_contract) fn draw_template_nodes(
    frame: &mut HostRgbaFrame,
    nodes: &ModelRc<TemplatePaneNodeData>,
    origin: &FrameRect,
    clip: &FrameRect,
    text_input_focus: Option<&HostTextInputFocusData>,
) -> bool {
    let Some(effective_clip) = effective_template_clip(frame, clip) else {
        return false;
    };

    let mut commands: Vec<HostPaintCommand> = Vec::new();
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "template_nodes_collect_commands");
        zircon_runtime::profile_counter!("editor", "template_node_count", nodes.row_count());
        for row in 0..nodes.row_count() {
            let Some(node) = nodes.row_data(row) else {
                continue;
            };
            // Region repaint must avoid generating commands for off-damage nodes:
            // image commands can rasterize previews before the final primitive clip runs.
            push_template_node_commands(
                &mut commands,
                &node,
                origin,
                &effective_clip,
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

fn effective_template_clip(frame: &HostRgbaFrame, clip: &FrameRect) -> Option<FrameRect> {
    match frame.paint_clip() {
        Some(active_clip) => intersect(active_clip, clip),
        None if is_visible_frame(clip) => Some(clip.clone()),
        None => None,
    }
}

pub(in crate::ui::retained_host::host_contract) fn has_template_nodes(
    nodes: &ModelRc<TemplatePaneNodeData>,
) -> bool {
    nodes.row_count() > 0
}

#[cfg(test)]
pub(crate) fn paint_template_nodes_for_test(
    width: u32,
    height: u32,
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    paint_template_nodes_for_test_with_background(width, height, [0, 0, 0, 255], nodes)
}

#[cfg(test)]
pub(crate) fn paint_template_nodes_for_test_with_background(
    width: u32,
    height: u32,
    background: [u8; 4],
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    let mut frame = HostRgbaFrame::filled(width, height, background);
    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: width as f32,
        height: height as f32,
    };
    draw_template_nodes(&mut frame, &nodes, &bounds, &bounds, None);
    frame.into_bytes()
}

#[cfg(test)]
#[path = "template_node_pipeline_tests.rs"]
mod tests;
