use super::*;

const M5_PERFORMANCE_SETTLE_FRAME_COUNT: usize = 300;
const M5_PERFORMANCE_WARM_SAMPLE_COUNT: usize = 31;

#[test]
#[ignore]
fn export_hybrid_gi_voxel_miss_fallback_wgpu_png() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let extract =
        scene_representation_extract(viewport_size, model, black_material, emissive_material);

    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    let cold_submit_started = Instant::now();
    server
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let cold_submit_wall_time_us =
        u64::try_from(cold_submit_started.elapsed().as_micros()).unwrap_or(u64::MAX);

    let first_stats = server.query_stats().unwrap();
    assert_eq!(first_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(first_stats.last_hybrid_gi_scene_card_count, 2);
    assert!(first_stats.last_hybrid_gi_surface_cache_resident_page_count >= 1);
    assert!(first_stats.last_hybrid_gi_voxel_resident_clipmap_count >= 1);
    assert!(
        !first_stats.last_hybrid_gi_global_sdf_mesh_projection_cache_hit,
        "the first product frame must measure the cold mesh projection path"
    );

    let renderdoc_capture_requested = std::env::var_os("ZR_HGI_M5_RENDERDOC_CAPTURE").is_some();
    if renderdoc_capture_requested {
        server.request_graphics_debugger_capture(viewport).unwrap();
    }

    let readback_wait_started = Instant::now();
    let readback_deadline = readback_wait_started + GPU_READBACK_EVIDENCE_TIMEOUT;
    let mut followup_frame_count = 0_usize;
    let mut last_stats = first_stats.clone();
    let mut gpu_readback_stats = None;
    for _ in 0..GPU_READBACK_EVIDENCE_FRAME_LIMIT {
        server
            .submit_frame_extract(viewport, extract.clone())
            .unwrap();
        followup_frame_count = followup_frame_count.saturating_add(1);
        last_stats = server.query_stats().unwrap();
        if renderdoc_capture_requested && followup_frame_count == 1 {
            let capture_status = server.query_graphics_debugger_status().unwrap();
            assert!(!capture_status.capture_pending);
            assert_eq!(
                capture_status.last_capture_frame,
                last_stats.last_generation
            );
            assert_eq!(capture_status.last_error, None);
        }
        if last_stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1
            && last_stats.last_hybrid_gi_probe_trace_tile_count >= 1
            && last_stats.last_hybrid_gi_global_sdf_mesh_projection_cache_hit
        {
            gpu_readback_stats = Some(last_stats.clone());
            break;
        }
        if Instant::now() >= readback_deadline {
            break;
        }
        thread::sleep(GPU_READBACK_EVIDENCE_POLL_INTERVAL);
    }
    let readback_wait_wall_time_us =
        u64::try_from(readback_wait_started.elapsed().as_micros()).unwrap_or(u64::MAX);
    let mut stats = gpu_readback_stats.unwrap_or_else(|| {
        panic!(
            "bounded follow-up frames must publish product GPU readback on the warm mesh projection path; depth_samples={}, trace_tiles={}, cache_hit={}, in_flight={}, completed={}, followup_frames={}, wait_us={}",
            last_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            last_stats.last_hybrid_gi_probe_trace_tile_count,
            last_stats.last_hybrid_gi_global_sdf_mesh_projection_cache_hit,
            last_stats.last_readback_in_flight_count,
            last_stats.last_readback_completed_count,
            followup_frame_count,
            readback_wait_wall_time_us,
        )
    });
    assert_eq!(
        stats.last_hybrid_gi_global_sdf_cpu_mesh_scene_sync_time_us, 0,
        "a warm product frame must not resynchronize the unchanged mesh SDF scene"
    );
    assert_eq!(
        stats.last_hybrid_gi_probe_trace_dispatch_group_count[0..2],
        [1, 1]
    );
    assert!(stats.last_hybrid_gi_probe_trace_dispatch_group_count[2] >= 1);
    assert!(
        stats.last_hybrid_gi_cache_entry_count >= 1,
        "expected stateful runtime prepare collector GPU readback to feed provider cache entries"
    );

    for _ in 0..M5_PERFORMANCE_SETTLE_FRAME_COUNT {
        server
            .submit_frame_extract(viewport, extract.clone())
            .unwrap();
    }
    let mut warm_submit_wall_time_us = [0_u64; M5_PERFORMANCE_WARM_SAMPLE_COUNT];
    let mut warm_cpu_prepare_time_us = [0_u64; M5_PERFORMANCE_WARM_SAMPLE_COUNT];
    let mut warm_cache_lookup_time_us = [0_u64; M5_PERFORMANCE_WARM_SAMPLE_COUNT];
    let mut warm_cache_hit_count = 0_usize;
    let mut warm_zero_scene_sync_count = 0_usize;
    let mut warm_zero_transient_upload_count = 0_usize;
    for sample_index in 0..M5_PERFORMANCE_WARM_SAMPLE_COUNT {
        let submit_started = Instant::now();
        server
            .submit_frame_extract(viewport, extract.clone())
            .unwrap();
        warm_submit_wall_time_us[sample_index] =
            u64::try_from(submit_started.elapsed().as_micros()).unwrap_or(u64::MAX);
        stats = server.query_stats().unwrap();
        warm_cpu_prepare_time_us[sample_index] =
            stats.last_hybrid_gi_global_sdf_cpu_prepare_time_us;
        warm_cache_lookup_time_us[sample_index] =
            stats.last_hybrid_gi_global_sdf_cpu_mesh_object_collection_time_us;
        warm_cache_hit_count +=
            usize::from(stats.last_hybrid_gi_global_sdf_mesh_projection_cache_hit);
        warm_zero_scene_sync_count +=
            usize::from(stats.last_hybrid_gi_global_sdf_cpu_mesh_scene_sync_time_us == 0);
        warm_zero_transient_upload_count += usize::from(
            stats.last_hybrid_gi_global_sdf_transient_upload_byte_count == 0
                && stats.last_hybrid_gi_global_sdf_transient_buffer_creation_count == 0
                && stats.last_hybrid_gi_global_sdf_transient_bind_group_creation_count == 0,
        );
    }
    assert_eq!(
        warm_cache_hit_count, M5_PERFORMANCE_WARM_SAMPLE_COUNT,
        "all settled warm samples must reuse the mesh projection"
    );
    assert_eq!(
        warm_zero_scene_sync_count, M5_PERFORMANCE_WARM_SAMPLE_COUNT,
        "all settled warm samples must avoid mesh SDF scene resynchronization"
    );
    assert_eq!(
        warm_zero_transient_upload_count, M5_PERFORMANCE_WARM_SAMPLE_COUNT,
        "all settled warm samples must avoid Global SDF transient build allocations and uploads"
    );
    let warm_submit_summary = warm_sample_summary(warm_submit_wall_time_us);
    let warm_cpu_prepare_summary = warm_sample_summary(warm_cpu_prepare_time_us);
    let warm_cache_lookup_summary = warm_sample_summary(warm_cache_lookup_time_us);

    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("Wgpu scene-representation frame capture should be available");
    let metrics = frame_metrics(&frame);
    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_png(output_dir.join(SCENE_REPRESENTATION_WGPU_PNG), &frame);
    fs::write(
        output_dir.join(SCENE_REPRESENTATION_WGPU_REPORT),
        format!(
            "png={}\nwidth={}\nheight={}\ngeneration={}\nvisible_pixels={}\nmin_luma={:.2}\nmax_luma={:.2}\ngpu_scene_prepare_depth_trace_readback=surface_cache_depth_texture+gpu_probe_trace_tile_buffer\ngpu_probe_trace_tile_generation=generate_probe_trace_tiles_compute+indirect_args_readback\ngpu_probe_trace_tile_dispatch=trace_probe_tiles_compute+writes_probe_trace_lighting_buffer\ngpu_probe_trace_tile_surface_cache_sampling=trace_probe_tiles_compute+surface_cache_atlas_depth_texture_load\ngpu_probe_trace_tile_voxel_miss_fallback=trace_probe_tiles_compute+scene_prepare_voxel_cell_descriptor_radiance\nvalidated_surface_cache_depth_sample_count={}\nvalidated_surface_cache_texture_sampling_shader=trace_probe_tiles_shader_samples_surface_cache_atlas_and_depth_textures_exact_rgb\nvalidated_voxel_miss_fallback_shader=trace_probe_tiles_shader_uses_voxel_cell_descriptor_when_surface_cache_sample_is_invalid_exact_rgb\ngpu_scene_screen_probe_prepare_work_items=screen_probe_descriptors_to_transient_prepare_probes\nneutral_hybrid_gi_prepared_frame_sideband=provider_prepare_output_resident_screen_probes+probe_scene_data\nruntime_prepare_material_capture_context=collector_context_material_capture_seed+sample_texture_rgba_from_resource_streamer\nruntime_prepare_collector_execution=stateful_gpu_prepare_pending_readback_collected\nruntime_prepare_scene_prepare_reconstruction=deferred_pending_neutral_scene_prepare_to_internal_card_requests\nvalidated_provider_prepared_frame_resident_screen_probe_count=2\nvalidated_runtime_prepare_transient_screen_probe_count=2\ncold_submit_wall_time_us={}\ngpu_readback_wait_wall_time_us={}\ngpu_readback_followup_frame_count={}\ncold_global_sdf_cpu_prepare_time_us={}\ncold_global_sdf_cpu_mesh_object_collection_time_us={}\ncold_global_sdf_cpu_mesh_scene_sync_time_us={}\ncold_global_sdf_mesh_projection_cache_hit={}\nwarm_global_sdf_cpu_prepare_time_us={}\nwarm_global_sdf_cpu_mesh_object_collection_time_us={}\nwarm_global_sdf_cpu_mesh_scene_sync_time_us={}\nwarm_global_sdf_mesh_projection_cache_hit={}\nlast_hybrid_gi_graph_executed_pass_count={}\nlast_hybrid_gi_cache_entry_count={}\nlast_hybrid_gi_scene_card_count={}\nlast_hybrid_gi_surface_cache_resident_page_count={}\nlast_hybrid_gi_surface_cache_feedback_card_count={}\nlast_hybrid_gi_surface_cache_depth_sample_count={}\nlast_hybrid_gi_probe_trace_tile_count={}\nlast_hybrid_gi_probe_trace_dispatch_group_count={:?}\nlast_hybrid_gi_scene_screen_probe_count={}\nlast_hybrid_gi_scene_radiance_cache_entry_count={}\nlast_hybrid_gi_voxel_resident_clipmap_count={}\nm5_performance_settle_frame_count={}\nm5_performance_warm_sample_count={}\nwarm_mesh_projection_cache_hit_count={}\nwarm_zero_mesh_scene_sync_count={}\nwarm_zero_transient_upload_count={}\nwarm_submit_wall_time_us_p50={}\nwarm_submit_wall_time_us_p95={}\nwarm_submit_wall_time_us_max={}\nwarm_global_sdf_cpu_prepare_time_us_p50={}\nwarm_global_sdf_cpu_prepare_time_us_p95={}\nwarm_global_sdf_cpu_prepare_time_us_max={}\nwarm_mesh_projection_cache_lookup_time_us_p50={}\nwarm_mesh_projection_cache_lookup_time_us_p95={}\nwarm_mesh_projection_cache_lookup_time_us_max={}\ncold_global_sdf_transient_upload_byte_count={}\nwarm_global_sdf_transient_upload_byte_count={}\n",
            SCENE_REPRESENTATION_WGPU_PNG,
            frame.width,
            frame.height,
            frame.generation,
            metrics.visible_pixels,
            metrics.min_luma,
            metrics.max_luma,
            stats.last_hybrid_gi_surface_cache_depth_sample_count,
            cold_submit_wall_time_us,
            readback_wait_wall_time_us,
            followup_frame_count,
            first_stats.last_hybrid_gi_global_sdf_cpu_prepare_time_us,
            first_stats.last_hybrid_gi_global_sdf_cpu_mesh_object_collection_time_us,
            first_stats.last_hybrid_gi_global_sdf_cpu_mesh_scene_sync_time_us,
            first_stats.last_hybrid_gi_global_sdf_mesh_projection_cache_hit,
            stats.last_hybrid_gi_global_sdf_cpu_prepare_time_us,
            stats.last_hybrid_gi_global_sdf_cpu_mesh_object_collection_time_us,
            stats.last_hybrid_gi_global_sdf_cpu_mesh_scene_sync_time_us,
            stats.last_hybrid_gi_global_sdf_mesh_projection_cache_hit,
            stats.last_hybrid_gi_graph_executed_pass_count,
            stats.last_hybrid_gi_cache_entry_count,
            stats.last_hybrid_gi_scene_card_count,
            stats.last_hybrid_gi_surface_cache_resident_page_count,
            stats.last_hybrid_gi_surface_cache_feedback_card_count,
            stats.last_hybrid_gi_surface_cache_depth_sample_count,
            stats.last_hybrid_gi_probe_trace_tile_count,
            stats.last_hybrid_gi_probe_trace_dispatch_group_count,
            stats.last_hybrid_gi_scene_screen_probe_count,
            stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            stats.last_hybrid_gi_voxel_resident_clipmap_count,
            M5_PERFORMANCE_SETTLE_FRAME_COUNT,
            M5_PERFORMANCE_WARM_SAMPLE_COUNT,
            warm_cache_hit_count,
            warm_zero_scene_sync_count,
            warm_zero_transient_upload_count,
            warm_submit_summary[0],
            warm_submit_summary[1],
            warm_submit_summary[2],
            warm_cpu_prepare_summary[0],
            warm_cpu_prepare_summary[1],
            warm_cpu_prepare_summary[2],
            warm_cache_lookup_summary[0],
            warm_cache_lookup_summary[1],
            warm_cache_lookup_summary[2],
            first_stats.last_hybrid_gi_global_sdf_transient_upload_byte_count,
            stats.last_hybrid_gi_global_sdf_transient_upload_byte_count,
        ),
    )
    .unwrap();
    assert!(
        metrics.visible_pixels > 0 && metrics.max_luma > 8.0,
        "expected nonblack Wgpu product frame; metrics={metrics:?}"
    );
}

