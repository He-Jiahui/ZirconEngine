use std::collections::{BTreeMap, HashMap};

use zr_rhi::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
    UiSurfaceImageResource, UiSurfaceImageUvRect, UiSurfaceRect,
};

use super::image_cache::UI_IMAGE_TEXTURE_FORMAT;
use super::*;

#[path = "tests/native_submission.rs"]
mod native_submission;

#[test]
fn retryable_surface_acquisition_does_not_advance_the_presented_frame_count() {
    for acquisition in [
        wgpu::CurrentSurfaceTexture::Outdated,
        wgpu::CurrentSurfaceTexture::Lost,
        wgpu::CurrentSurfaceTexture::Timeout,
        wgpu::CurrentSurfaceTexture::Occluded,
    ] {
        assert_eq!(
            retryable_surface_outcome(&acquisition),
            Some(UiSurfacePresentOutcome::RetryableNoSubmit)
        );
    }
    assert_eq!(
        advance_presented_frame_count(17, UiSurfacePresentOutcome::RetryableNoSubmit),
        17
    );
    assert_eq!(
        advance_presented_frame_count(17, UiSurfacePresentOutcome::Submitted),
        18
    );
}

#[test]
fn retryable_surface_presentation_preserves_surface_stats_contract() {
    let presentation = retryable_surface_presentation((640, 480));

    assert_eq!(
        presentation.outcome,
        UiSurfacePresentOutcome::RetryableNoSubmit
    );
    assert_eq!(
        presentation.draw_list_stats.outcome,
        UiSurfacePresentOutcome::RetryableNoSubmit
    );
    assert_eq!(presentation.draw_list_stats.surface_size, (640, 480));
    assert_eq!(presentation.draw_list_stats.draw_calls, 0);
    assert_eq!(presentation.draw_list_stats.visible_command_count, 0);
    assert!(presentation.image_resource_stats.is_none());
    assert!(presentation.recorded_stats.is_none());
    assert!(!presentation.gpu_timestamp_supported);
    assert_eq!(presentation.gpu_time_us, None);
    assert_eq!(presentation.gpu_profile_latency_frames, 0);
}

#[test]
fn wgpu_ui_surface_moves_staged_image_pixels_into_the_native_cache() {
    let rgba: std::sync::Arc<[u8]> = vec![17; 16].into();
    let rgba_ptr = rgba.as_ptr();
    let mut staged = Some(UiSurfaceImageResource {
        generation: 4,
        width: 2,
        height: 2,
        upload_bytes: 16,
        rgba,
    });
    let draw_list = UiSurfaceDrawList::new((2, 2), None, Vec::new());

    let moved = take_image_source_pixels(&mut staged, &draw_list, "image://owned", 4, None, 16)
        .expect("staged source should move into the cache");

    assert!(staged.is_none());
    assert_eq!(moved.as_ptr(), rgba_ptr);
    assert_eq!(moved.as_ref(), &[17; 16]);
}

