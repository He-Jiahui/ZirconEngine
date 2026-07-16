use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::ResourceId;

use crate::core::framework::render::ShadingModelId;

use super::geometry_source::GeometrySourceId;
use super::RenderShaderDefinitionValue;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RenderShaderVariantKey {
    pub entry_point: Option<String>,
    pub stage: Option<String>,
    pub defines: Vec<RenderShaderDefinitionValue>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderPassType {
    #[default]
    Forward,
    GBuffer,
    DepthPrepass,
    Shadow,
    Velocity,
    TaaReactiveMask,
}

impl ShaderPassType {
    pub const fn packed_value(self) -> u64 {
        match self {
            Self::Forward => 0,
            Self::GBuffer => 1,
            Self::DepthPrepass => 2,
            Self::Shadow => 3,
            Self::Velocity => 4,
            Self::TaaReactiveMask => 5,
        }
    }

    pub const fn token(self) -> &'static str {
        match self {
            Self::Forward => "forward",
            Self::GBuffer => "gbuffer",
            Self::DepthPrepass => "depth_prepass",
            Self::Shadow => "shadow",
            Self::Velocity => "velocity",
            Self::TaaReactiveMask => "taa_reactive_mask",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShaderFeatureBits(u32);

impl ShaderFeatureBits {
    pub const ALPHA_TEST: u32 = 1 << 0;
    pub const RECEIVE_SHADOWS: u32 = 1 << 1;
    pub const DOUBLE_SIDED: u32 = 1 << 2;
    pub const LOD_DITHER_CROSSFADE: u32 = 1 << 3;
    pub const INSTANCED_PREV_TRANSFORM: u32 = 1 << 4;
    pub const HAS_NORMAL_TEXTURE: u32 = 1 << 5;
    pub const PBR_CLEARCOAT: u32 = 1 << 6;
    pub const PBR_ANISOTROPY: u32 = 1 << 7;
    pub const PBR_TRANSMISSION: u32 = 1 << 8;
    pub const VOLUMETRIC_FOG: u32 = 1 << 9;

    pub const fn new(bits: u32) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, bit: u32) -> bool {
        (self.0 & bit) == bit
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderQualityTier {
    Low,
    #[default]
    Medium,
    High,
    Ultra,
}

impl ShaderQualityTier {
    pub const fn packed_value(self) -> u64 {
        match self {
            Self::Low => 0,
            Self::Medium => 1,
            Self::High => 2,
            Self::Ultra => 3,
        }
    }

    pub const fn token(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Ultra => "ultra",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderVariantKey {
    pub material_shader: ResourceId,
    pub material_revision: u64,
    pub material_layout_hash: u64,
    pub material_option_bits: u32,
    pub geometry_source: GeometrySourceId,
    pub shading_model: ShadingModelId,
    pub pass_type: ShaderPassType,
    pub features: ShaderFeatureBits,
    pub quality: ShaderQualityTier,
    pub platform_token: String,
}

impl ShaderVariantKey {
    pub fn packed_dims(&self) -> u64 {
        u64::from(self.geometry_source.value())
            | (u64::from(self.shading_model.value()) << 4)
            | (self.pass_type.packed_value() << 12)
            | (u64::from(self.features.bits()) << 16)
            | (self.quality.packed_value() << 48)
    }

    pub fn canonical_string(&self) -> String {
        format!(
            concat!(
                "shader_variant_v1",
                "|material={}",
                "|revision={}",
                "|layout={:#018x}",
                "|material_options={:#010x}",
                "|geometry={}",
                "|shading={}",
                "|pass={}",
                "|features={:#010x}",
                "|quality={}",
                "|platform={}"
            ),
            self.material_shader,
            self.material_revision,
            self.material_layout_hash,
            self.material_option_bits,
            self.geometry_source.value(),
            self.shading_model.value(),
            self.pass_type.token(),
            self.features.bits(),
            self.quality.token(),
            self.platform_token.trim()
        )
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::resource::ResourceId;

    use crate::core::framework::render::{
        GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
        SHADING_MODEL_ID_BLINN_PHONG,
    };

    #[test]
    fn render_shader_variant_key_packs_dimensions_stably() {
        let key = ShaderVariantKey {
            material_shader: ResourceId::from_stable_label("builtin://pbr"),
            material_revision: 7,
            material_layout_hash: 0,
            material_option_bits: 0x13,
            geometry_source: GeometrySourceId::new(3),
            shading_model: SHADING_MODEL_ID_BLINN_PHONG,
            pass_type: ShaderPassType::Velocity,
            features: ShaderFeatureBits::new(
                ShaderFeatureBits::ALPHA_TEST | ShaderFeatureBits::DOUBLE_SIDED,
            ),
            quality: ShaderQualityTier::High,
            platform_token: "wgpu-vulkan-downlevel-default".to_string(),
        };

        assert_eq!(
            key.packed_dims(),
            3 | (1 << 4) | (4 << 12) | (0b101 << 16) | (2 << 48)
        );
        assert_eq!(
            key.canonical_string(),
            format!(
                concat!(
                    "shader_variant_v1",
                    "|material={}",
                    "|revision=7",
                    "|layout=0x0000000000000000",
                    "|material_options=0x00000013",
                    "|geometry=3",
                    "|shading=1",
                    "|pass=velocity",
                    "|features=0x00000005",
                    "|quality=high",
                    "|platform=wgpu-vulkan-downlevel-default"
                ),
                ResourceId::from_stable_label("builtin://pbr")
            )
        );
    }

    #[test]
    fn render_shader_feature_bits_reports_named_flags() {
        let features = ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST).union(
            ShaderFeatureBits::new(ShaderFeatureBits::INSTANCED_PREV_TRANSFORM),
        );

        assert!(features.contains(ShaderFeatureBits::ALPHA_TEST));
        assert!(features.contains(ShaderFeatureBits::INSTANCED_PREV_TRANSFORM));
        assert!(!features.contains(ShaderFeatureBits::RECEIVE_SHADOWS));
    }

    #[test]
    fn render_shader_pass_type_names_taa_reactive_mask_separately_from_forward() {
        assert_eq!(ShaderPassType::Forward.packed_value(), 0);
        assert_eq!(ShaderPassType::TaaReactiveMask.packed_value(), 5);
        assert_eq!(ShaderPassType::TaaReactiveMask.token(), "taa_reactive_mask");
    }
}
