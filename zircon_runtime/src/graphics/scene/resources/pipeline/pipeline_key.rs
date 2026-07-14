use crate::core::framework::render::{
    GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
    ShadingModelId, GEOMETRY_SOURCE_ID_STATIC_MESH,
};
use crate::core::resource::ResourceId;

use super::super::fallback_shader_uri;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PipelineKey {
    pub(crate) shader_id: ResourceId,
    pub(crate) shader_revision: u64,
    pub(crate) material_layout_hash: u64,
    pub(crate) material_option_bits: u32,
    pub(crate) double_sided: bool,
    pub(crate) alpha_blend: bool,
    pub(crate) alpha_mask: bool,
    pub(crate) alpha_cutoff_bits: Option<u32>,
    pub(crate) receive_shadows: bool,
    pub(crate) shading_model_id: ShadingModelId,
    pub(crate) unlit: bool,
    pub(crate) has_base_color_texture: bool,
    pub(crate) has_normal_texture: bool,
    pub(crate) has_metallic_roughness_texture: bool,
    pub(crate) has_occlusion_texture: bool,
    pub(crate) has_emissive_texture: bool,
    pub(crate) pbr_clearcoat: bool,
    pub(crate) pbr_anisotropy: bool,
    pub(crate) pbr_transmission: bool,
}

impl PipelineKey {
    pub(crate) fn is_transparent(&self) -> bool {
        self.alpha_blend
    }

    pub(crate) fn is_alpha_mask(&self) -> bool {
        self.alpha_mask && !self.alpha_blend
    }

    pub(crate) fn requires_forward_path(&self) -> bool {
        self.pbr_clearcoat || self.pbr_anisotropy || self.pbr_transmission
    }

    pub(crate) fn uses_fallback_shader(&self) -> bool {
        self.shader_id == ResourceId::from_locator(&fallback_shader_uri())
    }

    pub(crate) fn shader_variant_key(
        &self,
        pass_type: ShaderPassType,
        platform_token: impl Into<String>,
    ) -> ShaderVariantKey {
        self.shader_variant_key_for_geometry(
            pass_type,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            platform_token,
        )
    }

    pub(crate) fn shader_variant_key_for_geometry(
        &self,
        pass_type: ShaderPassType,
        geometry_source: GeometrySourceId,
        platform_token: impl Into<String>,
    ) -> ShaderVariantKey {
        ShaderVariantKey {
            material_shader: self.shader_id,
            material_revision: self.shader_revision,
            material_layout_hash: self.material_layout_hash,
            material_option_bits: self.material_option_bits,
            geometry_source,
            shading_model: self.shading_model_id,
            pass_type,
            features: self.shader_feature_bits(),
            quality: ShaderQualityTier::Medium,
            platform_token: platform_token.into(),
        }
    }

    pub(crate) fn shader_feature_bits(&self) -> ShaderFeatureBits {
        let mut bits = 0;
        if self.alpha_mask {
            bits |= ShaderFeatureBits::ALPHA_TEST;
        }
        if self.double_sided {
            bits |= ShaderFeatureBits::DOUBLE_SIDED;
        }
        if self.receive_shadows {
            bits |= ShaderFeatureBits::RECEIVE_SHADOWS;
        }
        if self.has_normal_texture {
            bits |= ShaderFeatureBits::HAS_NORMAL_TEXTURE;
        }
        if self.pbr_clearcoat {
            bits |= ShaderFeatureBits::PBR_CLEARCOAT;
        }
        if self.pbr_anisotropy {
            bits |= ShaderFeatureBits::PBR_ANISOTROPY;
        }
        if self.pbr_transmission {
            bits |= ShaderFeatureBits::PBR_TRANSMISSION;
        }
        ShaderFeatureBits::new(bits)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        ShaderFeatureBits, ShaderPassType, SHADING_MODEL_ID_STANDARD_PBR,
    };
    use crate::core::resource::ResourceId;
    use crate::graphics::scene::resources::default_pipeline_key;

    #[test]
    fn pipeline_key_identifies_builtin_fallback_shader() {
        let mut key = default_pipeline_key();
        assert!(key.uses_fallback_shader());

        key.shader_id = ResourceId::from_stable_label("res://shaders/custom-material.wgsl");
        assert!(!key.uses_fallback_shader());
    }

    #[test]
    fn pipeline_key_derives_material_shader_variant_key() {
        let mut key = default_pipeline_key();
        key.shader_revision = 42;
        key.double_sided = true;
        key.alpha_mask = true;
        key.receive_shadows = true;

        let variant = key.shader_variant_key(ShaderPassType::GBuffer, "wgpu-test");

        assert_eq!(variant.material_shader, key.shader_id);
        assert_eq!(variant.material_revision, 42);
        assert_eq!(variant.shading_model, SHADING_MODEL_ID_STANDARD_PBR);
        assert_eq!(variant.pass_type, ShaderPassType::GBuffer);
        assert_eq!(variant.platform_token, "wgpu-test");
        assert!(variant.features.contains(ShaderFeatureBits::ALPHA_TEST));
        assert!(variant.features.contains(ShaderFeatureBits::DOUBLE_SIDED));
        assert!(variant
            .features
            .contains(ShaderFeatureBits::RECEIVE_SHADOWS));
        assert!(!variant
            .features
            .contains(ShaderFeatureBits::HAS_NORMAL_TEXTURE));
    }

    #[test]
    fn pipeline_key_derives_normal_texture_shader_feature() {
        let mut key = default_pipeline_key();
        key.has_normal_texture = true;

        let variant = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

        assert!(variant
            .features
            .contains(ShaderFeatureBits::HAS_NORMAL_TEXTURE));
    }

    #[test]
    fn pipeline_key_can_disable_receive_shadow_shader_feature() {
        let mut key = default_pipeline_key();
        key.receive_shadows = false;

        let variant = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

        assert!(!variant
            .features
            .contains(ShaderFeatureBits::RECEIVE_SHADOWS));
    }

    #[test]
    fn render_advanced_material_pipeline_key_tracks_authored_lobes() {
        let mut key = default_pipeline_key();

        let default_variant = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");
        assert!(!default_variant
            .features
            .contains(ShaderFeatureBits::PBR_CLEARCOAT));
        assert!(!default_variant
            .features
            .contains(ShaderFeatureBits::PBR_ANISOTROPY));
        assert!(!default_variant
            .features
            .contains(ShaderFeatureBits::PBR_TRANSMISSION));

        key.pbr_clearcoat = true;
        key.pbr_anisotropy = true;
        key.pbr_transmission = true;
        let advanced_variant = key.shader_variant_key(ShaderPassType::Forward, "wgpu-test");

        assert!(advanced_variant
            .features
            .contains(ShaderFeatureBits::PBR_CLEARCOAT));
        assert!(advanced_variant
            .features
            .contains(ShaderFeatureBits::PBR_ANISOTROPY));
        assert!(advanced_variant
            .features
            .contains(ShaderFeatureBits::PBR_TRANSMISSION));
    }
}
