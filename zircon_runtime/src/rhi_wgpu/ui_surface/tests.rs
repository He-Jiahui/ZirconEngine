use crate::rhi::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
    UiSurfaceImageUvRect, UiSurfaceRect,
};

use super::*;

#[test]
fn wgpu_ui_surface_presenter_records_present_stats() {
    let mut presenter = WgpuUiSurfacePresenter::new_headless(32, 16);
    let draw_list = UiSurfaceDrawList::new(
        (32, 16),
        None,
        vec![UiSurfaceCommand {
            z_index: 0,
            frame: UiSurfaceRect::new(0.0, 0.0, 16.0, 8.0),
            clip: None,
            kind: UiSurfaceCommandKind::Quad {
                color: [1, 2, 3, 255],
                corner_radius: 0.0,
            },
        }],
    );

    let stats = presenter.present(&draw_list).unwrap();

    assert_eq!(stats.surface_size, (32, 16));
    assert_eq!(stats.draw_calls, 1);
    assert_eq!(stats.visible_command_count, 1);
    assert_eq!(stats.visible_draw_item_count, 1);
    assert_eq!(stats.batch_layer_count, 1);
    assert_eq!(stats.batch_dependency_count, 0);
    assert_eq!(stats.presented_frame_count, 1);
    assert_eq!(presenter.last_present_stats(), stats);
}

#[test]
fn wgpu_ui_surface_presenter_resize_tracks_draw_list_size() {
    let mut presenter = WgpuUiSurfacePresenter::new_headless(1, 1);
    let draw_list = UiSurfaceDrawList::new((64, 48), None, Vec::new());

    let stats = presenter.present(&draw_list).unwrap();

    assert_eq!(presenter.descriptor().clamped_size(), (64, 48));
    assert_eq!(stats.surface_size, (64, 48));
}

#[test]
fn wgpu_ui_surface_prefers_opaque_swapchain_alpha() {
    assert_eq!(
        choose_alpha_mode(&[
            wgpu::CompositeAlphaMode::PreMultiplied,
            wgpu::CompositeAlphaMode::Opaque,
        ]),
        wgpu::CompositeAlphaMode::Opaque
    );
    assert_eq!(
        choose_alpha_mode(&[wgpu::CompositeAlphaMode::PostMultiplied]),
        wgpu::CompositeAlphaMode::PostMultiplied
    );
}

#[test]
fn wgpu_ui_surface_uses_non_srgb_formats_for_byte_exact_editor_parity() {
    assert_eq!(UI_IMAGE_TEXTURE_FORMAT, wgpu::TextureFormat::Rgba8Unorm);
    assert_eq!(
        choose_surface_format(&[
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Rgba8Unorm,
        ]),
        Some(wgpu::TextureFormat::Bgra8Unorm)
    );
    assert_eq!(
        choose_surface_format(&[wgpu::TextureFormat::Rgba8Unorm]),
        Some(wgpu::TextureFormat::Rgba8Unorm)
    );
    assert_eq!(
        choose_surface_format(&[
            wgpu::TextureFormat::Bgra8UnormSrgb,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ]),
        None
    );
}

#[test]
fn wgpu_ui_surface_presenter_uses_damage_for_patch_stats() {
    let mut presenter = WgpuUiSurfacePresenter::new_headless(100, 100);
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        Some(UiSurfaceRect::new(50.0, 50.0, 10.0, 10.0)),
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [255, 255, 255, 255],
                    corner_radius: 0.0,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(50.0, 50.0, 5.0, 5.0),
                clip: None,
                kind: UiSurfaceCommandKind::Image {
                    payload: UiSurfaceImagePayload {
                        resource_key: "viewport".to_string(),
                        width: 2,
                        height: 2,
                        upload_bytes: 16,
                        rgba: Some(vec![255; 16]),
                        atlas_uv: None,
                    },
                },
            },
        ],
    );

    let stats = presenter.present(&draw_list).unwrap();

    assert_eq!(stats.draw_calls, 1);
    assert_eq!(stats.visible_command_count, 1);
    assert_eq!(stats.visible_draw_item_count, 1);
    assert_eq!(stats.batch_layer_count, 1);
    assert_eq!(stats.batch_dependency_count, 0);
    assert_eq!(stats.image_count, 1);
    assert_eq!(stats.image_upload_bytes, 16);
}

