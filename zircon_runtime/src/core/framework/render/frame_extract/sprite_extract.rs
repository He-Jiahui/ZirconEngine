use super::super::{
    build_sprite_phase_queue, CorePipelineKind, RenderSpriteSnapshot, SpriteExtract,
    SpritePhaseInput,
};
use super::{resolved_phase_queue, SpritePhaseExtractInput};

// 默认入口从快照字段派生阶段输入并构建 phase queue；显式相位入口可保留上游已经解析的排序策略。
impl SpriteExtract {
    pub fn from_sprites(
        core_pipeline: CorePipelineKind,
        sprites: Vec<RenderSpriteSnapshot>,
    ) -> Self {
        let phase_queue = build_sprite_phase_queue(
            core_pipeline,
            sprites
                .iter()
                .enumerate()
                .map(|(sprite_index, sprite)| SpritePhaseInput {
                    entity: sprite.entity,
                    sprite_index,
                    queue: resolved_phase_queue(&sprite.material_alpha_mode, 0, 0),
                    z_order: sprite.z_order,
                    depth: sprite.transform.translation.z,
                    depth_bias: 0.0,
                    camera_order: 0,
                    sorting_layer: 0,
                    y_sort: None,
                    ui_z_index: 0,
                }),
        );
        Self {
            sprites,
            phase_queue,
        }
    }

    pub fn from_sprites_and_phase_inputs(
        core_pipeline: CorePipelineKind,
        sprites: Vec<RenderSpriteSnapshot>,
        phase_inputs: Vec<SpritePhaseExtractInput>,
    ) -> Self {
        let phase_queue = build_sprite_phase_queue(
            core_pipeline,
            phase_inputs.iter().map(|input| SpritePhaseInput {
                entity: input.entity,
                sprite_index: input.sprite_index,
                queue: resolved_phase_queue(
                    &input.material_alpha_mode,
                    input.render_queue,
                    input.material_queue,
                ),
                z_order: input.z_order,
                depth: input.depth,
                depth_bias: input.depth_bias,
                camera_order: 0,
                sorting_layer: 0,
                y_sort: None,
                ui_z_index: input.ui_z_index,
            }),
        );

        Self {
            sprites,
            phase_queue,
        }
    }
}

#[cfg(test)]
#[path = "sprite_extract/tests/optimization_batch_ht_runtime602_tests.rs"]
mod optimization_batch_ht_runtime602_tests;
