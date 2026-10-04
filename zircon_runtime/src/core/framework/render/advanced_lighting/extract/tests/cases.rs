use super::*;
use crate::core::framework::render::{
    FogVolumeData, FroxelGridQuality, OitSettings, RenderLayerSet, VolumetricFogSettings,
};
use crate::core::math::Vec3;

#[test]
fn render_advanced_extract_empty_keeps_optional_sections_empty() {
    let extract = AdvancedLightingExtract::default();

    assert!(extract.is_empty());
    assert!(extract.material_features.is_empty());
    assert!(extract.volumetric.is_none());
    assert!(extract.oit.is_none());
    assert!(extract.fog_volumes.is_empty());
    assert!(extract.volumetric_light_ids.is_empty());
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

#[test]
fn render_advanced_extract_filters_fog_volumes_for_camera_layers() {
    let extract = AdvancedLightingExtract {
        fog_volumes: vec![
            FogVolumeData {
                volume_id: 2,
                bounds_min: Vec3::ZERO,
                bounds_max: Vec3::ONE,
                density: 0.2,
                albedo: Vec3::ONE,
                layer_mask: RenderLayerSet::layer(2),
            },
            FogVolumeData {
                volume_id: 1,
                bounds_min: -Vec3::ONE,
                bounds_max: Vec3::ZERO,
                density: 0.1,
                albedo: Vec3::splat(0.5),
                layer_mask: RenderLayerSet::layer(1),
            },
        ],
        ..AdvancedLightingExtract::default()
    };

    let render_layers = RenderLayerSet::layer(1);
    let mut visible = extract.fog_volumes_for_layers(&render_layers);

    assert!(std::ptr::eq(
        visible.next().unwrap(),
        &extract.fog_volumes[1]
    ));
    assert!(visible.next().is_none());
}
