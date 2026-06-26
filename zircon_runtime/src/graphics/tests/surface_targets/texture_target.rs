use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AssetUri, TextureAsset, TextureAssetDescriptor};
use crate::core::framework::render::{
    RenderCameraTarget, RenderCameraTargetGraphImportStatus, RenderCameraTargetKind,
    RenderCameraTargetWritebackStatus, RenderCaptureSource, RenderFramework,
    RenderViewportDescriptor,
};
use crate::core::math::{UVec2, Vec4};
use crate::core::resource::{
    ResourceHandle, ResourceId, ResourceKind, ResourceRecord, TextureMarker,
};
use crate::graphics::WgpuRenderFramework;

use super::{
    dominant_green_pixels, dominant_red_pixels, empty_extract_with_cameras,
    empty_extract_with_target, render_target_texture_asset, render_target_texture_descriptor,
    srgb_render_target_texture_asset, texture_base_camera, texture_base_camera_with_entity,
    texture_overlay_camera, unsupported_camera_texture_surface_present,
    unsupported_camera_texture_target, unsupported_camera_texture_target_format,
    unsupported_camera_texture_target_usage, CameraDescriptorTestExt as _,
};

#[test]
fn graphics_camera_target_texture_missing_asset_reports_unsupported_without_primary_fallback_capture(
) {
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "tests/camera-target/texture",
    ));

    let error = framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(texture)),
        )
        .unwrap_err();

    assert_eq!(error, unsupported_camera_texture_target());
    assert_eq!(framework.capture_frame(viewport).unwrap(), None);
    assert_eq!(framework.query_stats().unwrap().submitted_frames, 0);
}

#[test]
fn graphics_camera_target_texture_requires_render_target_usage() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = AssetUri::parse("res://tests/camera-target/sampled.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            TextureAsset::new_rgba8(texture_uri, 72, 40, vec![0; 72 * 40 * 4]),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    let error = framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap_err();

    assert_eq!(error, unsupported_camera_texture_target_usage());
    assert_eq!(framework.capture_frame(viewport).unwrap(), None);
    assert_eq!(framework.query_stats().unwrap().submitted_frames, 0);
}

#[test]
fn graphics_camera_target_texture_requires_renderable_render_target_format() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri =
        AssetUri::parse("res://tests/camera-target/compressed-target.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            render_target_texture_asset(texture_uri, 72, 40).with_descriptor(
                TextureAssetDescriptor {
                    format: "dds/dxt1".to_string(),
                    ..render_target_texture_descriptor()
                },
            ),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    let error = framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap_err();

    assert_eq!(error, unsupported_camera_texture_target_format());
    assert_eq!(framework.capture_frame(viewport).unwrap(), None);
    assert_eq!(framework.query_stats().unwrap().submitted_frames, 0);
}

#[test]
fn graphics_camera_target_texture_render_target_metadata_controls_offscreen_capture_size() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = AssetUri::parse("res://tests/camera-target/render-target.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            render_target_texture_asset(texture_uri, 72, 40),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();

    assert_eq!(frame.width, 72);
    assert_eq!(frame.height, 40);
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureWritebackConversion
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::Converted
    );
    let target_resolution = framework
        .query_stats()
        .unwrap()
        .last_camera_target_resolution;
    assert_eq!(
        target_resolution.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(target_resolution.primary_target_size, UVec2::new(64, 48));
    assert_eq!(target_resolution.resolved_target_size, UVec2::new(72, 40));
    assert_eq!(target_resolution.effective_view_size, UVec2::new(72, 40));
    assert_eq!(target_resolution.effective_render_size, UVec2::new(72, 40));
    let stats = framework.query_stats().unwrap();
    assert_eq!(stats.submitted_frames, 1);
    assert_eq!(
        stats.last_capture_report.source,
        RenderCaptureSource::TextureWritebackConversion
    );
    assert_eq!(stats.last_capture_report.output_size, UVec2::new(72, 40));
    assert_eq!(
        stats.last_camera_target_writeback.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        stats.last_camera_target_writeback.status,
        RenderCameraTargetWritebackStatus::Converted
    );
    assert_eq!(
        stats.last_camera_target_writeback.target_size,
        UVec2::new(72, 40)
    );
    assert_eq!(stats.last_camera_target_writeback.copied_count, 0);
    assert_eq!(stats.last_camera_target_writeback.converted_count, 1);
    assert!(!stats.last_camera_target_writeback.debug_marker_emitted);
    assert_eq!(
        stats.last_camera_target_graph_import.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(
        stats.last_camera_target_graph_import.target_size,
        UVec2::new(72, 40)
    );
    assert_eq!(stats.last_camera_target_graph_import.direct_import_count, 0);
    assert_eq!(
        stats
            .last_camera_target_graph_import
            .conversion_writeback_count,
        1
    );
}

#[test]
fn graphics_camera_target_texture_srgb_target_imports_direct_graph_final_target() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri =
        AssetUri::parse("res://tests/camera-target/srgb-render-target.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            srgb_render_target_texture_asset(texture_uri, 72, 40),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(frame.width, 72);
    assert_eq!(frame.height, 40);
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(stats.submitted_frames, 1);
    assert_eq!(
        stats.last_capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
    assert_eq!(stats.last_capture_report.output_size, UVec2::new(72, 40));
    assert_eq!(
        stats.last_camera_target_writeback.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        stats.last_camera_target_writeback.status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(
        stats.last_camera_target_writeback.target_size,
        UVec2::new(72, 40)
    );
    assert_eq!(stats.last_camera_target_writeback.copied_count, 0);
    assert_eq!(stats.last_camera_target_writeback.converted_count, 0);
    assert!(!stats.last_camera_target_writeback.debug_marker_emitted);
    assert_eq!(
        stats.last_camera_target_graph_import.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        stats.last_camera_target_graph_import.target_size,
        UVec2::new(72, 40)
    );
    assert_eq!(stats.last_camera_target_graph_import.direct_import_count, 1);
    assert_eq!(
        stats
            .last_camera_target_graph_import
            .conversion_writeback_count,
        0
    );
}

#[test]
fn graphics_camera_target_texture_overlay_stack_preserves_base_composite() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri =
        AssetUri::parse("res://tests/camera-target/overlay-stack-composite.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            srgb_render_target_texture_asset(texture_uri, 64, 48),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_cameras(vec![
                texture_base_camera(texture_id, Vec4::new(1.0, 0.0, 0.0, 1.0)).with_stack([2]),
                texture_overlay_camera(texture_id),
            ]),
        )
        .unwrap();
    let frame = framework.capture_frame(viewport).unwrap().unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        stats.last_camera_target_writeback.status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert!(
        dominant_red_pixels(&frame.rgba) > ((frame.width * frame.height) as usize * 9 / 10),
        "base camera clear must survive overlay stack load into the texture final target"
    );
}