#[test]
fn wgpu_ui_surface_rejects_short_staged_image_pixels() {
    let mut staged = Some(UiSurfaceImageResource {
        generation: 4,
        width: 2,
        height: 2,
        upload_bytes: 16,
        rgba: vec![17; 15].into(),
    });
    let draw_list = UiSurfaceDrawList::new((2, 2), None, Vec::new());

    assert_eq!(
        take_image_source_pixels(&mut staged, &draw_list, "image://owned", 4, None, 16),
        None
    );
    assert!(staged.is_none());
}

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
    assert_eq!(stats.compiled_draw_calls, 1);
    assert_eq!(stats.render_pass_count, 0);
    assert_eq!(stats.retained_cache_copy_bytes, 0);
    assert!(!stats.gpu_timestamp_supported);
    assert_eq!(stats.gpu_time_us, None);
    assert_eq!(stats.gpu_profile_latency_frames, 0);
    assert_eq!(stats.visible_command_count, 1);
    assert_eq!(stats.visible_draw_item_count, 1);
    assert_eq!(stats.compiled_visible_draw_item_count, 1);
    assert_eq!(stats.batch_layer_count, 1);
    assert_eq!(stats.compiled_batch_layer_count, 1);
    assert_eq!(stats.batch_dependency_count, 0);
    assert_eq!(stats.compiled_batch_dependency_count, 0);
    assert_eq!(stats.batch_plan_build_count, 1);
    assert_eq!(stats.batch_plan_cache_hit_count, 0);
    assert_eq!(stats.batch_merge_count, 0);
    assert_eq!(stats.compiled_batch_merge_count, 0);
    assert_eq!(stats.solid_vertex_count, 6);
    assert_eq!(stats.compiled_solid_vertex_count, 6);
    assert_eq!(stats.solid_instance_count, 1);
    assert_eq!(stats.compiled_solid_instance_count, 1);
    assert_eq!(stats.image_vertex_count, 0);
    assert_eq!(stats.compiled_image_vertex_count, 0);
    assert_eq!(stats.image_cache_key_allocation_count, 0);
    assert_eq!(stats.image_cache_prune_visit_count, 0);
    assert_eq!(stats.image_prepare_command_visit_count, 0);
    assert_eq!(stats.image_prepare_cache_hit_count, 0);
    assert_eq!(stats.image_upload_write_count, 0);
    assert_eq!(stats.image_shared_resolve_count, 0);
    assert_eq!(stats.image_shared_upload_write_count, 0);
    assert_eq!(stats.image_shared_upload_bytes, 0);
    assert_eq!(stats.image_shared_resident_bytes, 0);
    assert_eq!(stats.presented_frame_count, 1);
    let stats_copy = stats;
    assert_eq!(stats, stats_copy);
    assert_eq!(presenter.last_present_stats(), stats);
}

#[test]
fn wgpu_ui_surface_headless_reuses_a_versioned_batch_plan() {
    let mut presenter = WgpuUiSurfacePresenter::new_headless(32, 16);
    let draw_list = UiSurfaceDrawList::with_generation(
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
        17,
    );

    let first = presenter.present(&draw_list).unwrap();
    let second = presenter.present(&draw_list).unwrap();

    assert_eq!(first.batch_plan_build_count, 1);
    assert_eq!(first.batch_plan_cache_hit_count, 0);
    assert_eq!(first.command_visibility_scan_count, 1);
    assert_eq!(first.command_stats_cache_hit_count, 0);
    assert_eq!(second.batch_plan_build_count, 0);
    assert_eq!(second.batch_plan_cache_hit_count, 1);
    assert_eq!(second.command_visibility_scan_count, 0);
    assert_eq!(second.command_stats_cache_hit_count, 1);
    assert_eq!(second.presented_frame_count, 2);
}

#[test]
fn wgpu_ui_surface_headless_projects_overlap_candidate_count() {
    let mut presenter = WgpuUiSurfacePresenter::new_headless(32, 16);
    let draw_list = UiSurfaceDrawList::new(
        (32, 16),
        None,
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(0.0, 0.0, 16.0, 8.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [1, 2, 3, 255],
                    corner_radius: 0.0,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(8.0, 0.0, 16.0, 8.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [3, 2, 1, 255],
                    corner_radius: 0.0,
                },
            },
        ],
    );

    let stats = presenter.present(&draw_list).unwrap();

    assert_eq!(stats.overlap_candidate_count, 1);
    assert_eq!(stats.batch_dependency_count, 1);
    assert_eq!(stats.compiled_batch_dependency_count, 1);
}

