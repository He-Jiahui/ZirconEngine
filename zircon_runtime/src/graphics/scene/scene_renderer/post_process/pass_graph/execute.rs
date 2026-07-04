use crate::core::framework::render::{PostProcessEffectKind, PostProcessPassGraph};
use crate::graphics::scene::scene_renderer::graph_execution::{
    RenderGraphExecutionRecord, RenderGraphExecutionResources,
};
use std::collections::BTreeSet;

pub(crate) fn execute_post_process_pass_graph(
    graph: &PostProcessPassGraph,
    resources: &RenderGraphExecutionResources,
    record: &mut RenderGraphExecutionRecord,
) {
    let executed_executor_ids = record
        .executed_executor_ids()
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if !executed_executor_ids.is_empty() {
        for node in &graph.nodes {
            if post_process_effect_executor_ids(node.kind)
                .iter()
                .any(|executor_id| executed_executor_ids.contains(*executor_id))
            {
                record.push_executed_post_process_node(node.name.clone());
            }
        }
        return;
    }

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

fn post_process_effect_executor_ids(kind: PostProcessEffectKind) -> &'static [&'static str] {
    match kind {
        PostProcessEffectKind::Blur => &["post.blur"],
        PostProcessEffectKind::Bloom => &["post.bloom", "post.bloom-extract"],
        PostProcessEffectKind::ColorLutBake => &["post.color-lut-bake"],
        PostProcessEffectKind::DepthOfField => &["post.depth-of-field"],
        PostProcessEffectKind::ExposureHistogram => &["post.exposure.histogram"],
        PostProcessEffectKind::ExposureResolve => &["post.exposure.resolve"],
        PostProcessEffectKind::MotionBlur => &["post.motion-blur"],
        PostProcessEffectKind::SceneComposite => &["post.scene-composite"],
        PostProcessEffectKind::TaaResolve => &["temporal.taa-resolve"],
        PostProcessEffectKind::Uber => &["post.uber"],
        PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramid => {
            &["post.screen-space-reflection-reflection-pyramid"]
        }
        PostProcessEffectKind::ScreenSpaceReflectionReflectionPyramidCoarse => {
            &["post.screen-space-reflection-reflection-pyramid-coarse"]
        }
        PostProcessEffectKind::ScreenSpaceReflectionSpecularOcclusion => {
            &["post.screen-space-reflection-specular-occlusion"]
        }
        PostProcessEffectKind::ScreenSpaceReflectionResolve => {
            &["post.screen-space-reflection-resolve"]
        }
        PostProcessEffectKind::Upscale => &["post.upscale"],
        PostProcessEffectKind::OutputTransfer => &["post.output-transfer"],
        PostProcessEffectKind::Fxaa => &["post.fxaa"],
        PostProcessEffectKind::Smaa => &["post.smaa"],
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
    use crate::graphics::RenderPassStage;
    use crate::render_graph::QueueLane;

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
        let graph = PostProcessPassGraph::from_ordered_nodes(
            vec![
                PostProcessPassNode::new("buffer-backed-effect", PostProcessEffectKind::Uber)
                    .with_required_inputs(["postprocess.compute-sideband"])
                    .with_produced_outputs(["postprocess.effect-stacked"]),
            ],
            Vec::new(),
            None,
        );
        let mut record = RenderGraphExecutionRecord::default();

        execute_post_process_pass_graph(&graph, &resources, &mut record);

        assert_eq!(
            record.executed_post_process_nodes(),
            &["buffer-backed-effect".to_string()]
        );
    }

    #[test]
    fn post_process_pass_graph_records_nodes_from_executed_executor_ids() {
        let resources = RenderGraphExecutionResources::new();
        let graph = PostProcessPassGraph::from_ordered_nodes(
            vec![
                PostProcessPassNode::new(
                    "screen-space-reflection-resolve",
                    PostProcessEffectKind::ScreenSpaceReflectionResolve,
                )
                .with_required_inputs(["screen-space-reflection-reflection-pyramid-coarse"])
                .with_produced_outputs(["screen-space-reflection-history"]),
                PostProcessPassNode::new("scene-composite", PostProcessEffectKind::SceneComposite)
                    .with_required_inputs(["screen-space-reflection-history"])
                    .with_produced_outputs(["scene-composited"]),
                PostProcessPassNode::new("uber", PostProcessEffectKind::Uber)
                    .with_required_inputs(["scene-composited"])
                    .with_produced_outputs(["tonemapped"]),
            ],
            Vec::new(),
            None,
        );
        let mut record = RenderGraphExecutionRecord::default();
        for executor_id in [
            "post.screen-space-reflection-resolve",
            "post.scene-composite",
            "post.uber",
        ] {
            record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
                Some(RenderPassStage::PostProcess),
                executor_id,
                executor_id,
                QueueLane::Graphics,
                QueueLane::Graphics,
                Vec::new(),
                Vec::new(),
            );
        }

        execute_post_process_pass_graph(&graph, &resources, &mut record);

        assert_eq!(
            record.executed_post_process_nodes(),
            &[
                "screen-space-reflection-resolve".to_string(),
                "scene-composite".to_string(),
                "uber".to_string(),
            ]
        );
    }

    #[test]
    fn post_process_pass_graph_records_bloom_extract_executor_as_bloom_node() {
        let resources = RenderGraphExecutionResources::new();
        let graph = PostProcessPassGraph::from_ordered_nodes(
            vec![
                PostProcessPassNode::new("bloom", PostProcessEffectKind::Bloom)
                    .with_required_inputs(["scene-color"])
                    .with_produced_outputs(["bloom-texture"]),
            ],
            Vec::new(),
            None,
        );
        let mut record = RenderGraphExecutionRecord::default();
        record.push_executed_pass_with_stage_declared_queue_dependencies_and_resources(
            Some(RenderPassStage::PostProcess),
            "bloom-extract",
            "post.bloom-extract",
            QueueLane::Graphics,
            QueueLane::Graphics,
            Vec::new(),
            Vec::new(),
        );

        execute_post_process_pass_graph(&graph, &resources, &mut record);

        assert_eq!(record.executed_post_process_nodes(), &["bloom".to_string()]);
    }
}
