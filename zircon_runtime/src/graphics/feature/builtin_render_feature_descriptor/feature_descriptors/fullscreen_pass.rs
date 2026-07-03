use crate::core::framework::render::{
    FullscreenPassBuilder, FullscreenPassPlan, FullscreenShaderRef, PostProcessGraphResourceNames,
    RenderShaderEntryPointDescriptor, RenderShaderStage, ShaderAssetKind, ShaderResourceAccess,
    ShaderResourceDescriptor, ShaderResourceKind,
};

const MOTION_VECTOR_TILE_MAX_SHADER: &str = "builtin://shaders/fullscreen/motion_vector_tile_max";
const MOTION_VECTOR_TILE_MAX_FRAGMENT: &str = "fs_main";

pub(super) fn motion_vector_tile_max_pass_plan() -> FullscreenPassPlan {
    let mut builder = FullscreenPassBuilder::new(
        FullscreenShaderRef::from_locator_str(
            MOTION_VECTOR_TILE_MAX_SHADER,
            MOTION_VECTOR_TILE_MAX_FRAGMENT,
        )
        .expect("builtin motion vector tile max fullscreen shader locator must be valid"),
    );
    builder
        .with_pipeline_label("zircon-motion-vector-tile-max-fullscreen")
        .bind_texture(PostProcessGraphResourceNames::SCENE_VELOCITY);

    builder
        .build(
            ShaderAssetKind::Fullscreen,
            &[RenderShaderEntryPointDescriptor {
                name: MOTION_VECTOR_TILE_MAX_FRAGMENT.to_string(),
                stage: RenderShaderStage::Fragment,
            }],
            &[ShaderResourceDescriptor {
                name: PostProcessGraphResourceNames::SCENE_VELOCITY.to_string(),
                kind: ShaderResourceKind::Texture,
                access: Some(ShaderResourceAccess::Read),
            }],
        )
        .expect("builtin motion vector tile max fullscreen contract must be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        FULLSCREEN_PASS_INPUT_GROUP, FULLSCREEN_TRIANGLE_VERTEX_ENTRY,
    };

    #[test]
    fn motion_vector_tile_max_fullscreen_plan_declares_scene_velocity_input() {
        let plan = motion_vector_tile_max_pass_plan();

        assert_eq!(plan.vertex_entry, FULLSCREEN_TRIANGLE_VERTEX_ENTRY);
        assert_eq!(
            plan.pipeline_label,
            "zircon-motion-vector-tile-max-fullscreen"
        );
        assert_eq!(plan.resources.len(), 1);
        assert_eq!(
            plan.resources[0].name,
            PostProcessGraphResourceNames::SCENE_VELOCITY
        );
        assert_eq!(plan.resources[0].abi.group, FULLSCREEN_PASS_INPUT_GROUP);
        assert_eq!(plan.resources[0].abi.binding, 0);
    }
}
