use super::*;

#[test]
#[ignore]
fn export_hybrid_gi_main_scene_hzb_surface_cache_trace_wgpu_png() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let extract = scene_representation_extract_with_debug_view(
        viewport_size,
        model,
        black_material,
        emissive_material,
        RenderHybridGiDebugView::None,
    );

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
        .unwrap();

    server
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let first_stats = server.query_stats().unwrap();
    server
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();

    assert_eq!(first_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert!(first_stats.last_hybrid_gi_surface_cache_resident_page_count >= 1);
    assert!(first_stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1);
    assert!(first_stats.last_hybrid_gi_voxel_resident_clipmap_count >= 1);
    assert!(stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(stats.last_hybrid_gi_scene_screen_probe_count >= 2);

    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("main-scene HZB trace Wgpu product frame capture should be available");
    let metrics = frame_metrics(&frame);
    assert!(
        metrics.visible_pixels > 0 && metrics.max_luma > 8.0,
        "expected nonblank main-scene HZB trace Wgpu frame; metrics={metrics:?}"
    );
    let center_red = average_region_channel(&frame.rgba, viewport_size, 0, 0.4, 0.6, 0.4, 0.6);
    let center_green = average_region_channel(&frame.rgba, viewport_size, 1, 0.4, 0.6, 0.4, 0.6);
    let center_blue = average_region_channel(&frame.rgba, viewport_size, 2, 0.4, 0.6, 0.4, 0.6);

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_png(
        output_dir.join(MAIN_SCENE_HZB_SURFACE_CACHE_TRACE_WGPU_PNG),
        &frame,
    );
    fs::write(
        output_dir.join(MAIN_SCENE_HZB_SURFACE_CACHE_TRACE_WGPU_REPORT),
        format!(
            "png={}\nwidth={}\nheight={}\ngeneration={}\nvisible_pixels={}\nmin_luma={:.2}\nmax_luma={:.2}\ncenter_red={:.2}\ncenter_green={:.2}\ncenter_blue={:.2}\nproduct_debug_view=none\ngpu_main_scene_hzb_build=command_ordered_per_mip_params+single_or_msaa_depth_variant\nhgi_scene_hzb_handoff=scene_depth_handoff_8x8_tiles+resolver_checked_full_mip_view\nhgi_camera_packet=jittered_current_frame_inverse_view_projection+camera_position+viewport\nhgi_main_scene_screen_trace=tile_footprint_mip+direction_safe_segment_skip+coarse_to_fine_mip0\nhgi_screen_hit_radiance=world_space_surface_cache_page_then_voxel_clipmap_fallback\nhgi_trace_resolve=screen_uv_to_8x8_trace_packet_hit_depth+world_distance+radiance+source_flags\nplugin_buffer_contract=feature_declared_scene_2840_bytes+trace_1792_bytes_minimum\nvalidated_wgpu_tests=trace_schedule_shader_marches_main_scene_hzb_and_samples_surface_cache_radiance+trace_schedule_shader_falls_back_to_world_space_voxel_clipmap_radiance\nlumen_reference=HZB0_FurthestClosestReduction+TraceScreen_InternalTraceScreen+surface_cache_then_voxel_fallback\nfirst_hybrid_gi_graph_executed_pass_count={}\nfirst_hybrid_gi_surface_cache_resident_page_count={}\nfirst_hybrid_gi_surface_cache_depth_sample_count={}\nfirst_hybrid_gi_voxel_resident_clipmap_count={}\nlast_hybrid_gi_graph_executed_pass_count={}\nlast_hybrid_gi_probe_trace_tile_count={}\nlast_hybrid_gi_scene_screen_probe_count={}\nlast_hybrid_gi_scene_radiance_cache_entry_count={}\n",
            MAIN_SCENE_HZB_SURFACE_CACHE_TRACE_WGPU_PNG,
            frame.width,
            frame.height,
            frame.generation,
            metrics.visible_pixels,
            metrics.min_luma,
            metrics.max_luma,
            center_red,
            center_green,
            center_blue,
            first_stats.last_hybrid_gi_graph_executed_pass_count,
            first_stats.last_hybrid_gi_surface_cache_resident_page_count,
            first_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            first_stats.last_hybrid_gi_voxel_resident_clipmap_count,
            stats.last_hybrid_gi_graph_executed_pass_count,
            stats.last_hybrid_gi_probe_trace_tile_count,
            stats.last_hybrid_gi_scene_screen_probe_count,
            stats.last_hybrid_gi_scene_radiance_cache_entry_count,
        )
        .replace(
            "trace_1792_bytes_minimum",
            "trace_2048_bytes_minimum_with_local_support_signature",
        ),
    )
    .unwrap();
}
