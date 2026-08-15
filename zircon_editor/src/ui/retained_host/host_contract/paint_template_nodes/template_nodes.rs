#[cfg(test)]
use crate::ui::retained_host::primitives::ModelRc;

#[cfg(test)]
use super::super::data::FrameRect;
#[cfg(test)]
use super::super::data::TemplatePaneNodeData;

mod commands;
mod fallback;
mod geometry;
mod ordering;
mod specialized;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_template_node_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::template_node_intersects_clip;

#[cfg(test)]
pub(crate) fn paint_template_nodes_for_test(
    width: u32,
    height: u32,
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    super::template_node_pipeline::paint_template_nodes_for_test(width, height, nodes)
}

#[cfg(test)]
pub(crate) fn paint_template_nodes_for_test_with_background(
    width: u32,
    height: u32,
    background: [u8; 4],
    nodes: ModelRc<TemplatePaneNodeData>,
) -> Vec<u8> {
    super::template_node_pipeline::paint_template_nodes_for_test_with_background(
        width, height, background, nodes,
    )
}

#[cfg(test)]
pub(crate) struct TemplateNodeCommandSummary {
    pub(crate) text_count: usize,
    pub(crate) text_frames: Vec<FrameRect>,
    pub(crate) image_frames: Vec<FrameRect>,
}

#[cfg(test)]
pub(crate) fn template_node_command_summary_for_test(
    node: &TemplatePaneNodeData,
) -> TemplateNodeCommandSummary {
    let origin = FrameRect::default();
    let clip = FrameRect {
        x: 0.0,
        y: 0.0,
        width: (node.frame.x + node.frame.width + 64.0).max(1.0),
        height: (node.frame.y + node.frame.height + 64.0).max(1.0),
    };
    let mut commands = Vec::new();
    push_template_node_commands(&mut commands, node, &origin, &clip, None, 0);
    TemplateNodeCommandSummary {
        text_count: commands
            .iter()
            .filter(|command| command.text.is_some())
            .count(),
        text_frames: commands
            .iter()
            .filter(|command| command.text.is_some())
            .map(|command| command.frame.clone())
            .collect(),
        image_frames: commands
            .iter()
            .filter(|command| command.image_pixels.is_some())
            .map(|command| command.frame.clone())
            .collect(),
    }
}
