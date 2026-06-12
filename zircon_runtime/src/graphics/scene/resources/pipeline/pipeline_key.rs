use crate::core::framework::render::RenderMaterialLightingModel;
use crate::core::resource::ResourceId;

use super::super::fallback_shader_uri;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PipelineKey {
    pub(crate) shader_id: ResourceId,
    pub(crate) shader_revision: u64,
    pub(crate) double_sided: bool,
    pub(crate) alpha_blend: bool,
    pub(crate) alpha_mask: bool,
    pub(crate) alpha_cutoff_bits: Option<u32>,
    pub(crate) lighting_model: RenderMaterialLightingModel,
    pub(crate) unlit: bool,
    pub(crate) has_base_color_texture: bool,
    pub(crate) has_normal_texture: bool,
    pub(crate) has_metallic_roughness_texture: bool,
    pub(crate) has_occlusion_texture: bool,
    pub(crate) has_emissive_texture: bool,
}

impl PipelineKey {
    pub(crate) fn is_transparent(&self) -> bool {
        self.alpha_blend
    }

    pub(crate) fn is_alpha_mask(&self) -> bool {
        self.alpha_mask && !self.alpha_blend
    }

    pub(crate) fn uses_fallback_shader(&self) -> bool {
        self.shader_id == ResourceId::from_locator(&fallback_shader_uri())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::resource::ResourceId;
    use crate::graphics::scene::resources::default_pipeline_key;

    #[test]
    fn pipeline_key_identifies_builtin_fallback_shader() {
        let mut key = default_pipeline_key();
        assert!(key.uses_fallback_shader());

        key.shader_id = ResourceId::from_stable_label("res://shaders/custom-material.wgsl");
        assert!(!key.uses_fallback_shader());
    }
}
