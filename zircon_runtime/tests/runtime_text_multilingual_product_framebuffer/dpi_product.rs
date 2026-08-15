use super::*;

const DPI_SCALE: f32 = 2.0;
const DPI_TEXT: &str = "DPI native glyph raster";

#[test]
#[ignore = "renders a 1x-to-2x native cache-key transition through the real WGPU framebuffer"]
fn render_text_dpi_rerasterizes_with_distinct_native_cache_entries() {
    let logical_viewport = UVec2::new(380, 160);
    let logical_frame = UiFrame::new(24.0, 48.0, 320.0, 64.0);
    let physical_viewport = UVec2::new(
        (logical_viewport.x as f32 * DPI_SCALE) as u32,
        (logical_viewport.y as f32 * DPI_SCALE) as u32,
    );
    let physical_frame = scale_frame(logical_frame, DPI_SCALE);
    let (asset_manager, fixture_root) = product_fixture_asset_manager("dpi-fixture");
    let mut renderer = ProductUiFrameRenderer::new(physical_viewport, asset_manager);

    let (one_x, one_x_background, one_x_stats, one_x_trace) = render_native_text_frame(
        &mut renderer,
        "runtime.ui.text.dpi",
        physical_viewport,
        physical_frame,
        1.0,
    );
    let (two_x, two_x_background, two_x_stats, two_x_trace) = render_native_text_frame(
        &mut renderer,
        "runtime.ui.text.dpi",
        physical_viewport,
        physical_frame,
        DPI_SCALE,
    );

    assert_raster_capture_settled(&one_x_stats);
    assert_raster_capture_settled(&two_x_stats);
    assert!(
        one_x_trace.source_cache_miss_count > 0,
        "initial native raster must populate physical source-cache keys: {one_x_trace:?}"
    );
    assert!(
        two_x_trace.source_cache_miss_count > 0,
        "the same renderer must miss the 1x cache when DPI changes physical glyph keys: {two_x_trace:?}"
    );

    let one_x_bounds = changed_pixel_bounds_in_frame(
        &one_x.rgba,
        &one_x_background.rgba,
        one_x.width,
        one_x.height,
        physical_frame,
        10,
    )
    .expect("1x native text must produce framebuffer pixels");
    let two_x_bounds = changed_pixel_bounds_in_frame(
        &two_x.rgba,
        &two_x_background.rgba,
        two_x.width,
        two_x.height,
        physical_frame,
        10,
    )
    .expect("2x native text must produce framebuffer pixels");
    let one_x_width = one_x_bounds.2 - one_x_bounds.0 + 1;
    let one_x_height = one_x_bounds.3 - one_x_bounds.1 + 1;
    let two_x_width = two_x_bounds.2 - two_x_bounds.0 + 1;
    let two_x_height = two_x_bounds.3 - two_x_bounds.1 + 1;
    let one_x_changed = one_x_bounds.4;
    let two_x_changed = two_x_bounds.4;

    assert!(
        two_x_width >= one_x_width.saturating_mul(2).saturating_sub(4)
            && two_x_height >= one_x_height.saturating_mul(2).saturating_sub(4),
        "the same device-space frame must receive 2x native glyph geometry after the DPI transition: 1x={one_x_bounds:?}, 2x={two_x_bounds:?}"
    );
    assert!(
        two_x_changed >= one_x_changed.saturating_mul(3),
        "the 2x native cache entry must produce higher-resolution framebuffer coverage: 1x={one_x_bounds:?}, 2x={two_x_bounds:?}"
    );
    assert_ne!(
        one_x.rgba, two_x.rgba,
        "the 1x and 2x native cache entries must not produce the same framebuffer"
    );

    drop(renderer);
    let _ = std::fs::remove_dir_all(fixture_root);
}

fn render_native_text_frame(
    renderer: &mut ProductUiFrameRenderer,
    tree_id: &str,
    viewport: UVec2,
    text_frame: UiFrame,
    raster_scale: f32,
) -> (
    zircon_runtime::core::framework::render::CapturedFrame,
    zircon_runtime::core::framework::render::CapturedFrame,
    zircon_runtime::core::framework::render::RenderStats,
    UiTextRasterFrameTrace,
) {
    let background = proof_background(viewport);
    let text = proof_text(
        901,
        text_frame,
        DPI_TEXT,
        UiTextDirection::LeftToRight,
        Some("en"),
        UiTextRenderMode::Native,
    );
    let (capture, stats, trace) =
        renderer.render_ui_extract_frame_with_raster_trace(UiRenderExtract {
            tree_id: UiTreeId::new(tree_id),
            list: UiRenderList {
                commands: vec![background.clone(), text],
            },
            raster_scale,
        });
    let (background_capture, _) = renderer.render_ui_extract_frame(UiRenderExtract {
        tree_id: UiTreeId::new(format!("{tree_id}.background")),
        list: UiRenderList {
            commands: vec![background],
        },
        raster_scale,
    });
    (capture, background_capture, stats, trace)
}

fn assert_raster_capture_settled(stats: &zircon_runtime::core::framework::render::RenderStats) {
    assert_eq!(
        stats.last_ui_text_raster_worker_pending_count, 0,
        "{stats:#?}"
    );
    assert_eq!(
        stats.last_ui_text_raster_worker_failed_count, 0,
        "{stats:#?}"
    );
    assert_eq!(
        stats.last_ui_text_visible_missing_raster_image_count, 0,
        "{stats:#?}"
    );
    assert_eq!(
        stats.last_ui_text_visible_raster_placeholder_count, 0,
        "{stats:#?}"
    );
    assert_eq!(
        stats.last_ui_text_raster_renderer_upload_requeued_count, 0,
        "{stats:#?}"
    );
    assert_eq!(
        stats.last_ui_text_raster_renderer_upload_failure_count, 0,
        "{stats:#?}"
    );
    assert_eq!(
        stats.last_ui_text_sdf_generation_pending_batch_count, 0,
        "{stats:#?}"
    );
    assert_eq!(
        stats.last_ui_text_sdf_generation_completion_backlog_count, 0,
        "{stats:#?}"
    );
    assert_eq!(
        stats.last_ui_text_sdf_generation_failure_count, 0,
        "{stats:#?}"
    );
}

fn scale_frame(frame: UiFrame, scale: f32) -> UiFrame {
    UiFrame::new(
        frame.x * scale,
        frame.y * scale,
        frame.width * scale,
        frame.height * scale,
    )
}
