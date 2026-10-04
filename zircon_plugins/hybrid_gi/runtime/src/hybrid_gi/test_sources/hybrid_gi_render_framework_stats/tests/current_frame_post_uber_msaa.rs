use super::*;
use zircon_runtime::core::framework::render::AntiAliasSettings;

#[test]
#[ignore]
fn export_hybrid_gi_current_frame_post_uber_msaa_wgpu_png() {
    let (asset_manager, root, smooth_white, rough_white, _, _) =
        material_surface_response_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let mut warm_extract = scene_representation_extract_with_debug_view_and_key_light(
        viewport_size,
        model.clone(),
        smooth_white.clone(),
        rough_white.clone(),
        RenderHybridGiDebugView::None,
        Vec3::new(1.0, 0.06, 0.03),
        false,
    );
    let mut cool_extract = scene_representation_extract_with_debug_view_and_key_light(
        viewport_size,
        model,
        smooth_white,
        rough_white,
        RenderHybridGiDebugView::None,
        Vec3::new(0.03, 0.08, 1.0),
        false,
    );
    enable_4x_msaa(&mut warm_extract);
    enable_4x_msaa(&mut cool_extract);

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
        .expect("warm MSAA Wgpu product frame capture should be available");

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
        .expect("cool MSAA Wgpu product frame capture should be available");

    assert_eq!(warm_stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(cool_stats.last_hybrid_gi_graph_executed_pass_count, 4);
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
        "expected nonblank MSAA Wgpu product frames; warm={warm_metrics:?}, cool={cool_metrics:?}"
    );
    assert!(
        warm_red > cool_red + 0.25,
        "expected warm MSAA HGI product composite to survive current-frame post.uber input; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.25,
        "expected cool MSAA HGI product composite to survive current-frame post.uber input; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_side_by_side_png(
        output_dir.join(CURRENT_FRAME_POST_UBER_MSAA_WGPU_PNG),
        &warm_frame,
        &cool_frame,
    );
    fs::write(
        output_dir.join(CURRENT_FRAME_POST_UBER_MSAA_WGPU_REPORT),
        format!(
            "png={}\nleft=warm_current_frame_hgi_msaa\nright=cool_current_frame_hgi_msaa\nwidth={}\nheight={}\nwarm_generation={}\ncool_generation={}\nwarm_visible_pixels={}\ncool_visible_pixels={}\nwarm_min_luma={:.2}\nwarm_max_luma={:.2}\ncool_min_luma={:.2}\ncool_max_luma={:.2}\nwarm_center_red={:.2}\ncool_center_red={:.2}\nwarm_minus_cool_red={:.2}\nwarm_center_blue={:.2}\ncool_center_blue={:.2}\ncool_minus_warm_blue={:.2}\ngraph_msaa_sample_count=4\nhybrid_gi_lighting_sample_count=1\ncurrent_frame_post_uber_input=hybrid-gi-lighting_single_sample_graph_product\ncurrent_frame_post_uber_binding=post_uber_history_global_illumination_slot_reused_for_current_frame_when_available\ncurrent_frame_fallback=history-global-illumination_when_hybrid_gi_lighting_missing\nrender_graph_route=hybrid-gi-resolve_write_texture_hybrid-gi-lighting_to_post.uber_read_texture\nstack_activation=PostProcessStackDescriptor_with_hybrid_gi_lighting_input_for_msaa_graph\nshader_branch=params.hybrid_gi_counts.w_current_frame_source\nmsaa_contract=scene_color_and_depth_graph_msaa_hgi_lighting_single_sample_current_frame_composite\nlumen_reference=CompositeTraces_ScreenProbeRadianceCurrentFrame_to_FinalCompose_DiffuseIndirect\nwarm_hybrid_gi_graph_executed_pass_count={}\ncool_hybrid_gi_graph_executed_pass_count={}\nwarm_hybrid_gi_cache_entry_count={}\ncool_hybrid_gi_cache_entry_count={}\nwarm_hybrid_gi_probe_trace_tile_count={}\ncool_hybrid_gi_probe_trace_tile_count={}\nwarm_hybrid_gi_scene_screen_probe_count={}\ncool_hybrid_gi_scene_screen_probe_count={}\nwarm_hybrid_gi_scene_radiance_cache_entry_count={}\ncool_hybrid_gi_scene_radiance_cache_entry_count={}\nwarm_hybrid_gi_surface_cache_resident_page_count={}\ncool_hybrid_gi_surface_cache_resident_page_count={}\nwarm_hybrid_gi_voxel_resident_clipmap_count={}\ncool_hybrid_gi_voxel_resident_clipmap_count={}\n",
            CURRENT_FRAME_POST_UBER_MSAA_WGPU_PNG,
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

fn enable_4x_msaa(extract: &mut RenderFrameExtract) {
    extract.view.camera.msaa_samples = 4;
    extract.view.anti_alias = AntiAliasSettings::msaa(4);
    for camera in &mut extract.view.cameras {
        camera.camera.msaa_samples = 4;
    }
}
