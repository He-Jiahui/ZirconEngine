use crate::graphics::pipeline::RenderPassStage;
use crate::graphics::CompiledRenderPipeline;

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
#[path = "tests/sprite_stage_selection.rs"]
mod tests;
