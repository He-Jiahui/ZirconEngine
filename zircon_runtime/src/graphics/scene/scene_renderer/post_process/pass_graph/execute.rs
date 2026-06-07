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
        .filter(|resource| resources.has_bound_resource(resource))
        .cloned()
        .collect::<BTreeSet<_>>();

    for node in &graph.nodes {
        if !node.required_inputs.iter().all(|resource| {
            resources.has_bound_resource(resource) && available_resources.contains(resource)
        }) {
            continue;
        }
        record.push_executed_post_process_node(node.name.clone());
        available_resources.extend(node.produced_outputs.iter().cloned());
    }
}

#[cfg(test)]
mod tests {
    use super::execute_post_process_pass_graph;
    use crate::core::framework::render::{
        PostProcessEffectKind, PostProcessPassGraph, PostProcessPassNode,
    };
    use crate::graphics::backend::RenderBackend;
    use crate::graphics::scene::scene_renderer::graph_execution::{
        RenderGraphExecutionRecord, RenderGraphExecutionResources,
    };

    #[test]
    fn post_process_pass_graph_executes_nodes_with_buffer_backed_inputs() {
        let backend = RenderBackend::new_offscreen().unwrap();
        let buffer = backend.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-postprocess-buffer-input"),
            size: 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let mut resources = RenderGraphExecutionResources::new();
        resources.insert_buffer("postprocess.compute-sideband", buffer);
        let graph = PostProcessPassGraph {
            nodes: vec![PostProcessPassNode {
                name: "buffer-backed-effect".to_string(),
                kind: PostProcessEffectKind::EffectStack,
                required_inputs: vec!["postprocess.compute-sideband".to_string()],
                produced_outputs: vec!["postprocess.effect-stacked".to_string()],
                after: Vec::new(),
            }],
            skipped_nodes: Vec::new(),
            final_composite_node: None,
        };
        let mut record = RenderGraphExecutionRecord::default();

        execute_post_process_pass_graph(&graph, &resources, &mut record);

        assert_eq!(
            record.executed_post_process_nodes(),
            &["buffer-backed-effect".to_string()]
        );
    }
}
