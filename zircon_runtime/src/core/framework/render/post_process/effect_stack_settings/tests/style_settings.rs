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
