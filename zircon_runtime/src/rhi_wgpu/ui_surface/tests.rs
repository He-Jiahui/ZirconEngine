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
    assert_eq!(stats.compiled_draw_calls, 1);
    assert_eq!(stats.render_pass_count, 0);
    assert_eq!(stats.retained_cache_copy_bytes, 0);
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
    assert_eq!(stats.image_vertex_count, 0);
    assert_eq!(stats.compiled_image_vertex_count, 0);
    assert_eq!(stats.image_cache_key_allocation_count, 0);
    assert_eq!(stats.image_cache_prune_visit_count, 0);
    assert_eq!(stats.image_prepare_command_visit_count, 0);
    assert_eq!(stats.image_prepare_cache_hit_count, 0);
    assert_eq!(stats.image_upload_write_count, 0);
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
            ("oldest", 1, 4, false, false),
            ("recent", 10, 4, false, false),
            ("middle", 5, 4, false, false),
            ("newest", 20, 4, true, false),
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
            evict_keys: vec!["oldest".into()]
        }
    );
    assert_eq!(visits, 4);
}

#[test]
fn wgpu_ui_surface_image_cache_admission_is_stable_for_ties() {
    let (action, visits) = image_cache_admission_plan(
        [
            ("b", 1, 4, false, false),
            ("c", 1, 4, false, false),
            ("a", 1, 4, false, false),
            ("d", 2, 4, true, false),
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
            evict_keys: vec!["a".into()]
        }
    );
    assert_eq!(visits, 4);
}

#[test]
fn wgpu_ui_surface_image_cache_admission_has_zero_stable_work_under_budget() {
    let (action, visits) = image_cache_admission_plan(
        [("one", 1, 4, true, false), ("two", 2, 4, true, false)].into_iter(),
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
            ("one", 7, 4, true, false),
            ("two", 7, 4, true, false),
            ("three", 7, 4, true, false),
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
            cache_saturated: true
        }
    );
    assert_eq!(visits, 3);
}

#[test]
fn wgpu_ui_surface_image_cache_byte_budget_evicts_multiple_inactive_entries() {
    let (action, visits) = image_cache_admission_plan(
        [
            ("old-a", 1, 8, false, false),
            ("old-b", 2, 8, false, false),
            ("active", 3, 8, true, false),
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
            evict_keys: vec!["old-a".into(), "old-b".into()]
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
            cache_saturated: false
        }
    );
    assert_eq!(visits, 0);
}

#[test]
fn invalid_supplied_image_payload_invalidates_a_previous_same_key_resource() {
    let mut cache = HashMap::from([("viewport".to_string(), 7_u64)]);

    assert_eq!(remove_cached_image(&mut cache, "viewport"), Some(7));
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
                width: 4,
                height: 4,
                upload_bytes: 64,
                rgba: Some(vec![255; 64]),
                atlas_uv: Some(atlas_uv),
            },
        },
    }
}

#[test]
fn wgpu_ui_surface_render_pass_coalesces_contiguous_non_text_ops() {
    let source = include_str!("render_pass.rs");
    let compact = source.split_whitespace().collect::<String>();

    assert!(
        compact.contains("letrun_end=non_text_run_end(draw_ops,op_index);"),
        "render recording must find the complete contiguous solid/image run"
    );
    assert!(
        compact.contains("forrun_indexinop_index..run_end{"),
        "one render pass must record every op in the contiguous non-text run"
    );
}

#[test]
fn wgpu_ui_surface_marks_the_complete_present_submission_for_renderdoc() {
    let source = include_str!("../ui_surface.rs");

    assert!(
        source.contains("encoder.push_debug_group(\"zircon::UI\");"),
        "the full UI submission must be grouped under the standard RenderDoc UI marker"
    );
    assert!(
        source.contains("encoder.pop_debug_group();"),
        "the UI debug group must close before command submission"
    );
}
