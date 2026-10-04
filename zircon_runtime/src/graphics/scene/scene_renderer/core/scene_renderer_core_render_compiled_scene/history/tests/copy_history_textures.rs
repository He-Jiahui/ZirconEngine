use crate::core::framework::render::{
    CameraRenderDescriptor, RenderViewportRect, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;
use crate::graphics::types::ViewportRenderRegion;
use crate::rhi::{TextureDesc, TextureFormat, TextureUsage};

use super::{
    history_region_copy_extent, history_region_copy_origin,
    owned_global_illumination_history_source_is_copyable,
    owned_global_illumination_temporal_metadata_is_copyable,
};

#[test]
fn history_copy_encoding_returns_intent_without_committing_persistent_state() {
    let source = include_str!("../copy_history_textures.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("SceneHistoryWriteIntent"));
    assert!(production.contains("write_intent.record("));
    assert!(
        production.contains("graph_history_writes.was_written(SceneHistoryDomain::TaaSceneColor)")
    );
    assert!(production.contains("graph_history_writes.was_written(SceneHistoryDomain::Exposure)"));
    assert!(production
        .contains("graph_history_writes.was_written(SceneHistoryDomain::ScreenSpaceReflection)"));
    assert!(
        production.contains("graph_history_writes.was_written(SceneHistoryDomain::HzbFurthest)")
    );
    assert!(production
        .contains("graph_history_writes.was_written(SceneHistoryDomain::VolumetricScattering)"));
    assert!(production.contains("was_written(SceneHistoryDomain::HybridGlobalIllumination)"));
    assert!(!production.contains("SceneHistoryDomain::AmbientOcclusion"));
    assert!(!production.contains("target.ambient_occlusion.as_image_copy()"));
    assert!(!production.contains("history.ambient_occlusion.as_image_copy()"));
    assert!(!production.contains("scene_color_copied = true"));
    assert!(!production.contains("exposure_copied = true"));
    assert!(production.contains("graph_owned_texture_for_access"));
    assert!(!production.contains("owned_texture("));
    assert!(!production.contains("PostProcessGraphResourceNames"));
    assert!(!production.contains("flip_taa_scene_color_history"));
    assert!(!production.contains("flip_exposure_history"));
    assert!(!production.contains("set_global_illumination_history_valid"));
    assert!(!production.contains("set_volumetric_history_valid"));
}

#[test]
fn global_illumination_history_source_requires_single_sample_rgba16_float() {
    let desc = TextureDesc::new(
        "hybrid-gi-lighting",
        64,
        64,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    );

    assert!(owned_global_illumination_history_source_is_copyable(&desc));
}

#[test]
fn global_illumination_history_source_rejects_msaa_depth_or_sdr_graph_output() {
    let msaa = TextureDesc::new(
        "hybrid-gi-lighting",
        64,
        64,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    )
    .with_sample_count(4);
    let depth = TextureDesc::new(
        "hybrid-gi-lighting",
        64,
        64,
        TextureFormat::Depth32Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    );
    let sdr = TextureDesc::new(
        "hybrid-gi-lighting",
        64,
        64,
        TextureFormat::Rgba8UnormSrgb,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    );

    assert!(!owned_global_illumination_history_source_is_copyable(&msaa));
    assert!(!owned_global_illumination_history_source_is_copyable(
        &depth
    ));
    assert!(!owned_global_illumination_history_source_is_copyable(&sdr));
}

#[test]
fn global_illumination_temporal_metadata_requires_single_sample_rgba16_float() {
    let valid = TextureDesc::new(
        "hybrid-gi-temporal-metadata",
        64,
        64,
        TextureFormat::Rgba16Float,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    );
    let wrong_format = TextureDesc::new(
        "hybrid-gi-temporal-metadata",
        64,
        64,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    );
    let msaa = valid.clone().with_sample_count(4);

    assert!(owned_global_illumination_temporal_metadata_is_copyable(
        &valid
    ));
    assert!(!owned_global_illumination_temporal_metadata_is_copyable(
        &wrong_format
    ));
    assert!(!owned_global_illumination_temporal_metadata_is_copyable(
        &msaa
    ));
}

#[test]
fn history_region_copy_targets_selected_camera_region() {
    let region = selected_camera_region(
        UVec2::new(1280, 720),
        UVec2::new(640, 0),
        UVec2::new(640, 720),
    );

    let extent = history_region_copy_extent(UVec2::new(640, 720), UVec2::new(1280, 720), region)
        .expect("selected camera copy should fit target");
    let origin = history_region_copy_origin(region);

    assert_eq!(extent.width, 640);
    assert_eq!(extent.height, 720);
    assert_eq!(origin.x, 640);
    assert_eq!(origin.y, 0);
}

#[test]
fn history_region_copy_clamps_dynamic_resolution_to_viewport_region() {
    let region = selected_camera_region(
        UVec2::new(1280, 720),
        UVec2::new(960, 540),
        UVec2::new(512, 512),
    );

    let extent = history_region_copy_extent(UVec2::new(512, 512), UVec2::new(1280, 720), region)
        .expect("partially clipped selected camera copy should retain visible area");

    assert_eq!(extent.width, 320);
    assert_eq!(extent.height, 180);
}

#[test]
fn history_region_copy_uses_local_extent_and_physical_destination() {
    let viewport_region = selected_camera_region(
        UVec2::new(1280, 720),
        UVec2::new(960, 540),
        UVec2::new(512, 512),
    );
    let render_region = viewport_region.with_local_size(UVec2::new(160, 90));

    let extent =
        history_region_copy_extent(UVec2::new(256, 256), UVec2::new(1280, 720), render_region)
            .expect("selected camera copy should retain local internal area");
    let origin = history_region_copy_origin(render_region);

    assert_eq!(extent.width, 160);
    assert_eq!(extent.height, 90);
    assert_eq!(origin.x, 960);
    assert_eq!(origin.y, 540);
}

fn selected_camera_region(
    target_size: UVec2,
    position: UVec2,
    size: UVec2,
) -> ViewportRenderRegion {
    let mut camera =
        CameraRenderDescriptor::from_camera_payload(None, ViewportCameraSnapshot::default());
    camera.viewport_rect = Some(RenderViewportRect::new(position, size));
    ViewportRenderRegion::from_camera(Some(&camera), target_size)
}
