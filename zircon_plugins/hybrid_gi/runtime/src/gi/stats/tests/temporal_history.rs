use super::*;

#[test]
#[ignore]
fn export_hybrid_gi_temporal_history_rejection_wgpu_png() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let warm_extract = scene_representation_extract_with_debug_view_and_key_light(
        viewport_size,
        model.clone(),
        black_material.clone(),
        emissive_material.clone(),
        RenderHybridGiDebugView::None,
        Vec3::new(1.0, 0.28, 0.18),
        false,
    );
    let cool_extract = scene_representation_extract_with_debug_view_and_key_light(
        viewport_size,
        model,
        black_material,
        emissive_material,
        RenderHybridGiDebugView::None,
        Vec3::new(0.18, 0.38, 1.0),
        false,
    );

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
        .unwrap();

    server
        .submit_frame_extract(viewport, warm_extract.clone())
        .unwrap();
    let first_stats = server.query_stats().unwrap();
    assert!(!first_stats.last_frame_history_status.previous_available);
    assert_eq!(
        first_stats.last_frame_history_status.invalidation_reason,
        Some(FrameHistoryInvalidationReason::NoPreviousFrame)
    );

    server
        .submit_frame_extract(viewport, warm_extract.clone())
        .unwrap();
    server.submit_frame_extract(viewport, warm_extract).unwrap();
    let stable_stats = server.query_stats().unwrap();
    assert!(stable_stats.last_frame_history_status.previous_available);
    assert_eq!(
        stable_stats.last_frame_history_status.invalidation_reason,
        None
    );
    assert_eq!(stable_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert!(
        stable_stats
            .last_frame_history_copy_report
            .global_illumination_copied,
        "static HGI frame should persist both resolved lighting and temporal metadata"
    );
    let stable_frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("stable temporal HGI Wgpu frame capture should be available");

    server.submit_frame_extract(viewport, cool_extract).unwrap();
    let rejected_stats = server.query_stats().unwrap();
    assert!(!rejected_stats.last_frame_history_status.previous_available);
    assert_eq!(
        rejected_stats.last_frame_history_status.invalidation_reason,
        Some(FrameHistoryInvalidationReason::FrameInputsChanged)
    );
    assert_eq!(rejected_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert!(
        rejected_stats
            .last_frame_history_copy_report
            .global_illumination_copied,
        "rejected HGI frame should replace lighting and temporal metadata history"
    );
    let rejected_frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("scene-changed temporal HGI Wgpu frame capture should be available");

    let stable_metrics = frame_metrics(&stable_frame);
    let rejected_metrics = frame_metrics(&rejected_frame);
    assert!(
        stable_metrics.visible_pixels > 0 && rejected_metrics.visible_pixels > 0,
        "expected nonblank temporal HGI product frames; stable={stable_metrics:?}, rejected={rejected_metrics:?}"
    );
    let stable_red =
        average_region_channel(&stable_frame.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let stable_blue =
        average_region_channel(&stable_frame.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let rejected_red = average_region_channel(
        &rejected_frame.rgba,
        viewport_size,
        0,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    let rejected_blue = average_region_channel(
        &rejected_frame.rgba,
        viewport_size,
        2,
        0.25,
        0.75,
        0.25,
        0.75,
    );
    assert!(
        stable_red > rejected_red + 0.25,
        "scene-signature rejection should not retain a visible warm ghost; stable_red={stable_red:.2}, rejected_red={rejected_red:.2}"
    );
    assert!(
        rejected_blue > stable_blue + 0.25,
        "scene-signature rejection should expose the new cool HGI result immediately; stable_blue={stable_blue:.2}, rejected_blue={rejected_blue:.2}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_side_by_side_png(
        output_dir.join(TEMPORAL_HISTORY_REJECTION_WGPU_PNG),
        &stable_frame,
        &rejected_frame,
    );
    fs::write(
        output_dir.join(TEMPORAL_HISTORY_REJECTION_WGPU_REPORT),
        format!(
            "png={}\nleft=static_scene_history_reuse\nright=changed_light_history_rejection\nwidth={}\nheight={}\nstable_generation={}\nrejected_generation={}\nstable_visible_pixels={}\nrejected_visible_pixels={}\nstable_min_luma={:.2}\nstable_max_luma={:.2}\nrejected_min_luma={:.2}\nrejected_max_luma={:.2}\nstable_center_red={:.2}\nrejected_center_red={:.2}\nstable_center_blue={:.2}\nrejected_center_blue={:.2}\nfirst_history_previous_available={}\nfirst_history_invalidation_reason={:?}\nstable_history_previous_available={}\nstable_history_invalidation_reason={:?}\nstable_global_illumination_history_copied={}\nstable_hybrid_gi_graph_executed_pass_count={}\nrejected_history_previous_available={}\nrejected_history_invalidation_reason={:?}\nrejected_global_illumination_history_copied={}\nrejected_hybrid_gi_graph_executed_pass_count={}\ntemporal_metadata=hit_depth+trace_source+scene_signature+confidence_rgba16_float\ntemporal_reprojection=scene_velocity_uv_reprojection\ntemporal_acceptance=onscreen+motion+depth+trace_source+scene_signature+luma\ntemporal_accumulation=confidence_weighted_history_with_current_neighborhood_clamp\ntemporal_rejection=frame_history_validity_or_per_pixel_support_change_resets_confidence\nvalidated_wgpu_tests=static_accumulation+motion_rejection+scene_signature_or_trace_source_rejection\nlumen_reference=ScreenProbeGather_TemporalReprojection_history_depth_frames_accumulated_fast_update\n",
            TEMPORAL_HISTORY_REJECTION_WGPU_PNG,
            stable_frame.width + 1 + rejected_frame.width,
            stable_frame.height,
            stable_frame.generation,
            rejected_frame.generation,
            stable_metrics.visible_pixels,
            rejected_metrics.visible_pixels,
            stable_metrics.min_luma,
            stable_metrics.max_luma,
            rejected_metrics.min_luma,
            rejected_metrics.max_luma,
            stable_red,
            rejected_red,
            stable_blue,
            rejected_blue,
            first_stats.last_frame_history_status.previous_available,
            first_stats.last_frame_history_status.invalidation_reason,
            stable_stats.last_frame_history_status.previous_available,
            stable_stats.last_frame_history_status.invalidation_reason,
            stable_stats
                .last_frame_history_copy_report
                .global_illumination_copied,
            stable_stats.last_hybrid_gi_graph_executed_pass_count,
            rejected_stats.last_frame_history_status.previous_available,
            rejected_stats.last_frame_history_status.invalidation_reason,
            rejected_stats
                .last_frame_history_copy_report
                .global_illumination_copied,
            rejected_stats.last_hybrid_gi_graph_executed_pass_count,
        )
        .replace(
            "hit_depth+trace_source+scene_signature+confidence_rgba16_float",
            "hit_depth+trace_source+local_support_signature+confidence_rgba16_float",
        )
        .replace(
            "onscreen+motion+depth+trace_source+scene_signature+luma",
            "onscreen+motion+depth+trace_source+local_support_signature+luma",
        ),
    )
    .unwrap();
}