#[test]
fn wgpu_ui_surface_headless_reuses_a_versioned_damage_projection() {
    let mut presenter = WgpuUiSurfacePresenter::new_headless(100, 100);
    let draw_list = UiSurfaceDrawList::with_generation(
        (100, 100),
        Some(UiSurfaceRect::new(0.0, 0.0, 10.0, 10.0)),
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
                frame: UiSurfaceRect::new(60.0, 60.0, 10.0, 10.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [0, 255, 0, 255],
                    corner_radius: 0.0,
                },
            },
        ],
        23,
    );

    let first = presenter.present(&draw_list).unwrap();
    let second = presenter.present(&draw_list).unwrap();

    assert_eq!(first.batch_plan_build_count, 1);
    assert_eq!(first.command_visibility_scan_count, 1);
    assert_eq!(first.command_stats_cache_hit_count, 0);
    assert_eq!(first.visible_command_count, 1);
    assert_eq!(second.batch_plan_build_count, 0);
    assert_eq!(second.batch_plan_cache_hit_count, 1);
    assert_eq!(second.command_visibility_scan_count, 1);
    assert_eq!(second.command_stats_cache_hit_count, 0);
    assert_eq!(second.visible_command_count, 1);
    assert_eq!(second.visible_draw_item_count, 2);
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
fn target_only_resize_uses_retained_copy_only_after_the_projection_is_ready() {
    let damage = UiSurfaceRect::new(4.0, 5.0, 6.0, 7.0);
    let mut draw_list =
        UiSurfaceDrawList::with_generation((320, 200), Some(damage), Vec::new(), 41);
    draw_list.retarget_surface_size_preserving_projection((480, 280));

    assert_eq!(
        surface_render_mode(&draw_list, false),
        SurfaceRenderMode::FullRedraw
    );
    assert_eq!(
        surface_render_mode(&draw_list, true),
        SurfaceRenderMode::RetainedProjectionCopy
    );
    assert_eq!(
        render_damage(&draw_list, SurfaceRenderMode::FullRedraw),
        None
    );
    assert_eq!(
        render_damage(&draw_list, SurfaceRenderMode::RetainedProjectionCopy),
        None
    );

    let mut cache = CompiledUiBatchPlanCache::default();
    let first = cache.resolve(&draw_list, true);
    assert_eq!(first.draw_list_stats.unwrap().surface_size, (480, 280));
    draw_list.retarget_surface_size_preserving_projection((520, 300));
    let reused = cache.resolve(&draw_list, true);
    assert_eq!(reused.batch_plan_cache_hit_count, 1);
    assert_eq!(reused.draw_list_stats.unwrap().surface_size, (520, 300));
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
fn wgpu_ui_surface_requests_gpu_timestamps_only_when_enabled_and_fully_supported() {
    let disabled = requested_device_features(GPU_TIMESTAMP_REQUIRED_FEATURES, false);
    let partial = requested_device_features(wgpu::Features::TIMESTAMP_QUERY, true);
    let full = requested_device_features(GPU_TIMESTAMP_REQUIRED_FEATURES, true);

    assert!(!disabled.intersects(GPU_TIMESTAMP_REQUIRED_FEATURES));
    assert!(!partial.intersects(GPU_TIMESTAMP_REQUIRED_FEATURES));
    assert!(full.contains(GPU_TIMESTAMP_REQUIRED_FEATURES));
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
fn wgpu_ui_surface_uses_raw_copy_only_when_the_surface_advertises_copy_destination() {
    assert_eq!(
        choose_surface_usage(wgpu::TextureUsages::RENDER_ATTACHMENT),
        wgpu::TextureUsages::RENDER_ATTACHMENT
    );
    assert_eq!(
        choose_surface_usage(
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST
        ),
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST
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
                        resource_generation: 0,
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
fn surfaces_without_copy_destination_use_direct_full_redraws() {
    let damaged = UiSurfaceDrawList::new(
        (100, 100),
        Some(UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0)),
        Vec::new(),
    );

    assert!(retained_cache_copy_supported(
        wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST
    ));
    assert_eq!(
        surface_render_mode(&damaged, true),
        SurfaceRenderMode::DamagePatch
    );
    assert!(!retained_cache_copy_supported(
        wgpu::TextureUsages::RENDER_ATTACHMENT
    ));
    assert_eq!(
        surface_render_mode(&damaged, false),
        SurfaceRenderMode::FullRedraw
    );
}

#[test]
fn surfaces_without_copy_destination_skip_retained_cache_allocation() {
    let source = include_str!("../ui_surface.rs");
    let compact = source.split_whitespace().collect::<String>();

    assert!(
        compact.contains("letretained_cache=retained_cache_copy_supported(config.usage).then(||"),
        "the retained cache must only exist when the surface supports COPY_DST"
    );
}

#[test]
fn full_redraw_ignores_damage_without_rebuilding_the_draw_list() {
    let draw_list = UiSurfaceDrawList::new(
        (100, 100),
        Some(UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0)),
        vec![
            UiSurfaceCommand {
                z_index: 0,
                frame: UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [255, 0, 0, 255],
                    corner_radius: 0.0,
                },
            },
            UiSurfaceCommand {
                z_index: 1,
                frame: UiSurfaceRect::new(60.0, 60.0, 20.0, 20.0),
                clip: None,
                kind: UiSurfaceCommandKind::Quad {
                    color: [0, 255, 0, 255],
                    corner_radius: 0.0,
                },
            },
        ],
    );

    let damage = render_damage(&draw_list, SurfaceRenderMode::FullRedraw);

    assert_eq!(damage, None);
    assert_eq!(draw_list.stats().visible_command_count, 1);
    assert_eq!(draw_list.commands.len(), 2);
}

#[test]
fn damage_patch_preserves_the_requested_damage_scissor() {
    let damage = UiSurfaceRect::new(10.0, 10.0, 20.0, 20.0);
    let draw_list = UiSurfaceDrawList::new((100, 100), Some(damage), Vec::new());

    assert_eq!(
        render_damage(&draw_list, SurfaceRenderMode::DamagePatch),
        Some(damage)
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
fn wgpu_ui_surface_image_cache_admission_evicts_the_oldest_inactive_entry() {
    let (action, visits) = image_cache_admission_plan(
        [
            ("oldest", 0, 1, 4, false, false),
            ("recent", 0, 10, 4, false, false),
            ("middle", 0, 5, 4, false, false),
            ("newest", 0, 20, 4, true, false),
        ]
        .into_iter(),
        5,
        20,
        4,
        64,
        4,
    );

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Admit {
            evict_keys: vec![("oldest".into(), 0)]
        }
    );
    assert_eq!(visits, 4);
}

#[test]
fn wgpu_ui_surface_image_cache_admission_is_stable_for_ties() {
    let (action, visits) = image_cache_admission_plan(
        [
            ("b", 0, 1, 4, false, false),
            ("c", 0, 1, 4, false, false),
            ("a", 0, 1, 4, false, false),
            ("d", 0, 2, 4, true, false),
        ]
        .into_iter(),
        5,
        20,
        4,
        64,
        4,
    );

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Admit {
            evict_keys: vec![("a".into(), 0)]
        }
    );
    assert_eq!(visits, 4);
}

#[test]
fn wgpu_ui_surface_image_cache_admission_distinguishes_generations_of_one_key() {
    let (action, visits) = image_cache_admission_plan(
        [
            ("atlas://editor/icons", 4, 1, 16, false, false),
            ("atlas://editor/icons", 5, 2, 16, false, false),
        ]
        .into_iter(),
        3,
        48,
        2,
        64,
        16,
    );

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Admit {
            evict_keys: vec![("atlas://editor/icons".into(), 4)]
        }
    );
    assert_eq!(visits, 2);
}

