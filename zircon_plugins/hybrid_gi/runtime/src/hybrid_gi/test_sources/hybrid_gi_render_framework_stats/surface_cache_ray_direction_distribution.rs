use super::*;

#[test]
#[ignore]
fn export_hybrid_gi_surface_cache_ray_direction_distribution_wgpu_png() {
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
        .expect("surface-cache ray-direction Wgpu product frame capture should be available");
    let metrics = frame_metrics(&frame);
    assert!(
        metrics.visible_pixels > 0 && metrics.max_luma > 8.0,
        "expected nonblank surface-cache ray-direction Wgpu frame; metrics={metrics:?}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_png(
        output_dir.join(SURFACE_CACHE_RAY_DIRECTION_DISTRIBUTION_WGPU_PNG),
        &frame,
    );
    fs::write(
        output_dir.join(SURFACE_CACHE_RAY_DIRECTION_DISTRIBUTION_WGPU_REPORT),
        format!(
            "png={}\nwidth={}\nheight={}\ngeneration={}\nvisible_pixels={}\nmin_luma={:.2}\nmax_luma={:.2}\nproduct_debug_view=surface_cache\nhybrid_gi_quality=high\nquality_tracing_budget=32\nsurface_cache_rays_per_trace_tile=16\ngpu_probe_trace_tile_surface_cache_ray_direction_distribution=trace_probe_tiles_compute+sample_id_octant_axis_diagonal_distribution\ngpu_probe_trace_tile_surface_cache_ray_march=trace_probe_tiles_compute+atlas_depth_multi_step_march\ngpu_probe_trace_tile_surface_cache_sampling=trace_probe_tiles_compute+surface_cache_atlas_depth_texture_load\nvalidated_surface_cache_ray_direction_distribution_shader=trace_probe_tiles_shader_distributes_surface_cache_ray_steps_by_sample_id_diagonal_near_depth_texel\nvalidated_surface_cache_ray_march_shader=trace_probe_tiles_shader_marches_surface_cache_depth_before_voxel_fallback_weighted_near_depth_texels\nvalidated_quality_ray_mapping=low_4_medium_8_high_16_default_8\nlumen_reference=GenerateRays_EquiAreaSphericalMapping_screen_probe_tracing_octahedron_plus_TraceScreen_ray_texel_trace\nfirst_hybrid_gi_surface_cache_depth_sample_count={}\nfirst_hybrid_gi_probe_trace_tile_count={}\nlast_hybrid_gi_graph_executed_pass_count={}\nlast_hybrid_gi_cache_entry_count={}\nlast_hybrid_gi_probe_trace_tile_count={}\nlast_hybrid_gi_probe_trace_dispatch_group_count={:?}\nlast_hybrid_gi_scene_screen_probe_count={}\nlast_hybrid_gi_scene_radiance_cache_entry_count={}\nlast_hybrid_gi_surface_cache_resident_page_count={}\nlast_hybrid_gi_surface_cache_depth_sample_count={}\nlast_hybrid_gi_voxel_resident_clipmap_count={}\n",
            SURFACE_CACHE_RAY_DIRECTION_DISTRIBUTION_WGPU_PNG,
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
