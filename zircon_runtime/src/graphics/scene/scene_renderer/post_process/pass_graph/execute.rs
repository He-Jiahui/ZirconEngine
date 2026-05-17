use crate::core::framework::render::PostProcessPassGraph;
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources,
};
use std::collections::BTreeSet;

pub(crate) fn execute_post_process_pass_graph(
    graph: &PostProcessPassGraph,
    resources: &RenderGraphExecutionResources,
    record: &mut RenderGraphExecutionRecord,
) {
    let produced_resources = graph
        .nodes
        .iter()
        .flat_map(|node| node.produced_outputs.iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut available_resources = graph
        .nodes
        .iter()
        .flat_map(|node| node.required_inputs.iter())
        .filter(|resource| !produced_resources.contains(*resource))
        .filter(|resource| resources.has_texture_view(resource))
        .cloned()
        .collect::<BTreeSet<_>>();

    for node in &graph.nodes {
        if !node.required_inputs.iter().all(|resource| {
            resources.has_texture_view(resource) && available_resources.contains(resource)
        }) {
            continue;
        }
        record.push_executed_post_process_node(node.name.clone());
        available_resources.extend(node.produced_outputs.iter().cloned());
    }
}