#[test]
fn wgpu_ui_surface_image_cache_admission_has_zero_stable_work_under_budget() {
    let (action, visits) = image_cache_admission_plan(
        [("one", 0, 1, 4, true, false), ("two", 0, 2, 4, true, false)].into_iter(),
        3,
        12,
        3,
        16,
        4,
    );

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Admit {
            evict_keys: Vec::new()
        }
    );
    assert_eq!(visits, 0);
}

#[test]
fn wgpu_ui_surface_image_cache_rejects_growth_when_the_hard_budget_is_fully_active() {
    let (action, visits) = image_cache_admission_plan(
        [
            ("one", 0, 7, 4, true, false),
            ("two", 0, 7, 4, true, false),
            ("three", 0, 7, 4, true, false),
        ]
        .into_iter(),
        4,
        16,
        3,
        64,
        4,
    );

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Reject {
            entry_saturated: true
        }
    );
    assert_eq!(visits, 3);
}

#[test]
fn wgpu_ui_surface_image_cache_byte_budget_evicts_multiple_inactive_entries() {
    let (action, visits) = image_cache_admission_plan(
        [
            ("old-a", 0, 1, 8, false, false),
            ("old-b", 0, 2, 8, false, false),
            ("active", 0, 3, 8, true, false),
        ]
        .into_iter(),
        4,
        40,
        8,
        24,
        16,
    );

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Admit {
            evict_keys: vec![("old-a".into(), 0), ("old-b".into(), 0)]
        }
    );
    assert_eq!(visits, 3);
}

