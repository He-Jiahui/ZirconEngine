use crate::core::math::UVec2;

use super::super::RenderUpscalerKind;
use super::{RenderLayerSet, ViewportCameraSnapshot};

#[test]
fn default_camera_preserves_the_view_family_full_resolution_contract() {
    let pipeline = ViewportCameraSnapshot::default()
        .render_view_family_pipeline(UVec2::new(1920, 1080), RenderUpscalerKind::Spatial);

    assert_eq!(
        pipeline.resolution().primary_extent(),
        UVec2::new(1920, 1080)
    );
    assert_eq!(
        pipeline.resolution().secondary_extent(),
        UVec2::new(1920, 1080)
    );
}

#[test]
fn camera_dynamic_resolution_adapts_into_the_view_family_resolution_policy() {
    let mut camera = ViewportCameraSnapshot::default();
    camera.dynamic_resolution = super::RenderDynamicResolutionSettings::fixed_scale(2.0 / 3.0);

    let pipeline =
        camera.render_view_family_pipeline(UVec2::new(1920, 1080), RenderUpscalerKind::Temporal);

    assert_eq!(
        pipeline.resolution().display_extent(),
        UVec2::new(1920, 1080)
    );
    assert_eq!(
        pipeline.resolution().primary_extent(),
        UVec2::new(1280, 720)
    );
    assert_eq!(
        pipeline.resolution().temporal_history_extent(),
        Some(UVec2::new(1920, 1080))
    );
}

#[test]
fn render_layer_schema_v1_uses_single_block_fast_paths() {
    let source = include_str!("layer_set.rs");
    assert!(!source.contains(concat!("for layer in 0..", "u32::BITS")));
    assert!(!source.contains(concat!(
        "self.intersects(&Self::from_scene_",
        "schema_v1_mask(mask))"
    )));

    let layers = RenderLayerSet::from_scene_schema_v1_mask(0b1010);
    assert_eq!(layers.iter().collect::<Vec<_>>(), vec![1, 3]);
    assert_eq!(layers.to_scene_schema_v1_mask_lossy(), 0b1010);
    assert!(layers.intersects_scene_schema_v1_mask(0b1000));
    assert!(!layers.intersects_scene_schema_v1_mask(0b0100));
    assert!(RenderLayerSet::from_scene_schema_v1_mask(0).is_empty());

    let wide = RenderLayerSet::layer(70).with(3);
    assert!(wide.intersects_scene_schema_v1_mask(0b1000));
    assert!(!wide.intersects_scene_schema_v1_mask(0b0100));
}
