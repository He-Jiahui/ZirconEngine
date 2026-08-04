use crate::core::math::{Real, Vec3};

const MIN_VIGNETTE_SMOOTHNESS: Real = 0.001;
const MIN_VIGNETTE_ROUNDNESS: Real = 0.001;
const MIN_DITHER_SCALE: Real = 0.001;

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
mod tests {
    use super::{
        RenderChromaticAberrationSettings, RenderDitherSettings, RenderFilmGrainSettings,
        RenderFogSettings, RenderVignetteSettings,
    };
    use crate::core::math::Vec3;

    #[test]
    fn stylistic_effect_settings_use_explicit_enable_predicates() {
        assert!(!RenderVignetteSettings::default().is_enabled());
        assert!(RenderVignetteSettings {
            intensity: 0.2,
            ..Default::default()
        }
        .is_enabled());

        assert!(!RenderFilmGrainSettings::default().is_enabled());
        assert!(RenderFilmGrainSettings {
            intensity: 0.1,
            ..Default::default()
        }
        .is_enabled());

        assert!(!RenderDitherSettings::default().is_enabled());
        assert!(RenderDitherSettings {
            intensity: 0.05,
            ..Default::default()
        }
        .is_enabled());

        assert!(!RenderChromaticAberrationSettings::default().is_enabled());
        assert!(RenderChromaticAberrationSettings {
            intensity: 0.08,
            ..Default::default()
        }
        .is_enabled());

        assert!(!RenderFogSettings::default().is_enabled());
        assert!(RenderFogSettings {
            density: 0.03,
            ..Default::default()
        }
        .is_enabled());
    }

    #[test]
    fn stylistic_effect_settings_sanitize_renderer_upload_values() {
        let vignette = RenderVignetteSettings {
            intensity: -0.25,
            smoothness: -1.0,
            roundness: 0.0,
        };
        assert_eq!(vignette.render_intensity(), 0.0);
        assert_eq!(vignette.render_smoothness(), 0.001);
        assert_eq!(vignette.render_roundness(), 0.001);

        let grain = RenderFilmGrainSettings {
            intensity: -0.1,
            response: -0.5,
        };
        assert_eq!(grain.render_intensity(), 0.0);
        assert_eq!(grain.render_response(), 0.0);

        let dither = RenderDitherSettings {
            intensity: -0.1,
            scale: 0.0,
        };
        assert_eq!(dither.render_intensity(), 0.0);
        assert_eq!(dither.render_scale(), 0.001);

        let chromatic_aberration = RenderChromaticAberrationSettings {
            intensity: -0.1,
            sample_spread: -2.0,
        };
        assert_eq!(chromatic_aberration.render_intensity(), 0.0);
        assert_eq!(chromatic_aberration.render_sample_spread(), 0.0);

        let fog = RenderFogSettings {
            density: -0.2,
            height_falloff: -3.0,
            color: Vec3::new(-1.0, 0.25, -0.5),
        };
        assert_eq!(fog.render_density(), 0.0);
        assert_eq!(fog.render_height_falloff(), 0.0);
        assert_eq!(fog.render_color(), Vec3::new(0.0, 0.25, 0.0));
    }
}
