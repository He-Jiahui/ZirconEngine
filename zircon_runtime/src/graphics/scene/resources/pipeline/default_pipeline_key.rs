use crate::core::framework::render::SHADING_MODEL_ID_STANDARD_PBR;
use crate::core::resource::ResourceId;

use super::super::fallback_shader_uri;
use super::PipelineKey;

pub(crate) fn default_pipeline_key() -> PipelineKey {
    PipelineKey {
        shader_id: ResourceId::from_locator(&fallback_shader_uri()),
        shader_revision: 1,
        material_layout_hash: 0,
        material_option_bits: 0,
        double_sided: false,
        alpha_blend: false,
        alpha_mask: false,
        alpha_cutoff_bits: None,
        receive_shadows: true,
        shading_model_id: SHADING_MODEL_ID_STANDARD_PBR,
        unlit: false,
        has_base_color_texture: false,
        has_normal_texture: false,
        has_metallic_roughness_texture: false,
        has_occlusion_texture: false,
        has_emissive_texture: false,
    }
}
