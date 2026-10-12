use crate::core::math::{Real, Vec3};

const MIN_VIGNETTE_SMOOTHNESS: Real = 0.001;
const MIN_VIGNETTE_ROUNDNESS: Real = 0.001;
const MIN_DITHER_SCALE: Real = 0.001;

/// 晕影属于显示映射阶段的合成风格参数，由统一后处理通道消费。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderVignetteSettings {
    pub intensity: Real,
    pub smoothness: Real,
    pub roundness: Real,
}

impl Default for RenderVignetteSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            smoothness: 0.5,
            roundness: 1.0,
        }
    }
}

impl RenderVignetteSettings {
    pub fn is_enabled(self) -> bool {
        self.intensity > 0.0
    }

    pub fn render_intensity(self) -> Real {
        self.intensity.max(0.0)
    }

    pub fn render_smoothness(self) -> Real {
        self.smoothness.max(MIN_VIGNETTE_SMOOTHNESS)
    }

    pub fn render_roundness(self) -> Real {
        self.roundness.max(MIN_VIGNETTE_ROUNDNESS)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFilmGrainSettings {
    pub intensity: Real,
    pub response: Real,
}

impl Default for RenderFilmGrainSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            response: 1.0,
        }
    }
}

impl RenderFilmGrainSettings {
    pub fn is_enabled(self) -> bool {
        self.intensity > 0.0
    }

    pub fn render_intensity(self) -> Real {
        self.intensity.max(0.0)
    }

    pub fn render_response(self) -> Real {
        self.response.max(0.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDitherSettings {
    pub intensity: Real,
    pub scale: Real,
}

impl Default for RenderDitherSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            scale: 1.0,
        }
    }
}

impl RenderDitherSettings {
    pub fn is_enabled(self) -> bool {
        self.intensity > 0.0
    }

    pub fn render_intensity(self) -> Real {
        self.intensity.max(0.0)
    }

    pub fn render_scale(self) -> Real {
        self.scale.max(MIN_DITHER_SCALE)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderChromaticAberrationSettings {
    pub intensity: Real,
    pub sample_spread: Real,
}

impl Default for RenderChromaticAberrationSettings {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            sample_spread: 1.0,
        }
    }
}

impl RenderChromaticAberrationSettings {
    pub fn is_enabled(self) -> bool {
        self.intensity > 0.0
    }

    pub fn render_intensity(self) -> Real {
        self.intensity.max(0.0)
    }

    pub fn render_sample_spread(self) -> Real {
        self.sample_spread.max(0.0)
    }
}

/// 屏幕空间雾参与场景合成，必须在颜色映射和终端抗锯齿之前确定。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderFogSettings {
    pub density: Real,
    pub height_falloff: Real,
    pub color: Vec3,
}

impl Default for RenderFogSettings {
    fn default() -> Self {
        Self {
            density: 0.0,
            height_falloff: 0.0,
            color: Vec3::ONE,
        }
    }
}

impl RenderFogSettings {
    pub fn is_enabled(self) -> bool {
        self.density > 0.0
    }

    pub fn render_density(self) -> Real {
        self.density.max(0.0)
    }

    pub fn render_height_falloff(self) -> Real {
        self.height_falloff.max(0.0)
    }

    pub fn render_color(self) -> Vec3 {
        Vec3::new(
            self.color.x.max(0.0),
            self.color.y.max(0.0),
            self.color.z.max(0.0),
        )
    }
}

#[cfg(test)]
#[path = "tests/style_settings.rs"]
mod tests;
