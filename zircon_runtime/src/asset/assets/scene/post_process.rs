use crate::core::math::Real;
use serde::{Deserialize, Serialize};

use super::defaults::{
    default_bloom_threshold, default_color_white, default_one_real, default_true,
    default_vignette_smoothness,
};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneBloomSettingsAsset {
    #[serde(default = "default_bloom_threshold")]
    pub threshold: Real,
    #[serde(default)]
    pub intensity: Real,
    #[serde(default)]
    pub radius: Real,
}

impl Default for SceneBloomSettingsAsset {
    fn default() -> Self {
        Self {
            threshold: default_bloom_threshold(),
            intensity: 0.0,
            radius: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneColorGradingSettingsAsset {
    #[serde(default = "default_one_real")]
    pub exposure: Real,
    #[serde(default = "default_one_real")]
    pub contrast: Real,
    #[serde(default = "default_one_real")]
    pub saturation: Real,
    #[serde(default = "default_one_real")]
    pub gamma: Real,
    #[serde(default = "default_color_white")]
    pub tint: [Real; 3],
}

impl Default for SceneColorGradingSettingsAsset {
    fn default() -> Self {
        Self {
            exposure: default_one_real(),
            contrast: default_one_real(),
            saturation: default_one_real(),
            gamma: default_one_real(),
            tint: default_color_white(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SceneTonemapOperatorAsset {
    #[default]
    None,
    Reinhard,
    Aces,
    Filmic,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneTonemapSettingsAsset {
    #[serde(default)]
    pub operator: SceneTonemapOperatorAsset,
    #[serde(default)]
    pub exposure_bias: Real,
    #[serde(default = "default_one_real")]
    pub white_point: Real,
}

impl Default for SceneTonemapSettingsAsset {
    fn default() -> Self {
        Self {
            operator: SceneTonemapOperatorAsset::None,
            exposure_bias: 0.0,
            white_point: default_one_real(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneVignetteSettingsAsset {
    #[serde(default)]
    pub intensity: Real,
    #[serde(default = "default_vignette_smoothness")]
    pub smoothness: Real,
    #[serde(default = "default_one_real")]
    pub roundness: Real,
}

impl Default for SceneVignetteSettingsAsset {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            smoothness: default_vignette_smoothness(),
            roundness: default_one_real(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneFilmGrainSettingsAsset {
    #[serde(default)]
    pub intensity: Real,
    #[serde(default = "default_one_real")]
    pub response: Real,
}

impl Default for SceneFilmGrainSettingsAsset {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            response: default_one_real(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneDitherSettingsAsset {
    #[serde(default)]
    pub intensity: Real,
    #[serde(default = "default_one_real")]
    pub scale: Real,
}

impl Default for SceneDitherSettingsAsset {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            scale: default_one_real(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneChromaticAberrationSettingsAsset {
    #[serde(default)]
    pub intensity: Real,
    #[serde(default = "default_one_real")]
    pub sample_spread: Real,
}

impl Default for SceneChromaticAberrationSettingsAsset {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            sample_spread: default_one_real(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneFogSettingsAsset {
    #[serde(default)]
    pub density: Real,
    #[serde(default)]
    pub height_falloff: Real,
    #[serde(default = "default_color_white")]
    pub color: [Real; 3],
}

impl Default for SceneFogSettingsAsset {
    fn default() -> Self {
        Self {
            density: 0.0,
            height_falloff: 0.0,
            color: default_color_white(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ScenePostProcessEffectStackAsset {
    #[serde(default)]
    pub tonemap: SceneTonemapSettingsAsset,
    #[serde(default)]
    pub vignette: SceneVignetteSettingsAsset,
    #[serde(default)]
    pub grain: SceneFilmGrainSettingsAsset,
    #[serde(default)]
    pub dither: SceneDitherSettingsAsset,
    #[serde(default)]
    pub chromatic_aberration: SceneChromaticAberrationSettingsAsset,
    #[serde(default)]
    pub fog: SceneFogSettingsAsset,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ScenePostProcessSettingsAsset {
    #[serde(default)]
    pub bloom: SceneBloomSettingsAsset,
    #[serde(default)]
    pub color_grading: SceneColorGradingSettingsAsset,
    #[serde(default)]
    pub effect_stack: ScenePostProcessEffectStackAsset,
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ScenePostProcessVolumeProfileAsset {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bloom: Option<SceneBloomSettingsAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_grading: Option<SceneColorGradingSettingsAsset>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_stack: Option<ScenePostProcessEffectStackAsset>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScenePostProcessVolumeAsset {
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default = "default_true")]
    pub is_global: bool,
    #[serde(default)]
    pub priority: Real,
    #[serde(default = "default_one_real")]
    pub weight: Real,
    #[serde(default)]
    pub blend_distance: Real,
    #[serde(default)]
    pub profile: ScenePostProcessVolumeProfileAsset,
}

impl Default for ScenePostProcessVolumeAsset {
    fn default() -> Self {
        Self {
            active: true,
            is_global: true,
            priority: 0.0,
            weight: default_one_real(),
            blend_distance: 0.0,
            profile: ScenePostProcessVolumeProfileAsset::default(),
        }
    }
}