#[test]
fn graphics_camera_target_texture_base_stacks_write_independent_texture_targets() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let red_texture_uri =
        AssetUri::parse("res://tests/camera-target/multi-target-red.texture").unwrap();
    let green_texture_uri =
        AssetUri::parse("res://tests/camera-target/multi-target-green.texture").unwrap();
    let red_texture_id = ResourceId::from_locator(&red_texture_uri);
    let green_texture_id = ResourceId::from_locator(&green_texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                red_texture_id,
                ResourceKind::Texture,
                red_texture_uri.clone(),
            ),
            srgb_render_target_texture_asset(red_texture_uri, 64, 48),
        )
        .expect("red texture insert");
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(
                green_texture_id,
                ResourceKind::Texture,
                green_texture_uri.clone(),
            ),
            srgb_render_target_texture_asset(green_texture_uri, 64, 48),
        )
        .expect("green texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            empty_extract_with_cameras(vec![
                texture_base_camera_with_entity(
                    1,
                    0,
                    red_texture_id,
                    Vec4::new(1.0, 0.0, 0.0, 1.0),
                ),
                texture_base_camera_with_entity(
                    2,
                    1,
                    green_texture_id,
                    Vec4::new(0.0, 1.0, 0.0, 1.0),
                ),
            ]),
        )
        .unwrap();

    let (red_size, red_rgba) = framework
        .read_output_target_texture_rgba_for_tests(red_texture_id)
        .unwrap()
        .expect("red target should stay prepared after submit");
    let (green_size, green_rgba) = framework
        .read_output_target_texture_rgba_for_tests(green_texture_id)
        .unwrap()
        .expect("green target should stay prepared after submit");
    let frame = framework.capture_frame(viewport).unwrap().unwrap();

    assert_eq!(red_size, UVec2::new(64, 48));
    assert_eq!(green_size, UVec2::new(64, 48));
    let target_pixels = (64 * 48) as usize;
    let red_target_red = dominant_red_pixels(&red_rgba);
    let red_target_green = dominant_green_pixels(&red_rgba);
    let green_target_red = dominant_red_pixels(&green_rgba);
    let green_target_green = dominant_green_pixels(&green_rgba);

    assert!(
        red_target_red > target_pixels * 9 / 10,
        "first texture Base stack should write red pixels into its own final target; red={red_target_red}, green={red_target_green}, total={target_pixels}"
    );
    assert!(
        red_target_green < target_pixels / 20,
        "second texture Base stack should not overwrite the first texture target; green={red_target_green}, total={target_pixels}"
    );
    assert!(
        green_target_green > target_pixels * 9 / 10,
        "second texture Base stack should write green pixels into its own final target; green={green_target_green}, red={green_target_red}, total={target_pixels}"
    );
    assert!(
        green_target_red < target_pixels / 20,
        "first texture Base stack should not leak into the second texture target; red={green_target_red}, total={target_pixels}"
    );
    assert!(
        dominant_green_pixels(&frame.rgba) > target_pixels * 9 / 10,
        "viewport capture remains owned by the last texture Base stack"
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
}

#[test]
fn graphics_camera_target_texture_present_reports_unsupported_surface_fallback() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let texture_uri = AssetUri::parse("res://tests/camera-target/present-target.texture").unwrap();
    let texture_id = ResourceId::from_locator(&texture_uri);
    asset_manager
        .assets::<TextureAsset>()
        .insert(
            ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
            render_target_texture_asset(texture_uri, 72, 40),
        )
        .expect("texture insert");
    let framework = WgpuRenderFramework::new(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(64, 48)))
        .unwrap();

    let error = framework
        .present_frame_extract(
            viewport,
            empty_extract_with_target(RenderCameraTarget::Texture(
                ResourceHandle::<TextureMarker>::new(texture_id),
            )),
        )
        .unwrap_err();

    assert_eq!(error, unsupported_camera_texture_surface_present());
}
