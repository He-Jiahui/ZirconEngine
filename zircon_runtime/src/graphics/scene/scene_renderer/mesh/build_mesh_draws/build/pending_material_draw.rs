use std::sync::Arc;

use crate::core::framework::render::{
    CastShadowsMode, RenderMaterialPropertyUniformPayload, RendererCommon,
};
use crate::core::math::Vec4;
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::{
    GpuMaterialUniformResource, MaterialDisabledPasses, PipelineKey,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MaterialTextureSet;

/// Material state consumed by one pending draw and projected into command-cache identity.
///
/// This is deliberately one field on `PendingMeshDraw`: context-qualified last-good selection
/// can replace it as a unit before GPUScene and cached command construction.
#[derive(Clone)]
pub(super) struct PendingMaterialDraw {
    pub(super) resource_id: ResourceId,
    pub(super) draw_generation: Option<u64>,
    pub(super) textures: MaterialTextureSet,
    pub(super) uniform: Arc<GpuMaterialUniformResource>,
    pub(super) uniform_override_payload: Option<RenderMaterialPropertyUniformPayload>,
    pub(super) standard_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) pipeline_key: PipelineKey,
    pub(super) common: Arc<RendererCommon>,
    pub(super) renderer_cast_shadows: CastShadowsMode,
    pub(super) disabled_passes: MaterialDisabledPasses,
    pub(super) taa_reactive_mask_strength: f32,
    pub(super) half_resolution_transparency: bool,
    pub(super) draw_tint: Vec4,
}

#[cfg(test)]
mod tests {
    #[test]
    fn pending_mesh_draw_owns_material_state_as_one_replaceable_field() {
        let pending_mesh_draw = include_str!("pending_mesh_draw.rs");
        let body = pending_mesh_draw
            .split_once("pub(super) struct PendingMeshDraw {")
            .and_then(|(_, body)| body.split_once("\n}"))
            .map(|(body, _)| body)
            .expect("pending mesh draw declaration");

        assert!(body.contains("material: PendingMaterialDraw"));
        assert!(!body.contains("material_uniform:"));
        assert!(!body.contains("pipeline_key:"));
        assert!(!body.contains("disabled_passes:"));
    }

    #[test]
    fn pending_material_keeps_the_unhashed_draw_generation() {
        let source = include_str!("pending_material_draw.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("pending material draw test boundary");

        assert!(source.contains("draw_generation: Option<u64>"));
    }

    #[test]
    fn pending_material_preserves_renderer_authored_shadow_mode_before_material_merge() {
        let pending_material = include_str!("pending_material_draw.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("pending material draw test boundary");
        let builder = include_str!("extend_pending_draws_for_mesh_instance.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("pending material builder test boundary");

        assert!(pending_material.contains("renderer_cast_shadows: CastShadowsMode"));
        assert!(builder.contains("renderer_cast_shadows: mesh_instance.common.cast_shadows"));
    }
}