#[test]
fn wgpu_ui_surface_image_cache_rejects_a_single_oversized_resource_without_saturation() {
    let (action, visits) = image_cache_admission_plan(std::iter::empty(), 1, 65, 8, 64, 65);

    assert_eq!(
        action,
        ImageCacheAdmissionAction::Reject {
            entry_saturated: false
        }
    );
    assert_eq!(visits, 0);
}

#[test]
fn wgpu_ui_surface_image_cache_byte_rejection_does_not_poison_a_smaller_admission() {
    let active = [("active", 0, 3, 60, true, false)];
    let (large, large_visits) = image_cache_admission_plan(active.into_iter(), 2, 70, 4, 64, 10);
    let (small, small_visits) = image_cache_admission_plan(active.into_iter(), 2, 64, 4, 64, 4);

    assert_eq!(
        large,
        ImageCacheAdmissionAction::Reject {
            entry_saturated: false
        }
    );
    assert_eq!(large_visits, 1);
    assert_eq!(
        small,
        ImageCacheAdmissionAction::Admit {
            evict_keys: Vec::new()
        }
    );
    assert_eq!(small_visits, 0);
}

#[test]
fn invalid_supplied_image_payload_invalidates_a_previous_same_key_resource() {
    let mut cache = HashMap::from([(
        "viewport".to_string(),
        BTreeMap::from([(12_u64, 7_u64), (13_u64, 8_u64)]),
    )]);

    assert_eq!(remove_cached_image(&mut cache, "viewport", 12), Some(7));
    assert_eq!(
        cache
            .get("viewport")
            .and_then(|generations| generations.get(&13)),
        Some(&8)
    );
    assert_eq!(remove_cached_image(&mut cache, "viewport", 13), Some(8));
    assert!(!cache.contains_key("viewport"));
}

#[test]
fn versioned_image_uploads_skip_an_unchanged_generation() {
    assert!(image_upload_needs_write(Some(17), None));
    assert!(!image_upload_needs_write(Some(17), Some(17)));
    assert!(image_upload_needs_write(Some(18), Some(17)));
    assert!(image_upload_needs_write(None, Some(17)));
}

#[test]
fn image_payload_layout_rejects_unsupported_and_overflowing_extents() {
    assert_eq!(
        image_payload_layout(2, 2, 4_096),
        Some(ImagePayloadLayout {
            expected_len: 16,
            bytes_per_row: 8,
        })
    );
    assert_eq!(image_payload_layout(0, 2, 4_096), None);
    assert_eq!(image_payload_layout(2, 0, 4_096), None);
    assert_eq!(image_payload_layout(4_097, 2, 4_096), None);
    assert_eq!(image_payload_layout(u32::MAX, 1, u32::MAX), None);
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
                resource_generation: 0,
                width: 4,
                height: 4,
                upload_bytes: 64,
                rgba: Some(vec![255; 64]),
                atlas_uv: Some(atlas_uv),
            },
        },
    }
}
