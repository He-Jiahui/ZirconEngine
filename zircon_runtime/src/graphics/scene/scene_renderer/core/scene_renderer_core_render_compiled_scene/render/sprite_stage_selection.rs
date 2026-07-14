use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::CompiledRenderPipeline;

const SPRITE_GRAPH_STAGES: &[RenderPassStage] = &[
    RenderPassStage::Opaque2d,
    RenderPassStage::AlphaMask2d,
    RenderPassStage::Transparent2d,
];
pub(super) fn active_sprite_graph_stages(
    pipeline: &CompiledRenderPipeline,
) -> Vec<RenderPassStage> {
    SPRITE_GRAPH_STAGES
        .iter()
        .copied()
        .filter(|stage| pipeline_has_active_sprite_stage(pipeline, *stage))
        .collect()
}

fn pipeline_has_active_sprite_stage(
    pipeline: &CompiledRenderPipeline,
    stage: RenderPassStage,
) -> bool {
    pipeline
        .pass_stages
        .iter()
        .filter(|stage_entry| stage_entry.stage == stage)
        .any(|stage_entry| {
            pipeline.graph.passes().iter().any(|pass| {
                pass.name == stage_entry.pass_name
                    && !pass.culled
                    && pass
                        .executor_id
                        .as_deref()
                        .is_some_and(|executor_id| executor_id.starts_with("sprite."))
            })
        })
}

#[cfg(test)]
mod tests {
    use super::{active_sprite_graph_stages, SPRITE_GRAPH_STAGES};
    use crate::core::framework::render::RenderPipelineHandle;
    use crate::graphics::pipeline::RenderPassStage;
    use crate::graphics::pipeline::{CompiledRenderPipeline, CompiledRenderPipelinePassStage};
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
            active_sprite_graph_stages(&pipeline),
            vec![RenderPassStage::Opaque2d, RenderPassStage::Transparent2d]
        );
    }

    fn compiled_pipeline_with_passes<const N: usize>(
        passes: [(RenderPassStage, &str, &str); N],
    ) -> CompiledRenderPipeline {
        let mut graph = RenderGraphBuilder::new("sprite-stage-test");
        let mut pass_stages = Vec::new();
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
            pass_stages.push(CompiledRenderPipelinePassStage::new(pass_name, stage));
        }

        CompiledRenderPipeline {
            handle: RenderPipelineHandle::new(99),
            name: "sprite-stage-test".to_string(),
            renderer_name: "sprite-stage-test".to_string(),
            stages: SPRITE_GRAPH_STAGES.to_vec(),
            pass_stages,
            enabled_features: Vec::new(),
            required_extract_sections: Vec::new(),
            capability_requirements: Vec::new(),
            history_bindings: Vec::new(),
            environment_ibl_bake_request: None,
            graph: graph.compile().expect("sprite stage test graph"),
        }
    }
}
