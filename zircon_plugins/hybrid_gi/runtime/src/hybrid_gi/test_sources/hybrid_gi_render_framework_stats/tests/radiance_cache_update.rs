use super::*;

const RADIANCE_CACHE_UPDATE_WGPU_PNG: &str =
    "plan18_hybrid_gi_radiance_cache_update_wgpu_20260810.png";
const RADIANCE_CACHE_UPDATE_WGPU_REPORT: &str =
    "plan18_hybrid_gi_radiance_cache_update_wgpu_20260810.txt";
const RADIANCE_CACHE_READBACK_EVIDENCE_FRAME_LIMIT: usize = 16;

#[test]
fn render_framework_stats_distinguish_radiance_cache_update_work_from_stable_frame_reuse() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(160, 120);
    let extract =
        scene_representation_extract(viewport_size, model, black_material, emissive_material);

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
    let update_stats = server.query_stats().unwrap();
    assert!(
        update_stats.last_hybrid_gi_radiance_cache_update_probe_count > 0,
        "the first RC generation must expose its bounded update work through public RenderStats"
    );

    server.submit_frame_extract(viewport, extract).unwrap();
    let stable_stats = server.query_stats().unwrap();
    assert_eq!(
        stable_stats.last_hybrid_gi_radiance_cache_update_probe_count, 0,
        "an unchanged frame must reuse the committed RC generation without source-snapshot update work"
    );
    assert_eq!(
        stable_stats.last_hybrid_gi_radiance_cache_generation,
        update_stats.last_hybrid_gi_radiance_cache_generation,
        "stable RC reuse must retain the committed generation"
    );
}

#[test]
#[ignore]
fn export_hybrid_gi_radiance_cache_update_wgpu_png() {
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
    let update_stats = server.query_stats().unwrap();
    assert!(
        update_stats.last_hybrid_gi_radiance_cache_update_probe_count > 0,
        "the first product frame must dispatch a bounded RC update"
    );

    let mut gpu_update_dispatch_counts = None;
    let mut stable_gpu_dispatch_counts = None;
    let mut stable_stats = None;
    let consume_stage_index = RenderHybridGiRadianceCacheGpuStage::Consume.index();
    for _ in 0..RADIANCE_CACHE_READBACK_EVIDENCE_FRAME_LIMIT {
        server
            .submit_frame_extract(viewport, extract.clone())
            .unwrap();
        let stats = server.query_stats().unwrap();
        let counts = stats.last_hybrid_gi_radiance_cache_gpu_stage_dispatch_counts;
        if gpu_update_dispatch_counts.is_none()
            && counts[..consume_stage_index].iter().all(|count| *count > 0)
            && counts[consume_stage_index] > 0
        {
            gpu_update_dispatch_counts = Some(counts);
            continue;
        }
        if gpu_update_dispatch_counts.is_some()
            && counts[..consume_stage_index]
                .iter()
                .all(|count| *count == 0)
            && counts[consume_stage_index] > 0
        {
            stable_gpu_dispatch_counts = Some(counts);
            stable_stats = Some(stats);
            break;
        }
    }
    let gpu_update_dispatch_counts = gpu_update_dispatch_counts.expect(
        "bounded readback frames must expose five GPU update counts and committed consume writes",
    );
    let stable_gpu_dispatch_counts = stable_gpu_dispatch_counts.expect(
        "a later stable readback must expose committed consume writes without retracing the RC generation",
    );
    let stable_stats = stable_stats.expect("stable GPU dispatch evidence must retain RenderStats");
    assert_eq!(
        stable_stats.last_hybrid_gi_radiance_cache_update_probe_count, 0,
        "the captured stable product frame must reuse its committed RC atlas"
    );
    assert_eq!(
        stable_stats.last_hybrid_gi_radiance_cache_generation,
        update_stats.last_hybrid_gi_radiance_cache_generation,
        "stable RC consumption must preserve the completed generation"
    );
    assert!(stable_stats.last_hybrid_gi_radiance_cache_resident_probe_count >= 8);

    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("radiance-cache Wgpu product frame capture should be available");
    let metrics = frame_metrics(&frame);
    assert!(
        metrics.visible_pixels > 0 && metrics.max_luma > 8.0,
        "expected a nonblank RC product frame; metrics={metrics:?}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_png(output_dir.join(RADIANCE_CACHE_UPDATE_WGPU_PNG), &frame);
    fs::write(
        output_dir.join(RADIANCE_CACHE_UPDATE_WGPU_REPORT),
        format!(
            "png={}\nwidth={}\nheight={}\ngeneration={}\nvisible_pixels={}\nmin_luma={:.2}\nmax_luma={:.2}\nproduct_debug_view=none\nradiance_cache_update_pipeline=mark_allocate_trace_filter_border_fixup_mip_then_screen_probe_consume\nradiance_cache_storage=buffer_atlas_32_slots_4x4_tiles_final_mip\nradiance_cache_stable_frame_reuses_final_atlas=true\nradiance_cache_consume_counter_semantics=committed_resident_probe_write_count\nfirst_radiance_cache_update_probe_count={}\nstable_radiance_cache_update_probe_count={}\nfirst_radiance_cache_generation={}\nstable_radiance_cache_generation={}\nstable_radiance_cache_resident_probe_count={}\nstable_scene_radiance_cache_entry_count={}\ngpu_update_dispatch_counts={:?}\ngpu_stable_dispatch_counts={:?}\n",
            RADIANCE_CACHE_UPDATE_WGPU_PNG,
            frame.width,
            frame.height,
            frame.generation,
            metrics.visible_pixels,
            metrics.min_luma,
            metrics.max_luma,
            update_stats.last_hybrid_gi_radiance_cache_update_probe_count,
            stable_stats.last_hybrid_gi_radiance_cache_update_probe_count,
            update_stats.last_hybrid_gi_radiance_cache_generation,
            stable_stats.last_hybrid_gi_radiance_cache_generation,
            stable_stats.last_hybrid_gi_radiance_cache_resident_probe_count,
            stable_stats.last_hybrid_gi_scene_radiance_cache_entry_count,
            gpu_update_dispatch_counts,
            stable_gpu_dispatch_counts,
        ),
    )
    .unwrap();
}
