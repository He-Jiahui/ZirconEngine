use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderQueueValue;
use crate::core::resource::AssetReference;

pub const STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS: f32 = 0.5;
pub const STANDARD_PBR_DEFAULT_IOR: f32 = 1.5;
pub const STANDARD_PBR_DEFAULT_DIELECTRIC_F0: f32 = 0.04;
pub const STANDARD_PBR_TRANSMISSION_RENDER_QUEUE: RenderQueueValue = RenderQueueValue::new(2_900);

/// Finite serialization- and GPU-safe equivalent of an unbounded attenuation distance.
pub const STANDARD_PBR_NO_ATTENUATION_DISTANCE: f32 = 1.0e30;

/// Forward-only Standard PBR extensions consumed by the material pipeline.
///
/// Zero-valued lobe strengths preserve the baseline Standard PBR variant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StandardPbrMaterialFeatures {
    pub clearcoat: f32,
    pub clearcoat_perceptual_roughness: f32,
    pub clearcoat_normal_texture: Option<AssetReference>,
    pub clearcoat_normal_scale: f32,
    pub anisotropy_strength: f32,
    pub anisotropy_rotation: f32,
    pub specular_transmission: f32,
    pub diffuse_transmission: f32,
    pub thickness: f32,
    pub ior: f32,
    pub attenuation_color: [f32; 3],
    pub attenuation_distance: f32,
}

impl Default for StandardPbrMaterialFeatures {
    fn default() -> Self {
        Self {
            clearcoat: 0.0,
            clearcoat_perceptual_roughness: STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS,
            clearcoat_normal_texture: None,
            clearcoat_normal_scale: 1.0,
            anisotropy_strength: 0.0,
            anisotropy_rotation: 0.0,
            specular_transmission: 0.0,
            diffuse_transmission: 0.0,
            thickness: 0.0,
            ior: STANDARD_PBR_DEFAULT_IOR,
            attenuation_color: [1.0; 3],
            attenuation_distance: STANDARD_PBR_NO_ATTENUATION_DISTANCE,
        }
    }
}

impl StandardPbrMaterialFeatures {
    pub fn is_default(&self) -> bool {
        self == &Self::default()
    }

    pub fn uses_clearcoat(&self) -> bool {
        is_active_strength(self.clearcoat)
    }

    pub fn uses_anisotropy(&self) -> bool {
        is_active_strength(self.anisotropy_strength)
    }

    pub fn uses_transmission(&self) -> bool {
        is_active_strength(self.specular_transmission)
            || is_active_strength(self.diffuse_transmission)
    }

    /// Dielectric reflectance at normal incidence derived from the normalized IOR.
    pub fn dielectric_f0(&self) -> f32 {
        let ior = normalized_ior(self.ior);
        if ior.to_bits() == STANDARD_PBR_DEFAULT_IOR.to_bits() {
            return STANDARD_PBR_DEFAULT_DIELECTRIC_F0;
        }
        let ratio = (ior - 1.0) / (ior + 1.0);
        ratio * ratio
    }

    /// A non-default F0 cannot be represented by the current deferred GBuffer.
    pub fn uses_dielectric_f0_override(&self) -> bool {
        normalized_ior(self.ior).to_bits() != STANDARD_PBR_DEFAULT_IOR.to_bits()
    }

    pub fn requires_forward_path(&self) -> bool {
        self.uses_clearcoat()
            || self.uses_anisotropy()
            || self.uses_transmission()
            || self.uses_dielectric_f0_override()
    }

    pub fn requires_scene_color_copy(&self) -> bool {
        is_active_strength(self.specular_transmission)
    }

    /// 资产属性经继承解析后在此归一化，再用于可见材质统计、变体判定和 GPU 参数。
    /// 调用方应使用返回值，避免同一材质的非法数值在不同阶段产生不同解释。
    pub fn normalized(&self) -> Self {
        Self {
            clearcoat: normalized_unit(self.clearcoat, 0.0),
            clearcoat_perceptual_roughness: normalized_unit(
                self.clearcoat_perceptual_roughness,
                STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS,
            ),
            clearcoat_normal_texture: self.clearcoat_normal_texture.clone(),
            clearcoat_normal_scale: normalized_finite(self.clearcoat_normal_scale, 1.0),
            anisotropy_strength: normalized_unit(self.anisotropy_strength, 0.0),
            anisotropy_rotation: normalized_finite(self.anisotropy_rotation, 0.0),
            specular_transmission: normalized_unit(self.specular_transmission, 0.0),
            diffuse_transmission: normalized_unit(self.diffuse_transmission, 0.0),
            thickness: normalized_nonnegative(self.thickness, 0.0),
            ior: normalized_ior(self.ior),
            attenuation_color: self
                .attenuation_color
                .map(|channel| normalized_unit(channel, 1.0)),
            attenuation_distance: if self.attenuation_distance.is_finite()
                && self.attenuation_distance > 0.0
            {
                self.attenuation_distance
                    .min(STANDARD_PBR_NO_ATTENUATION_DISTANCE)
            } else {
                STANDARD_PBR_NO_ATTENUATION_DISTANCE
            },
        }
    }
}

fn normalized_ior(value: f32) -> f32 {
    normalized_finite(value, STANDARD_PBR_DEFAULT_IOR).max(1.0)
}

fn is_active_strength(value: f32) -> bool {
    value.is_finite() && value > 0.0
}

fn normalized_unit(value: f32, fallback: f32) -> f32 {
    normalized_finite(value, fallback).clamp(0.0, 1.0)
}

fn normalized_nonnegative(value: f32, fallback: f32) -> f32 {
    normalized_finite(value, fallback).max(0.0)
}

fn normalized_finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
#[path = "tests/material_features.rs"]
mod tests;
