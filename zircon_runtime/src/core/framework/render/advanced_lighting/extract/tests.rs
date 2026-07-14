use super::*;
use crate::core::framework::render::{FroxelGridQuality, OitSettings, VolumetricFogSettings};

#[test]
fn render_advanced_extract_empty_keeps_optional_sections_empty() {
    let extract = AdvancedLightingExtract::default();

    assert!(extract.is_empty());
    assert!(extract.material_features.is_empty());
    assert!(extract.volumetric.is_none());
    assert!(extract.oit.is_none());
    assert!(extract.fog_volumes.is_empty());
    assert!(extract.cookies.is_empty());
    assert!(extract.irradiance_volumes.is_empty());
    assert!(extract.planar_probes.is_empty());
    assert!(extract.subsurface_profiles.is_empty());
    assert!(extract.subsurface_material_profile_indices.is_empty());
}

#[test]
fn render_advanced_extract_tracks_material_driven_scene_copy_usage() {
    let extract = AdvancedLightingExtract {
        material_features: crate::core::framework::render::AdvancedPbrMaterialFrameUsage {
            specular_transmission: true,
            ..Default::default()
        },
        ..AdvancedLightingExtract::default()
    };

    assert!(!extract.is_empty());
    assert!(extract.requires_transmission_scene_copy());
    assert_eq!(extract.transmission_scene_copy_step_count(), 1);
    assert_eq!(extract.transmission_draw_step_count(), 1);
}

#[test]
fn render_advanced_extract_transmission_steps_zero_keep_environment_only_draw() {
    let extract = AdvancedLightingExtract {
        material_features: crate::core::framework::render::AdvancedPbrMaterialFrameUsage {
            specular_transmission: true,
            ..Default::default()
        },
        screen_space_transmission:
            crate::core::framework::render::ScreenSpaceTransmissionSettings::new(0),
        ..AdvancedLightingExtract::default()
    };

    assert!(!extract.requires_transmission_scene_copy());
    assert_eq!(extract.transmission_scene_copy_step_count(), 0);
    assert_eq!(extract.transmission_draw_step_count(), 1);
}

#[test]
fn render_advanced_extract_tracks_view_local_subsurface_profile_usage() {
    let extract = AdvancedLightingExtract {
        subsurface_material_profile_indices: vec![7],
        ..AdvancedLightingExtract::default()
    };

    assert!(!extract.is_empty());
    assert!(extract.uses_subsurface_profile(7));
    assert!(!extract.uses_subsurface_profile(3));
}

#[test]
fn render_advanced_extract_oit_settings_make_sideband_non_empty() {
    let extract = AdvancedLightingExtract {
        oit: Some(OitSettings::default()),
        ..AdvancedLightingExtract::default()
    };

    assert!(!extract.is_empty());
    assert_eq!(extract.oit, Some(OitSettings::DEFAULT));
}

#[test]
fn render_advanced_extract_volumetric_quality_controls_froxel_dimensions() {
    let extract = AdvancedLightingExtract {
        volumetric: Some(VolumetricFogSettings::default()),
        ..AdvancedLightingExtract::default()
    };

    assert!(!extract.is_empty());
    assert_eq!(
        extract.froxel_dimensions(FroxelGridQuality::Medium),
        [160, 90, 64]
    );
    assert_eq!(
        extract.froxel_dimensions(FroxelGridQuality::Low),
        [160, 90, 48]
    );
    assert_eq!(
        extract.froxel_dimensions(FroxelGridQuality::High),
        [160, 90, 96]
    );
}
