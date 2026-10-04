use crate::core::framework::render::{
    GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
    ShadingModelId, GEOMETRY_SOURCE_ID_STATIC_MESH,
};
use crate::core::resource::{ResourceId, ResourceReadinessRowIdentity};

use super::super::fallback_shader_uri;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PipelineKey {
    pub(crate) shader_id: ResourceId,
    pub(crate) shader_revision: u64,
    /// Retained publication for the shader's complete resource dependency closure.
    /// `None` identifies the engine-owned static fallback before asset preparation.
    /// Persistent cache identity remains content-addressed by assembled WGSL.
    pub(crate) shader_dependency_identity: Option<ResourceReadinessRowIdentity>,
    pub(crate) material_layout_hash: u64,
    pub(crate) material_option_bits: u32,
    pub(crate) double_sided: bool,
    /// Reverses the raster front-face convention for negative-determinant instances.
    /// This is PSO state and must not enter persistent shader variant identity.
    pub(crate) reverse_raster_winding: bool,
    pub(crate) alpha_blend: bool,
    pub(crate) alpha_mask: bool,
    pub(crate) alpha_cutoff_bits: Option<u32>,
    pub(crate) receive_shadows: bool,
    pub(crate) shading_model_id: ShadingModelId,
    pub(crate) unlit: bool,
    /// Normal mapping changes generated shader code and vertex requirements.
    pub(crate) has_normal_texture: bool,
    pub(crate) pbr_clearcoat: bool,
    pub(crate) pbr_anisotropy: bool,
    /// Routes a non-default dielectric F0 away from the fixed deferred GBuffer.
    /// It is intentionally not a shader specialization bit.
    pub(crate) pbr_ior_override: bool,
    pub(crate) pbr_transmission: bool,
    pub(crate) volumetric_fog: bool,
}

impl PipelineKey {
    pub(crate) fn is_transparent(&self) -> bool {
        self.alpha_blend
    }

    pub(crate) fn is_alpha_mask(&self) -> bool {
        self.alpha_mask && !self.alpha_blend
    }

    pub(crate) fn requires_forward_path(&self) -> bool {
        self.pbr_clearcoat || self.pbr_anisotropy || self.pbr_ior_override || self.pbr_transmission
    }

    /// Returns the PSO identity after removing draw-list-only routing state.
    pub(crate) fn pipeline_variant_identity(&self) -> Self {
        let mut identity = self.clone();
        identity.pbr_ior_override = false;
        identity
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
        if self.volumetric_fog {
            bits |= ShaderFeatureBits::VOLUMETRIC_FOG;
        }
        ShaderFeatureBits::new(bits)
    }
}

#[cfg(test)]
#[path = "tests/pipeline_key.rs"]
mod tests;
