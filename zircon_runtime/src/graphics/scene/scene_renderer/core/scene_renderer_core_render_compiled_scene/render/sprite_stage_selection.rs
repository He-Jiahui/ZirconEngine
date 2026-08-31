use crate::graphics::CompiledRenderPipeline;
use crate::graphics::pipeline::RenderPassStage;

const SPRITE_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::Opaque2d,
    RenderPassStage::AlphaMask2d,
    RenderPassStage::Transparent2d,
];
pub(super) fn active_sprite_graph_stages(
    pipeline: &CompiledRenderPipeline,
) -> impl Iterator<Item = RenderPassStage> + '_ {
    SPRITE_GRAPH_STAGES
        .iter()
        .copied()
        .filter(|stage| pipeline_has_active_sprite_stage(pipeline, *stage))
}

fn pipeline_has_active_sprite_stage(
    pipeline: &CompiledRenderPipeline,
    stage: RenderPassStage,
) -> bool {
    pipeline.execution_batches_for_stage(stage).any(|batch| {
        pipeline
            .execution_passes_for_batch(batch)
            .filter(|execution_pass| execution_pass.stage == stage)
            .any(|execution_pass| {
                pipeline
                    .graph()
                    .passes()
                    .get(execution_pass.graph_pass_index)
                    .is_some_and(|pass| {
                        pass.executor_id
                            .as_deref()
                            .is_some_and(|executor_id| executor_id.starts_with("sprite."))
                    })
            })
    })
}

#[cfg(test)]
mod tests {
    use super::{SPRITE_GRAPH_STAGES, active_sprite_graph_stages};
    use crate::core::framework::render::RenderPipelineHandle;
    use crate::graphics::pipeline::RenderPassStage;
    use crate::graphics::pipeline::{CompiledRenderPipeline, RenderGraphExecutionPassMetadata};
    use crate::render_graph::{PassFlags, QueueLane, RenderGraphBuilder};

    #[test]
    fn compiled_scene_sprite_stage_list_owns_core2d_product_stages() {
        assert!(SPRITE_GRAPH_STAGES.contains(&RenderPassStage::Opaque2d));
        assert!(SPRITE_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask2d));
        assert!(SPRITE_GRAPH_STAGES.contains(&RenderPassStage::Transparent2d));
        assert!(!SPRITE_GRAPH_STAGES.contains(&RenderPassStage::Deferred));
        assert!(!SPRITE_GRAPH_STAGES.contains(&RenderPassStage::Lighting));
        assert!(!SPRITE_GRAPH_STAGES.contains(&RenderPassStage::AlphaMask3d));
    }

    #[test]
    fn active_sprite_stage_selection_returns_an_iterator_without_collecting() {
        let source = include_str!("sprite_stage_selection.rs");
        let iterator_return = ["-> impl ", "Iterator<Item = RenderPassStage>"].concat();

        assert!(source.contains(&iterator_return));
    }

    #[test]
    fn active_sprite_graph_stages_follow_unculled_sprite_passes() {
        let pipeline = compiled_pipeline_with_passes([
            (RenderPassStage::Opaque2d, "sprite-opaque", "sprite.opaque"),
            (
                RenderPassStage::Transparent2d,
                "sprite-transparent",
                "sprite.transparent",
            ),
            (RenderPassStage::Ui, "runtime-ui", "ui.screen-space"),
        ]);

        assert_eq!(
            active_sprite_graph_stages(&pipeline).collect::<Vec<_>>(),
            vec![RenderPassStage::Opaque2d, RenderPassStage::Transparent2d]
        );
    }

    fn compiled_pipeline_with_passes<const N: usize>(
        passes: [(RenderPassStage, &str, &str); N],
    ) -> CompiledRenderPipeline {
        let mut graph = RenderGraphBuilder::new("sprite-stage-test");
        let mut execution_pass_metadata = Vec::new();
        for (stage, pass_name, executor_id) in passes {
            let pass =
                graph.add_pass_with_executor(pass_name, QueueLane::Graphics, Some(executor_id));
            // This fixture tests stage filtering only, so synthetic passes are rooted directly.
            graph
                .set_pass_flags(
                    pass,
                    PassFlags {
                        has_side_effects: true,
                        ..PassFlags::default()
                    },
                )
                .expect("sprite stage test root");
            execution_pass_metadata.push(RenderGraphExecutionPassMetadata::new(pass, stage));
        }

        CompiledRenderPipeline::from_parts(crate::graphics::pipeline::CompiledRenderPipelineParts {
            handle: RenderPipelineHandle::new(99),
            name: "sprite-stage-test".to_string(),
            renderer_name: "sprite-stage-test".to_string(),
            execution_pass_metadata,
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            environment_ibl_bake_request: None,
            ambient_occlusion_profile: None,
            half_resolution_transparency_depth_sigma:
                crate::core::framework::render::DEFAULT_HALF_RES_TRANSPARENCY_DEPTH_SIGMA,
            graph: graph.compile().expect("sprite stage test graph"),
        })
        .expect("sprite stage execution packet")
    }
}