fn warm_sample_summary(mut samples: [u64; M5_PERFORMANCE_WARM_SAMPLE_COUNT]) -> [u64; 3] {
    samples.sort_unstable();
    let p50_index = samples.len() / 2;
    let p95_index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    [
        samples[p50_index],
        samples[p95_index],
        samples[samples.len() - 1],
    ]
}

#[test]
#[ignore]
fn export_hybrid_gi_runtime_trace_lighting_product_resolve_wgpu_png() {
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
    assert_eq!(first_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert!(first_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(first_stats.last_hybrid_gi_scene_screen_probe_count >= 2);

    server
        .submit_frame_extract(viewport, extract.clone())
        .unwrap();
    let second_stats = server.query_stats().unwrap();
    assert!(
        second_stats.last_hybrid_gi_cache_entry_count >= 1,
        "expected first GPU trace lighting readback to become provider cache history"
    );

    server.submit_frame_extract(viewport, extract).unwrap();
    let stats = server.query_stats().unwrap();
    assert!(stats.last_hybrid_gi_cache_entry_count >= 1);
    assert!(stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(stats.last_hybrid_gi_scene_screen_probe_count >= 2);
    assert!(stats.last_hybrid_gi_scene_radiance_cache_entry_count >= 2);

    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("Wgpu product frame capture should be available");
    let metrics = frame_metrics(&frame);
    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_png(
        output_dir.join(RUNTIME_TRACE_LIGHTING_PRODUCT_WGPU_PNG),
        &frame,
    );
    fs::write(
        output_dir.join(RUNTIME_TRACE_LIGHTING_PRODUCT_WGPU_REPORT),
        format!(
            "png={}\nwidth={}\nheight={}\ngeneration={}\nvisible_pixels={}\nmin_luma={:.2}\nmax_luma={:.2}\nproduct_debug_view=none\nruntime_trace_lighting_readback=trace_probe_tiles_compute+writes_probe_trace_lighting_buffer\nruntime_trace_lighting_provider_history=completion_probe_trace_lighting_rgb_to_hybrid_gi_runtime_state\nruntime_trace_lighting_neutral_sideband=provider_resolve_runtime_probe_rt_lighting_rgb_to_render_hybrid_gi_prepared_frame\nruntime_trace_lighting_collector_rebuild=render_hybrid_gi_prepared_probe_rt_lighting_to_hybrid_gi_resolve_runtime\nruntime_trace_lighting_public_path=render_framework_prepare_runtime_submission_to_runtime_prepare_collector\nvalidated_provider_trace_lighting_sideband=provider_projects_probe_rt_lighting_history_into_neutral_prepared_frame_sideband\nvalidated_collector_trace_lighting_rebuild=neutral_prepared_frame_projects_to_gpu_prepare_inputs\nfirst_hybrid_gi_probe_trace_tile_count={}\nsecond_hybrid_gi_cache_entry_count={}\nlast_hybrid_gi_graph_executed_pass_count={}\nlast_hybrid_gi_cache_entry_count={}\nlast_hybrid_gi_probe_trace_tile_count={}\nlast_hybrid_gi_probe_trace_dispatch_group_count={:?}\nlast_hybrid_gi_scene_screen_probe_count={}\nlast_hybrid_gi_scene_radiance_cache_entry_count={}\n",
            RUNTIME_TRACE_LIGHTING_PRODUCT_WGPU_PNG,
            frame.width,
            frame.height,
            frame.generation,
            metrics.visible_pixels,
            metrics.min_luma,
            metrics.max_luma,
            first_stats.last_hybrid_gi_probe_trace_tile_count,
            second_stats.last_hybrid_gi_cache_entry_count,
            stats.last_hybrid_gi_graph_executed_pass_count,
            stats.last_hybrid_gi_cache_entry_count,
            stats.last_hybrid_gi_probe_trace_tile_count,
            stats.last_hybrid_gi_probe_trace_dispatch_group_count,
            stats.last_hybrid_gi_scene_screen_probe_count,
            stats.last_hybrid_gi_scene_radiance_cache_entry_count,
        ),
    )
    .unwrap();
    assert!(
        metrics.visible_pixels > 0 && metrics.max_luma > 8.0,
        "expected nonblack Wgpu product frame; metrics={metrics:?}"
    );
}

#[test]
#[ignore]
fn export_hybrid_gi_product_composite_spatial_radiance_wgpu_png() {
    let (asset_manager, root, smooth_white, rough_white, _, _) =
        material_surface_response_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let warm_extract = scene_representation_extract_with_debug_view_and_key_light(
        viewport_size,
        model.clone(),
        smooth_white.clone(),
        rough_white.clone(),
        RenderHybridGiDebugView::None,
        Vec3::new(1.0, 0.06, 0.03),
        false,
    );
    let cool_extract = scene_representation_extract_with_debug_view_and_key_light(
        viewport_size,
        model,
        smooth_white,
        rough_white,
        RenderHybridGiDebugView::None,
        Vec3::new(0.03, 0.08, 1.0),
        false,
    );
    let warm_server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager.clone());
    let warm_viewport = warm_server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    warm_server
        .set_quality_profile(warm_viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    warm_server
        .submit_frame_extract(warm_viewport, warm_extract.clone())
        .unwrap();
    warm_server
        .submit_frame_extract(warm_viewport, warm_extract.clone())
        .unwrap();
    warm_server
        .submit_frame_extract(warm_viewport, warm_extract)
        .unwrap();
    let warm_stats = warm_server.query_stats().unwrap();
    let warm_frame = warm_server
        .capture_frame(warm_viewport)
        .unwrap()
        .expect("warm Wgpu product frame capture should be available");

    let cool_server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let cool_viewport = cool_server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    cool_server
        .set_quality_profile(cool_viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    cool_server
        .submit_frame_extract(cool_viewport, cool_extract.clone())
        .unwrap();
    cool_server
        .submit_frame_extract(cool_viewport, cool_extract.clone())
        .unwrap();
    cool_server
        .submit_frame_extract(cool_viewport, cool_extract)
        .unwrap();
    let cool_stats = cool_server.query_stats().unwrap();
    let cool_frame = cool_server
        .capture_frame(cool_viewport)
        .unwrap()
        .expect("cool Wgpu product frame capture should be available");

    assert_eq!(warm_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(cool_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert!(warm_stats.last_hybrid_gi_cache_entry_count >= 1);
    assert!(cool_stats.last_hybrid_gi_cache_entry_count >= 1);
    assert!(warm_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(cool_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(warm_stats.last_hybrid_gi_scene_screen_probe_count >= 2);
    assert!(cool_stats.last_hybrid_gi_scene_screen_probe_count >= 2);

    let warm_metrics = frame_metrics(&warm_frame);
    let cool_metrics = frame_metrics(&cool_frame);
    let warm_red =
        average_region_channel(&warm_frame.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red =
        average_region_channel(&cool_frame.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue =
        average_region_channel(&warm_frame.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue =
        average_region_channel(&cool_frame.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_metrics.visible_pixels > 0 && cool_metrics.visible_pixels > 0,
        "expected nonblank Wgpu product frames; warm={warm_metrics:?}, cool={cool_metrics:?}"
    );
    assert!(
        warm_red > cool_red + 0.25,
        "expected warm scene direct-light seed to survive HGI product composite with preview direct lighting disabled; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.25,
        "expected cool scene direct-light seed to survive HGI product composite with preview direct lighting disabled; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_side_by_side_png(
        output_dir.join(PRODUCT_COMPOSITE_SPATIAL_RADIANCE_WGPU_PNG),
        &warm_frame,
        &cool_frame,
    );
    fs::write(
        output_dir.join(PRODUCT_COMPOSITE_SPATIAL_RADIANCE_WGPU_REPORT),
        format!(
            "png={}\nleft=warm_spatial_radiance\nright=cool_spatial_radiance\nwidth={}\nheight={}\nwarm_generation={}\ncool_generation={}\nwarm_visible_pixels={}\ncool_visible_pixels={}\nwarm_min_luma={:.2}\nwarm_max_luma={:.2}\ncool_min_luma={:.2}\ncool_max_luma={:.2}\nwarm_center_red={:.2}\ncool_center_red={:.2}\nwarm_minus_cool_red={:.2}\nwarm_center_blue={:.2}\ncool_center_blue={:.2}\ncool_minus_warm_blue={:.2}\nproduct_debug_view=none\nproduct_preview_direct_lighting=disabled_for_product_gi_isolation\ngpu_probe_trace_tile_radiance=trace_probe_tiles_compute_preserves_spatially_lit_surface_cache_radiance\ncompletion_scene_light_seed_scope=synthetic_legacy_fallback_only\nproduct_composite_source=scene_prepare_spatial_direct_radiance_to_surface_cache_trace_to_global_illumination\nlumen_reference=CompositeTraces_ScreenProbeRadianceCurrentFrame_to_FinalCompose_DiffuseIndirect\nwarm_hybrid_gi_graph_executed_pass_count={}\ncool_hybrid_gi_graph_executed_pass_count={}\nwarm_hybrid_gi_cache_entry_count={}\ncool_hybrid_gi_cache_entry_count={}\nwarm_hybrid_gi_probe_trace_tile_count={}\ncool_hybrid_gi_probe_trace_tile_count={}\nwarm_hybrid_gi_scene_screen_probe_count={}\ncool_hybrid_gi_scene_screen_probe_count={}\nwarm_hybrid_gi_scene_radiance_cache_entry_count={}\ncool_hybrid_gi_scene_radiance_cache_entry_count={}\nwarm_hybrid_gi_surface_cache_resident_page_count={}\ncool_hybrid_gi_surface_cache_resident_page_count={}\nwarm_hybrid_gi_voxel_resident_clipmap_count={}\ncool_hybrid_gi_voxel_resident_clipmap_count={}\n",
            PRODUCT_COMPOSITE_SPATIAL_RADIANCE_WGPU_PNG,
            warm_frame.width + 1 + cool_frame.width,
            warm_frame.height,
            warm_frame.generation,
            cool_frame.generation,
            warm_metrics.visible_pixels,
            cool_metrics.visible_pixels,
            warm_metrics.min_luma,
            warm_metrics.max_luma,
            cool_metrics.min_luma,
            cool_metrics.max_luma,
            warm_red,
            cool_red,
            warm_red - cool_red,
            warm_blue,
            cool_blue,
            cool_blue - warm_blue,
            warm_stats.last_hybrid_gi_graph_executed_pass_count,
            cool_stats.last_hybrid_gi_graph_executed_pass_count,
            warm_stats.last_hybrid_gi_cache_entry_count,
            cool_stats.last_hybrid_gi_cache_entry_count,
            warm_stats.last_hybrid_gi_probe_trace_tile_count,
            cool_stats.last_hybrid_gi_probe_trace_tile_count,
            warm_stats.last_hybrid_gi_scene_screen_probe_count,
            cool_stats.last_hybrid_gi_scene_screen_probe_count,
            warm_stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            cool_stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            warm_stats.last_hybrid_gi_surface_cache_resident_page_count,
            cool_stats.last_hybrid_gi_surface_cache_resident_page_count,
            warm_stats.last_hybrid_gi_voxel_resident_clipmap_count,
            cool_stats.last_hybrid_gi_voxel_resident_clipmap_count,
        ),
    )
    .unwrap();
    write_side_by_side_png(
        output_dir.join(CURRENT_FRAME_POST_UBER_WGPU_PNG),
        &warm_frame,
        &cool_frame,
    );
    fs::write(
        output_dir.join(CURRENT_FRAME_POST_UBER_WGPU_REPORT),
        format!(
            "png={}\nleft=warm_current_frame_hgi\nright=cool_current_frame_hgi\nwidth={}\nheight={}\nwarm_generation={}\ncool_generation={}\nwarm_visible_pixels={}\ncool_visible_pixels={}\nwarm_min_luma={:.2}\nwarm_max_luma={:.2}\ncool_min_luma={:.2}\ncool_max_luma={:.2}\nwarm_center_red={:.2}\ncool_center_red={:.2}\nwarm_minus_cool_red={:.2}\nwarm_center_blue={:.2}\ncool_center_blue={:.2}\ncool_minus_warm_blue={:.2}\ncurrent_frame_post_uber_input=hybrid-gi-lighting_graph_resource\ncurrent_frame_post_uber_binding=post_uber_history_global_illumination_slot_reused_for_current_frame_when_available\ncurrent_frame_fallback=history-global-illumination_when_hybrid_gi_lighting_missing\nrender_graph_route=hybrid-gi-resolve_write_texture_hybrid-gi-lighting_to_post.uber_read_texture\nstack_activation=PostProcessStackDescriptor_with_hybrid_gi_lighting_input\nshader_branch=params.hybrid_gi_counts.w_current_frame_source\nlumen_reference=CompositeTraces_ScreenProbeRadianceCurrentFrame_to_FinalCompose_DiffuseIndirect\nwarm_hybrid_gi_graph_executed_pass_count={}\ncool_hybrid_gi_graph_executed_pass_count={}\nwarm_hybrid_gi_cache_entry_count={}\ncool_hybrid_gi_cache_entry_count={}\nwarm_hybrid_gi_probe_trace_tile_count={}\ncool_hybrid_gi_probe_trace_tile_count={}\nwarm_hybrid_gi_scene_screen_probe_count={}\ncool_hybrid_gi_scene_screen_probe_count={}\nwarm_hybrid_gi_scene_radiance_cache_entry_count={}\ncool_hybrid_gi_scene_radiance_cache_entry_count={}\nwarm_hybrid_gi_surface_cache_resident_page_count={}\ncool_hybrid_gi_surface_cache_resident_page_count={}\nwarm_hybrid_gi_voxel_resident_clipmap_count={}\ncool_hybrid_gi_voxel_resident_clipmap_count={}\n",
            CURRENT_FRAME_POST_UBER_WGPU_PNG,
            warm_frame.width + 1 + cool_frame.width,
            warm_frame.height,
            warm_frame.generation,
            cool_frame.generation,
            warm_metrics.visible_pixels,
            cool_metrics.visible_pixels,
            warm_metrics.min_luma,
            warm_metrics.max_luma,
            cool_metrics.min_luma,
            cool_metrics.max_luma,
            warm_red,
            cool_red,
            warm_red - cool_red,
            warm_blue,
            cool_blue,
            cool_blue - warm_blue,
            warm_stats.last_hybrid_gi_graph_executed_pass_count,
            cool_stats.last_hybrid_gi_graph_executed_pass_count,
            warm_stats.last_hybrid_gi_cache_entry_count,
            cool_stats.last_hybrid_gi_cache_entry_count,
            warm_stats.last_hybrid_gi_probe_trace_tile_count,
            cool_stats.last_hybrid_gi_probe_trace_tile_count,
            warm_stats.last_hybrid_gi_scene_screen_probe_count,
            cool_stats.last_hybrid_gi_scene_screen_probe_count,
            warm_stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            cool_stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            warm_stats.last_hybrid_gi_surface_cache_resident_page_count,
            cool_stats.last_hybrid_gi_surface_cache_resident_page_count,
            warm_stats.last_hybrid_gi_voxel_resident_clipmap_count,
            cool_stats.last_hybrid_gi_voxel_resident_clipmap_count,
        ),
    )
    .unwrap();
}

#[test]
#[ignore]
fn export_hybrid_gi_scene_depth_source_sampling_wgpu_png() {
    let (asset_manager, root, smooth_white, rough_white, _, _) =
        material_surface_response_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let near_extract = scene_representation_extract_with_card_positions(
        viewport_size,
        model.clone(),
        smooth_white.clone(),
        rough_white.clone(),
        RenderHybridGiDebugView::None,
        Vec3::ONE,
        false,
        Vec3::new(-1.0, 0.0, -24.0),
        Vec3::new(3.0, 0.0, 0.0),
    );
    let far_extract = scene_representation_extract_with_card_positions(
        viewport_size,
        model,
        smooth_white,
        rough_white,
        RenderHybridGiDebugView::None,
        Vec3::ONE,
        false,
        Vec3::new(-1.0, 0.0, 24.0),
        Vec3::new(3.0, 0.0, 0.0),
    );

    let near_server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager.clone());
    let near_viewport = near_server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    near_server
        .set_quality_profile(near_viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    near_server
        .submit_frame_extract(near_viewport, near_extract.clone())
        .unwrap();
    let near_first_stats = near_server.query_stats().unwrap();
    near_server
        .submit_frame_extract(near_viewport, near_extract.clone())
        .unwrap();
    near_server
        .submit_frame_extract(near_viewport, near_extract)
        .unwrap();
    let near_stats = near_server.query_stats().unwrap();
    let near_frame = near_server
        .capture_frame(near_viewport)
        .unwrap()
        .expect("near-depth Wgpu product frame capture should be available");

    let far_server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let far_viewport = far_server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    far_server
        .set_quality_profile(far_viewport, hybrid_gi_only_quality_profile())
        .unwrap();
    far_server
        .submit_frame_extract(far_viewport, far_extract.clone())
        .unwrap();
    let far_first_stats = far_server.query_stats().unwrap();
    far_server
        .submit_frame_extract(far_viewport, far_extract.clone())
        .unwrap();
    far_server
        .submit_frame_extract(far_viewport, far_extract)
        .unwrap();
    let far_stats = far_server.query_stats().unwrap();
    let far_frame = far_server
        .capture_frame(far_viewport)
        .unwrap()
        .expect("far-depth Wgpu product frame capture should be available");

    assert!(near_first_stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1);
    assert!(far_first_stats.last_hybrid_gi_surface_cache_depth_sample_count >= 1);
    assert!(near_stats.last_hybrid_gi_probe_trace_tile_count >= 1);
    assert!(far_stats.last_hybrid_gi_probe_trace_tile_count >= 1);

    let near_metrics = frame_metrics(&near_frame);
    let far_metrics = frame_metrics(&far_frame);
    assert!(
        near_metrics.visible_pixels > 0 && far_metrics.visible_pixels > 0,
        "expected nonblank depth-source Wgpu product frames; near={near_metrics:?}, far={far_metrics:?}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_side_by_side_png(
        output_dir.join(SCENE_DEPTH_SOURCE_SAMPLING_WGPU_PNG),
        &near_frame,
        &far_frame,
    );
    fs::write(
        output_dir.join(SCENE_DEPTH_SOURCE_SAMPLING_WGPU_REPORT),
        format!(
            "png={}\nleft=near_scene_depth_source\nright=far_scene_depth_source\nwidth={}\nheight={}\nnear_generation={}\nfar_generation={}\nnear_visible_pixels={}\nfar_visible_pixels={}\nnear_min_luma={:.2}\nnear_max_luma={:.2}\nfar_min_luma={:.2}\nfar_max_luma={:.2}\nscene_depth_source_sampling=collect_inputs_scene_prepare_card_bounds_to_surface_cache_depth_source_rgba\nscene_depth_source_precedence=surface_cache_depth_source_samples_preferred_over_bounds_fallback\nwgpu_depth_upload=scene_prepare_surface_cache_depth_texture_upload_and_readback\ntrace_depth_consumer=trace_probe_tiles_compute_surface_cache_depth_texture_load\ndirect_dsrt_scene_depth_texture=hybrid_gi_scene_prepare_graph_executor_texture_depth_load_to_hybrid_gi_scene_buffer\nlumen_reference=ScreenProbeGather_surface_cache_trace_depth_then_composite_indirect\nnear_first_hybrid_gi_surface_cache_depth_sample_count={}\nfar_first_hybrid_gi_surface_cache_depth_sample_count={}\nnear_last_hybrid_gi_graph_executed_pass_count={}\nfar_last_hybrid_gi_graph_executed_pass_count={}\nnear_last_hybrid_gi_surface_cache_depth_sample_count={}\nfar_last_hybrid_gi_surface_cache_depth_sample_count={}\nnear_last_hybrid_gi_probe_trace_tile_count={}\nfar_last_hybrid_gi_probe_trace_tile_count={}\nnear_last_hybrid_gi_probe_trace_dispatch_group_count={:?}\nfar_last_hybrid_gi_probe_trace_dispatch_group_count={:?}\nnear_last_hybrid_gi_scene_screen_probe_count={}\nfar_last_hybrid_gi_scene_screen_probe_count={}\nnear_last_hybrid_gi_surface_cache_resident_page_count={}\nfar_last_hybrid_gi_surface_cache_resident_page_count={}\n",
            SCENE_DEPTH_SOURCE_SAMPLING_WGPU_PNG,
            near_frame.width + 1 + far_frame.width,
            near_frame.height,
            near_frame.generation,
            far_frame.generation,
            near_metrics.visible_pixels,
            far_metrics.visible_pixels,
            near_metrics.min_luma,
            near_metrics.max_luma,
            far_metrics.min_luma,
            far_metrics.max_luma,
            near_first_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            far_first_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            near_stats.last_hybrid_gi_graph_executed_pass_count,
            far_stats.last_hybrid_gi_graph_executed_pass_count,
            near_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            far_stats.last_hybrid_gi_surface_cache_depth_sample_count,
            near_stats.last_hybrid_gi_probe_trace_tile_count,
            far_stats.last_hybrid_gi_probe_trace_tile_count,
            near_stats.last_hybrid_gi_probe_trace_dispatch_group_count,
            far_stats.last_hybrid_gi_probe_trace_dispatch_group_count,
            near_stats.last_hybrid_gi_scene_screen_probe_count,
            far_stats.last_hybrid_gi_scene_screen_probe_count,
            near_stats.last_hybrid_gi_surface_cache_resident_page_count,
            far_stats.last_hybrid_gi_surface_cache_resident_page_count,
        ),
    )
    .unwrap();
}
