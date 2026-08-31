use super::super::{
    build_sprite_phase_queue, CorePipelineKind, RenderSpriteSnapshot, SpriteExtract,
    SpritePhaseInput,
};
use super::{resolved_phase_queue, SpritePhaseExtractInput};

impl SpriteExtract {
    pub fn from_sprites(
        core_pipeline: CorePipelineKind,
        sprites: Vec<RenderSpriteSnapshot>,
    ) -> Self {
        let phase_inputs = sprites
            .iter()
            .enumerate()
            .map(|(sprite_index, sprite)| {
                SpritePhaseExtractInput::new(
                    sprite.entity,
                    sprite_index,
                    sprite.material_alpha_mode,
                    sprite.z_order,
                    sprite.transform.translation.z,
                )
            })
            .collect::<Vec<_>>();
        Self::from_sprites_and_phase_inputs(core_pipeline, sprites, phase_inputs)
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
