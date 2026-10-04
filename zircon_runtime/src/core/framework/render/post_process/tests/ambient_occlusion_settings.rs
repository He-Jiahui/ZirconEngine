use super::{AoQualityTier, AoSourceSettings, AoSourceSettingsKey, AO_SOURCE_SETTINGS_VERSION};

#[test]
fn ao_source_settings_key_preserves_physical_units_and_discrete_modes() {
    let settings = AoSourceSettings {
        intensity: 0.75,
        radius_meters: 2.5,
        thickness_meters: 0.25,
        depth_bias_meters: 0.03,
        falloff_start_meters: 1.25,
        quality: AoQualityTier::Ultra,
        half_resolution: false,
        temporal: true,
    };

    let key = AoSourceSettingsKey::from(settings);

    assert_eq!(key.version(), AO_SOURCE_SETTINGS_VERSION);
    assert_eq!(key.intensity(), settings.intensity);
    assert_eq!(key.radius_meters(), settings.radius_meters);
    assert_eq!(key.thickness_meters(), settings.thickness_meters);
    assert_eq!(key.depth_bias_meters(), settings.depth_bias_meters);
    assert_eq!(key.falloff_start_meters(), settings.falloff_start_meters);
    assert_eq!(key.quality(), settings.quality);
    assert!(!key.half_resolution());
    assert!(key.temporal());
    assert_eq!(AoSourceSettings::from(key), settings);
}

#[test]
fn ao_source_settings_default_does_not_request_unqualified_history() {
    assert!(!AoSourceSettings::default().temporal);
}
