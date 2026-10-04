use super::*;

const DEBUG_VIEW_PRODUCT_PNG: &str = "plan18_hybrid_gi_debug_views_wgpu_20260810.png";
const DEBUG_VIEW_PRODUCT_REPORT: &str = "plan18_hybrid_gi_debug_views_wgpu_20260810.txt";
const DEBUG_VIEW_WARMUP_FRAME_COUNT: usize = 3;
const MINIMUM_DEBUG_VIEW_MEAN_ABSOLUTE_DIFFERENCE: f32 = 0.5;

#[test]
#[ignore]
fn export_hybrid_gi_debug_views_wgpu_png() {
    let views = [
        RenderHybridGiDebugView::None,
        RenderHybridGiDebugView::Cards,
        RenderHybridGiDebugView::SurfaceCache,
        RenderHybridGiDebugView::VoxelClipmap,
        RenderHybridGiDebugView::InputSet,
    ];
    let frames = views.map(capture_debug_view);
    let baseline = &frames[0];
    let differences = std::array::from_fn::<_, 4, _>(|index| {
        mean_absolute_rgb_difference(baseline, &frames[index + 1])
    });

    for (view, frame) in views.into_iter().zip(frames.iter()) {
        let metrics = frame_metrics(frame);
        assert!(
            metrics.visible_pixels > 0 && metrics.max_luma > 8.0,
            "expected a nonblank {view:?} product frame; metrics={metrics:?}"
        );
    }
    for (view, difference) in views.into_iter().skip(1).zip(differences) {
        assert!(
            difference >= MINIMUM_DEBUG_VIEW_MEAN_ABSOLUTE_DIFFERENCE,
            "{view:?} must change actual product pixels relative to None; mean absolute RGB difference={difference:.3}"
        );
    }

    let output_dir = render_test_output_dir();
    fs::create_dir_all(&output_dir).unwrap();
    write_frame_strip_png(output_dir.join(DEBUG_VIEW_PRODUCT_PNG), &frames);
    fs::write(
        output_dir.join(DEBUG_VIEW_PRODUCT_REPORT),
        format!(
            "png={}\nwidth={}\nheight={}\npanel_count={}\npanel_order=none,cards,surface_cache,voxel_clipmap,input_set\nmean_absolute_rgb_difference_from_none={:?}\nminimum_required_difference={:.3}\ndebug_view_selector_source=RenderHybridGiExtract.debug_view\ndebug_view_gpu_abi=resolve_temporal_params_viewport_and_flags_w\ndebug_view_temporal_policy=current_frame_only_history_disabled\ndebug_view_product_path=HybridGiResolveTraceDepthSourcePass_to_current_hybrid_gi_lighting_to_post_uber\nlumen_reference=FinalCompose_bVisualizeDiffuseIndirect\n",
            DEBUG_VIEW_PRODUCT_PNG,
            baseline.width,
            baseline.height,
            frames.len(),
            differences,
            MINIMUM_DEBUG_VIEW_MEAN_ABSOLUTE_DIFFERENCE,
        ),
    )
    .unwrap();
}

fn capture_debug_view(debug_view: RenderHybridGiDebugView) -> CapturedFrame {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let _cleanup = TempProjectCleanup(root);
    let model = model_handle(&asset_manager);
    let viewport_size = UVec2::new(192, 128);
    let extract = scene_representation_extract_with_debug_view(
        viewport_size,
        model,
        black_material,
        emissive_material,
        debug_view,
    );
    let server = pluginized_wgpu_render_framework_with_asset_manager(asset_manager);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hybrid_gi_only_quality_profile())
        .unwrap();

    for _ in 0..DEBUG_VIEW_WARMUP_FRAME_COUNT {
        server
            .submit_frame_extract(viewport, extract.clone())
            .unwrap();
    }
    let stats = server.query_stats().unwrap();
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert!(stats.last_hybrid_gi_probe_trace_tile_count > 0);
    server
        .capture_frame(viewport)
        .unwrap()
        .expect("hybrid GI debug-view Wgpu product frame should be available")
}

fn mean_absolute_rgb_difference(left: &CapturedFrame, right: &CapturedFrame) -> f32 {
    assert_eq!((left.width, left.height), (right.width, right.height));
    assert_eq!(left.rgba.len(), right.rgba.len());
    let mut difference = 0_u64;
    let mut channel_count = 0_u64;
    for (left_pixel, right_pixel) in left.rgba.chunks_exact(4).zip(right.rgba.chunks_exact(4)) {
        for channel in 0..3 {
            difference += u64::from(left_pixel[channel].abs_diff(right_pixel[channel]));
            channel_count += 1;
        }
    }
    difference as f32 / channel_count.max(1) as f32
}

fn write_frame_strip_png(path: PathBuf, frames: &[CapturedFrame]) {
    let first = frames
        .first()
        .expect("debug-view strip has at least one frame");
    assert!(frames.iter().all(|frame| {
        frame.width == first.width
            && frame.height == first.height
            && frame.rgba.len() == (frame.width * frame.height * 4) as usize
    }));
    let separator_width = frames.len().saturating_sub(1) as u32;
    let output_width = first.width * frames.len() as u32 + separator_width;
    let mut rgba = vec![0_u8; (output_width * first.height * 4) as usize];

    for y in 0..first.height as usize {
        let output_row = y * output_width as usize * 4;
        let source_row = y * first.width as usize * 4;
        let source_len = first.width as usize * 4;
        for (frame_index, frame) in frames.iter().enumerate() {
            let target_start = output_row + frame_index * (source_len + 4);
            rgba[target_start..target_start + source_len]
                .copy_from_slice(&frame.rgba[source_row..source_row + source_len]);
            if frame_index + 1 < frames.len() {
                rgba[target_start + source_len..target_start + source_len + 4]
                    .copy_from_slice(&[255, 255, 255, 255]);
            }
        }
    }

    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(output_width, first.height, rgba)
        .expect("debug-view strip rgba payload should match its dimensions");
    image.save_with_format(path, ImageFormat::Png).unwrap();
}
