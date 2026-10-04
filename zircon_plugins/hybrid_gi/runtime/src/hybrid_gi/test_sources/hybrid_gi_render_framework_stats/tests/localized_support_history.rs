use super::*;

#[test]
#[ignore]
fn export_hybrid_gi_localized_support_history_wgpu_png() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let stable_extract = scene_representation_extract_with_card_positions(
        viewport_size,
        model.clone(),
        emissive_material.clone(),
        black_material,
        RenderHybridGiDebugView::None,
        Vec3::ONE,
        false,
        Vec3::new(-1.5, 0.0, 0.0),
        Vec3::new(1.5, 0.0, 0.0),
    );
    let changed_extract = scene_representation_extract_with_card_positions(
        viewport_size,
        model,
        emissive_material.clone(),
        emissive_material,
        RenderHybridGiDebugView::None,
        Vec3::ONE,
        false,
        Vec3::new(-1.5, 0.0, 0.0),
        Vec3::new(1.5, 0.0, 0.0),
    );

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
        .unwrap();

    server
        .submit_frame_extract(viewport, stable_extract.clone())
        .unwrap();
    server
        .submit_frame_extract(viewport, stable_extract.clone())
        .unwrap();
    server
        .submit_frame_extract(viewport, stable_extract)
        .unwrap();
    let stable_stats = server.query_stats().unwrap();
    assert!(stable_stats.last_frame_history_status.previous_available);
    assert!(
        stable_stats
            .last_frame_history_copy_report
            .global_illumination_copied
    );
    let stable_history = stable_stats.last_frame_history;
    let stable_frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("stable localized-support HGI frame should be capturable");

    server
        .submit_frame_extract(viewport, changed_extract)
        .unwrap();
    let changed_stats = server.query_stats().unwrap();
    assert!(!changed_stats.last_frame_history_status.previous_available);
    assert_eq!(
        changed_stats.last_frame_history_status.invalidation_reason,
        Some(FrameHistoryInvalidationReason::FrameInputsChanged)
    );
    assert_eq!(
        changed_stats.last_frame_history, stable_history,
        "frame-input changes should retain HGI history allocation for local validity checks"
    );
    assert!(
        changed_stats
            .last_frame_history_copy_report
            .global_illumination_copied
    );
    assert_eq!(changed_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    let changed_frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("changed localized-support HGI frame should be capturable");

    let stable_metrics = frame_metrics(&stable_frame);
    let changed_metrics = frame_metrics(&changed_frame);
    assert!(stable_metrics.visible_pixels > 0 && changed_metrics.visible_pixels > 0);
    let unchanged_left_delta = average_region_rgb_delta(
        &stable_frame,
        &changed_frame,
        viewport_size,
        0.08,
        0.48,
        0.2,
        0.8,
    );
    let changed_right_delta = average_region_rgb_delta(
        &stable_frame,
        &changed_frame,
        viewport_size,
        0.52,
        0.92,
        0.2,
        0.8,
    );
    assert!(
        changed_right_delta > unchanged_left_delta + 0.1,
        "localized material change should alter the right support more than the stable left support: left_delta={unchanged_left_delta:.3}, right_delta={changed_right_delta:.3}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_side_by_side_png(
        output_dir.join(LOCALIZED_SUPPORT_HISTORY_WGPU_PNG),
        &stable_frame,
        &changed_frame,
    );
    fs::write(
        output_dir.join(LOCALIZED_SUPPORT_HISTORY_WGPU_REPORT),
        format!(
            "png={}\nleft=stable_surface_cache_and_dark_voxel_card\nright=unchanged_surface_cache_and_changed_emissive_voxel_card\nwidth={}\nheight={}\nstable_visible_pixels={}\nchanged_visible_pixels={}\nstable_max_luma={:.2}\nchanged_max_luma={:.2}\nunchanged_left_average_rgb_delta={:.3}\nchanged_right_average_rgb_delta={:.3}\nstable_history_handle={:?}\nchanged_history_handle={:?}\nchanged_global_history_previous_available={}\nchanged_global_history_invalidation_reason={:?}\nchanged_global_illumination_history_copied={}\nchanged_hybrid_gi_graph_executed_pass_count={}\ntrace_tile_contract=7_words_with_local_surface_page_or_voxel_cell_support_signature\nhgi_history_allocation=preserved_for_frame_inputs_changed\nhgi_temporal_validity=per_pixel_depth+source+local_support_signature+motion+luma\nvalidated_wgpu_test=resolve_temporal_history_reuses_unchanged_support_and_rejects_changed_neighbor\n",
            LOCALIZED_SUPPORT_HISTORY_WGPU_PNG,
            stable_frame.width + 1 + changed_frame.width,
            stable_frame.height,
            stable_metrics.visible_pixels,
            changed_metrics.visible_pixels,
            stable_metrics.max_luma,
            changed_metrics.max_luma,
            unchanged_left_delta,
            changed_right_delta,
            stable_history,
            changed_stats.last_frame_history,
            changed_stats.last_frame_history_status.previous_available,
            changed_stats.last_frame_history_status.invalidation_reason,
            changed_stats
                .last_frame_history_copy_report
                .global_illumination_copied,
            changed_stats.last_hybrid_gi_graph_executed_pass_count,
        ),
    )
    .unwrap();
}

fn average_region_rgb_delta(
    before: &CapturedFrame,
    after: &CapturedFrame,
    size: UVec2,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
) -> f32 {
    let channel_delta = |channel| {
        (average_region_channel(&before.rgba, size, channel, x_min, x_max, y_min, y_max)
            - average_region_channel(&after.rgba, size, channel, x_min, x_max, y_min, y_max))
        .abs()
    };
    (channel_delta(0) + channel_delta(1) + channel_delta(2)) / 3.0
}