#[test]
fn wgpu_ui_surface_render_mode_requires_initialized_cache_for_damage_patch() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        Some(UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0)),
        Vec::new(),
    );

    assert_eq!(
        surface_render_mode(&draw_list, false),
        SurfaceRenderMode::FullRedraw
    );
    assert_eq!(
        surface_render_mode(&draw_list, true),
        SurfaceRenderMode::DamagePatch
    );
    assert_eq!(
        surface_render_mode(&UiSurfaceDrawList::new((100, 100), None, Vec::new()), true),
        SurfaceRenderMode::FullRedraw
    );
}

#[test]
fn wgpu_ui_surface_presenter_stats_report_batched_draw_calls() {
    let mut presenter = WgpuUiSurfacePresenter::new_headless(100, 100);
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [255, 0, 0, 255],
                    corner_radius: 0.0,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [0, 255, 0, 255],
                    corner_radius: 0.0,
                },
            },
        ],
    );

    let stats = presenter.present(&draw_list).unwrap();

    assert_eq!(stats.visible_command_count, 2);
    assert_eq!(stats.visible_draw_item_count, 2);
    assert_eq!(stats.draw_calls, 1);
    assert_eq!(stats.batch_layer_count, 1);
    assert_eq!(stats.batch_dependency_count, 0);
}

#[test]
fn wgpu_ui_surface_headless_stats_batch_atlas_images_by_resource_key() {
    let mut presenter = WgpuUiSurfacePresenter::new_headless(100, 100);
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        None,
        vec![
            atlas_image(
                0,
                UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0),
                UiSurfaceImageUvRect {
                    min: [0.0, 0.0],
                    max: [0.5, 0.5],
                },
            ),
            atlas_image(
                1,
                UiSurfaceRect::new(20.0, 0.0, 10.0, 10.0),
                UiSurfaceImageUvRect {
                    min: [0.5, 0.0],
                    max: [1.0, 0.5],
                },
            ),
        ],
    );

    let stats = presenter.present(&draw_list).unwrap();

    assert_eq!(stats.visible_command_count, 2);
    assert_eq!(stats.visible_draw_item_count, 2);
    assert_eq!(stats.image_count, 2);
    assert_eq!(stats.image_upload_bytes, 64);
    assert_eq!(stats.draw_calls, 1);
    assert_eq!(stats.batch_layer_count, 1);
    assert_eq!(stats.batch_dependency_count, 0);
}

#[test]
fn wgpu_ui_surface_image_cache_prune_keeps_recent_entries() {
    let prune = image_cache_keys_to_prune(
        [("oldest", 1), ("recent", 10), ("middle", 5), ("newest", 20)].into_iter(),
        2,
    );

    assert_eq!(prune, vec!["oldest".to_string(), "middle".to_string()]);
}

#[test]
fn wgpu_ui_surface_image_cache_prune_is_stable_for_ties() {
    let prune = image_cache_keys_to_prune([("b", 1), ("c", 1), ("a", 1), ("d", 2)].into_iter(), 2);

    assert_eq!(prune, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn wgpu_ui_surface_image_cache_prune_is_noop_under_budget() {
    let prune = image_cache_keys_to_prune([("one", 1), ("two", 2)].into_iter(), 2);

    assert!(prune.is_empty());
}

fn atlas_image(
    z_index: i32,
    frame: UiSurfaceRect,
    atlas_uv: UiSurfaceImageUvRect,
) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index,
        frame,
        clip: None,
        kind: UiSurfaceCommandKind::Image {
            payload: UiSurfaceImagePayload {
                resource_key: "atlas://editor/icons".to_string(),
                width: 4,
                height: 4,
                upload_bytes: 64,
                rgba: Some(vec![255; 64]),
                atlas_uv: Some(atlas_uv),
            },
        },
    }
}
