use serde::{Deserialize, Serialize};

use crate::core::framework::render::RenderQueueValue;
use crate::core::resource::AssetReference;

pub const STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS: f32 = 0.5;
pub const STANDARD_PBR_DEFAULT_IOR: f32 = 1.5;
pub const STANDARD_PBR_TRANSMISSION_RENDER_QUEUE: RenderQueueValue = RenderQueueValue::new(2_900);

/// Finite serialization-safe equivalent of an unbounded attenuation distance.
pub const STANDARD_PBR_NO_ATTENUATION_DISTANCE: f32 = f32::MAX;

/// Forward-only Standard PBR extensions consumed by the material pipeline.
///
/// Zero-valued lobe strengths preserve the baseline Standard PBR variant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StandardPbrMaterialFeatures {
    pub clearcoat: f32,
    pub clearcoat_perceptual_roughness: f32,
    pub clearcoat_normal_texture: Option<AssetReference>,
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

    pub fn requires_forward_path(&self) -> bool {
        self.uses_clearcoat() || self.uses_anisotropy() || self.uses_transmission()
    }

    pub fn requires_scene_color_copy(&self) -> bool {
        is_active_strength(self.specular_transmission)
    }

    pub fn normalized(&self) -> Self {
        Self {
            clearcoat: normalized_unit(self.clearcoat, 0.0),
            clearcoat_perceptual_roughness: normalized_unit(
                self.clearcoat_perceptual_roughness,
                STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS,
            ),
            clearcoat_normal_texture: self.clearcoat_normal_texture.clone(),
            anisotropy_strength: normalized_unit(self.anisotropy_strength, 0.0),
            anisotropy_rotation: normalized_finite(self.anisotropy_rotation, 0.0),
            specular_transmission: normalized_unit(self.specular_transmission, 0.0),
            diffuse_transmission: normalized_unit(self.diffuse_transmission, 0.0),
            thickness: normalized_nonnegative(self.thickness, 0.0),
            ior: normalized_finite(self.ior, STANDARD_PBR_DEFAULT_IOR).max(1.0),
            attenuation_color: self
                .attenuation_color
                .map(|channel| normalized_unit(channel, 1.0)),
            attenuation_distance: if self.attenuation_distance.is_finite()
                && self.attenuation_distance > 0.0
            {
                self.attenuation_distance
            } else {
                STANDARD_PBR_NO_ATTENUATION_DISTANCE
            },
        }
    }
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
    if value.is_finite() { value } else { fallback }
}

#[cfg(test)]
mod tests {
    use super::{
        STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS, STANDARD_PBR_DEFAULT_IOR,
        STANDARD_PBR_NO_ATTENUATION_DISTANCE, STANDARD_PBR_TRANSMISSION_RENDER_QUEUE,
        StandardPbrMaterialFeatures,
    };
    use crate::core::framework::render::{CorePipelineKind, RenderPhase, RenderQueueValue};

    #[test]
    fn render_advanced_material_features_default_has_no_feature_work() {
        let features = StandardPbrMaterialFeatures::default();

        assert_eq!(
            features.clearcoat_perceptual_roughness,
            STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS
        );
        assert_eq!(features.ior, STANDARD_PBR_DEFAULT_IOR);
        assert_eq!(
            features.attenuation_distance,
            STANDARD_PBR_NO_ATTENUATION_DISTANCE
        );
        assert!(features.is_default());
        assert!(!features.uses_clearcoat());
        assert!(!features.uses_anisotropy());
        assert!(!features.uses_transmission());
        assert!(!features.requires_forward_path());
        assert!(!features.requires_scene_color_copy());

        let encoded = toml::to_string(&features).expect("default material features serialize");
        let decoded: StandardPbrMaterialFeatures =
            toml::from_str(&encoded).expect("default material features deserialize");
        assert_eq!(decoded, features);
    }

    #[test]
    fn render_advanced_material_features_enable_only_authored_lobes() {
        let clearcoat = StandardPbrMaterialFeatures {
            clearcoat: 0.75,
            ..Default::default()
        };
        assert!(clearcoat.uses_clearcoat());
        assert!(clearcoat.requires_forward_path());
        assert!(!clearcoat.requires_scene_color_copy());

        let anisotropy = StandardPbrMaterialFeatures {
            anisotropy_strength: 0.5,
            anisotropy_rotation: 1.25,
            ..Default::default()
        };
        assert!(anisotropy.uses_anisotropy());
        assert!(anisotropy.requires_forward_path());
        assert!(!anisotropy.requires_scene_color_copy());

        let diffuse_transmission = StandardPbrMaterialFeatures {
            diffuse_transmission: 0.25,
            ..Default::default()
        };
        assert!(diffuse_transmission.uses_transmission());
        assert!(diffuse_transmission.requires_forward_path());
        assert!(!diffuse_transmission.requires_scene_color_copy());

        let specular_transmission = StandardPbrMaterialFeatures {
            specular_transmission: 0.5,
            ..Default::default()
        };
        assert!(specular_transmission.uses_transmission());
        assert!(specular_transmission.requires_forward_path());
        assert!(specular_transmission.requires_scene_color_copy());
    }

    #[test]
    fn render_advanced_material_features_normalize_invalid_values() {
        let resolved = StandardPbrMaterialFeatures {
            clearcoat: 2.0,
            clearcoat_perceptual_roughness: f32::NAN,
            anisotropy_strength: -1.0,
            anisotropy_rotation: f32::INFINITY,
            specular_transmission: 1.5,
            diffuse_transmission: f32::NAN,
            thickness: -4.0,
            ior: 0.5,
            attenuation_color: [2.0, -1.0, f32::NAN],
            attenuation_distance: f32::INFINITY,
            ..Default::default()
        }
        .normalized();

        assert_eq!(resolved.clearcoat, 1.0);
        assert_eq!(
            resolved.clearcoat_perceptual_roughness,
            STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS
        );
        assert_eq!(resolved.anisotropy_strength, 0.0);
        assert_eq!(resolved.anisotropy_rotation, 0.0);
        assert_eq!(resolved.specular_transmission, 1.0);
        assert_eq!(resolved.diffuse_transmission, 0.0);
        assert_eq!(resolved.thickness, 0.0);
        assert_eq!(resolved.ior, 1.0);
        assert_eq!(resolved.attenuation_color, [1.0, 0.0, 1.0]);
        assert_eq!(
            resolved.attenuation_distance,
            STANDARD_PBR_NO_ATTENUATION_DISTANCE
        );
    }

    #[test]
    fn render_transmission_queue_value_is_2900_in_transparent_band() {
        assert_eq!(STANDARD_PBR_TRANSMISSION_RENDER_QUEUE.raw(), 2_900);
        assert_eq!(
            STANDARD_PBR_TRANSMISSION_RENDER_QUEUE.phase(CorePipelineKind::Core3d),
            RenderPhase::Transparent3d
        );
        assert!(STANDARD_PBR_TRANSMISSION_RENDER_QUEUE < RenderQueueValue::TRANSPARENT);
    }
}
