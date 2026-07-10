use super::*;

#[test]
#[ignore]
fn export_hybrid_gi_surface_cache_hzb_trace_wgpu_png() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let extract = scene_representation_extract_with_debug_view(
        viewport_size,
        model,
        black_material,
        emissive_material,
        RenderHybridGiDebugView::SurfaceCache,
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
    assert!(first_stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1);
    assert!(first_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert!(stats.last_hybrid_gi_cache_entry_count >= 1);
    assert!(stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(stats.last_hybrid_gi_scene_screen_probe_count >= 2);
    assert!(stats.last_hybrid_gi_scene_radiance_cache_entry_count >= 2);
    assert!(stats.last_hybrid_gi_surface_cache_resident_page_count >= 1);

    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("surface-cache HZB trace Wgpu product frame capture should be available");
    let metrics = frame_metrics(&frame);
    assert!(
        metrics.visible_pixels > 0 && metrics.max_luma > 8.0,
        "expected nonblank surface-cache HZB trace Wgpu frame; metrics={metrics:?}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_png(output_dir.join(SURFACE_CACHE_HZB_TRACE_WGPU_PNG), &frame);
    fs::write(
        output_dir.join(SURFACE_CACHE_HZB_TRACE_WGPU_REPORT),
        format!(
            "png={}\nwidth={}\nheight={}\ngeneration={}\nvisible_pixels={}\nmin_luma={:.2}\nmax_luma={:.2}\nproduct_debug_view=surface_cache\ngpu_surface_cache_depth_hierarchy=build_surface_cache_depth_hierarchy_compute+rgba8_min_max_depth_range_mips\nsurface_cache_hzb_page_mip_contract=seven_levels_mip0_to_mip6_one_texel_per_64x64_aligned_page\ngpu_probe_trace_tile_surface_cache_hzb=trace_probe_tiles_compute+aligned_block_skip+coarse_to_fine_refinement\ngpu_probe_trace_tile_surface_cache_ray_direction_distribution=sample_id_octant_axis_diagonal_distribution\ngpu_probe_trace_tile_voxel_miss_fallback=scene_prepare_voxel_cell_descriptor_radiance\nvalidated_surface_cache_hzb_shader=trace_probe_tiles_shader_uses_surface_cache_hzb_to_skip_depth_disjoint_blocks_exact_far_near_depth_hit_rgb\nlumen_reference=HZB0_FurthestClosestReduction_plus_TraceScreen_InternalTraceScreen_mip_tile_skip_and_thickness_refinement\nfirst_hybrid_gi_surface_cache_depth_sample_count={}\nfirst_hybrid_gi_probe_trace_tile_count={}\nlast_hybrid_gi_graph_executed_pass_count={}\nlast_hybrid_gi_cache_entry_count={}\nlast_hybrid_gi_probe_trace_tile_count={}\nlast_hybrid_gi_probe_trace_dispatch_group_count={:?}\nlast_hybrid_gi_scene_screen_probe_count={}\nlast_hybrid_gi_scene_radiance_cache_entry_count={}\nlast_hybrid_gi_surface_cache_resident_page_count={}\nlast_hybrid_gi_surface_cache_depth_sample_count={}\nlast_hybrid_gi_voxel_resident_clipmap_count={}\n",
            SURFACE_CACHE_HZB_TRACE_WGPU_PNG,
            frame.width,
            frame.height,
            frame.generation,
            metrics.visible_pixels,
            metrics.min_luma,
            metrics.max_luma,
            first_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            first_stats.last_hybrid_gi_probe_trace_tile_count,
            stats.last_hybrid_gi_graph_executed_pass_count,
            stats.last_hybrid_gi_cache_entry_count,
            stats.last_hybrid_gi_probe_trace_tile_count,
            stats.last_hybrid_gi_probe_trace_dispatch_group_count,
            stats.last_hybrid_gi_scene_screen_probe_count,
            stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            stats.last_hybrid_gi_surface_cache_resident_page_count,
            stats.last_hybrid_gi_surface_cache_depth_sample_count,
            stats.last_hybrid_gi_voxel_resident_clipmap_count,
        ),
    )
    .unwrap();
}
