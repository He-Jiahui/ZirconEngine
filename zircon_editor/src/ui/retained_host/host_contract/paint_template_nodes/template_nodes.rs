#[cfg(test)]
use crate::ui::retained_host::primitives::ModelRc;

#[cfg(test)]
use super::super::data::TemplatePaneNodeData;

mod commands;
mod fallback;
mod geometry;
mod ordering;
mod specialized;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_template_node_commands;

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
