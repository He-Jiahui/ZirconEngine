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
#[path = "tests/pending_material_draw.rs"]
mod tests;
