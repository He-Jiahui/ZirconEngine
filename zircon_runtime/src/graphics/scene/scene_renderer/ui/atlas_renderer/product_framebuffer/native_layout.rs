use std::path::PathBuf;

use crate::core::math::UVec2;
use crate::graphics::backend::{RenderBackend, read_texture_rgba};
use crate::graphics::scene::scene_renderer::ui::render::native_text_batches_for_product_proof;
use crate::graphics::scene::scene_renderer::ui::text::native_bitmap_atlas_glyph_runs;
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{GlyphAtlasBitmapRetryFrameState, GlyphAtlasFormat, GlyphAtlasSet};
use crate::text::font::shared_font_database_snapshot;
use crate::text::native_bitmap_atlas::{
    NativeBitmapAtlasGlyphRun, NativeBitmapAtlasHandoff, NativeBitmapAtlasSourceCache,
    bitmap_atlas_page_size, native_bitmap_atlas_frame, native_bitmap_atlas_handoff_for_report,
};
use crate::text::parallel::raster_pool::{TextRasterWorkerPool, TextRasterWorkerPoolOptions};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiTextAlign, UiTextRenderMode, UiTextWrap,
};
use zircon_runtime_interface::ui::{event_ui::UiNodeId, event_ui::UiTreeId};

use crate::ui::surface::layout_text;

use super::{
    GlyphAtlasBitmapRenderer, PROOF_HEIGHT, PROOF_WIDTH, assert_product_proof_is_outside_target,
    changed_pixels, proof_path_for, workspace_root,
};

const NATIVE_LAYOUT_PROOF_FILE_NAME: &str =
    "runtime_text_native_bitmap_layout_product_framebuffer_20260802.png";
pub(super) const NATIVE_LAYOUT_TEXT: &str = "中文排版引擎文本与布局 中文排版引擎文本与布局";
pub(super) const NATIVE_LAYOUT_CJK_FAMILY: &str = "Zircon Noto Sans CJK SC Proof";
const NATIVE_LAYOUT_ORIGIN_X: f32 = 32.0;
const NATIVE_LAYOUT_ORIGIN_Y: f32 = 34.0;
const NATIVE_LAYOUT_WIDTH: f32 = 192.0;
const NATIVE_LAYOUT_FONT_SIZE: f32 = 24.0;
const NATIVE_LAYOUT_LINE_HEIGHT: f32 = 30.0;

#[test]
fn native_bitmap_layout_product_proof_path_is_workspace_docs_not_target() {
    let workspace_root = workspace_root();
    let output = native_layout_proof_path();

    assert_eq!(
        output,
        workspace_root
            .join("docs")
            .join("tests")
            .join("runtime")
            .join("text")
            .join(NATIVE_LAYOUT_PROOF_FILE_NAME),
    );
    assert!(!output.starts_with(workspace_root.join("target")));
    assert_eq!(
        super::canonicalize_or_normalize_path(&output)
            .parent()
            .expect("canonical native layout proof path parent"),
        workspace_root
            .join("docs")
            .join("tests")
            .join("runtime")
            .join("text")
            .canonicalize()
            .expect("canonical native layout proof directory"),
    );
}

#[test]
#[should_panic(expected = "CARGO_TARGET_DIR must be an absolute coordinator path")]
fn native_bitmap_layout_product_proof_rejects_relative_cargo_target_directory() {
    let _ = super::require_absolute_cargo_target_dir(
        PathBuf::from("cargo-targets").join("native-layout-proof"),
    );
}

#[test]
fn native_bitmap_layout_product_scene_projects_actual_cjk_layout_glyphs() {
    let (glyph_runs, line_rects) = native_layout_glyph_runs(UVec2::new(PROOF_WIDTH, PROOF_HEIGHT));

    assert!(
        glyph_runs.len() > 1,
        "CJK layout must produce multiple native runs"
    );
    assert_eq!(glyph_runs.len(), line_rects.len());
    assert!(
        glyph_runs
            .iter()
            .all(|glyph_run| !glyph_run.glyphs.is_empty()),
        "every materialized CJK line must preserve a native glyph run"
    );
}

#[test]
#[ignore = "exports an explicit runtime WGPU native bitmap text layout framebuffer proof"]
fn render_text_native_bitmap_layout_product_framebuffer() {
    let viewport_size = UVec2::new(PROOF_WIDTH, PROOF_HEIGHT);
    let (glyph_runs, layout_line_rects) = native_layout_glyph_runs(viewport_size);
    let (_, font_database) = shared_font_database_snapshot();
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(128),
    );
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(256);
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();

    let cold_frame = native_bitmap_atlas_frame(
        &font_database,
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        viewport_size,
        1,
        &glyph_runs,
    );
    let cold_report = cold_frame.prepare_report();
    assert!(cold_report.missing_raster_image_count > 0);
    assert!(cold_report.source_cache.worker_request_submitted_count > 0);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&cold_report),
        NativeBitmapAtlasHandoff::TransparentPlaceholder
    );
    assert!(
        worker_pool.process_next_request_for_test(),
        "the cold text frame must enqueue real swash work"
    );
    while worker_pool.process_next_request_for_test() {}

    let warm_frame = native_bitmap_atlas_frame(
        &font_database,
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        viewport_size,
        2,
        &glyph_runs,
    );
    let warm_report = warm_frame.prepare_report();
    assert!(warm_frame.supports_native_submission());
    assert!(warm_report.visible_raster_glyph_count >= 12);
    assert!(warm_report.source_cache.worker_completion_insert_count > 0);
    assert!(warm_frame.submission.run.slot_cache_insert_count > 0);
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&warm_report),
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    );

    let backend = RenderBackend::new_offscreen().expect("headless WGPU native text backend");
    let target_format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let atlas_format = warm_frame
        .atlas_format()
        .expect("real alpha text must select one native atlas format");
    let mut renderer = GlyphAtlasBitmapRenderer::new(&backend.device, target_format);
    let mut buffer_uploads = zr_rhi_wgpu::WgpuBufferUploadBatch::new();
    let mut texture_uploads = zr_rhi_wgpu::WgpuTextureUploadBatch::new();
    let warm_shadow_commit = renderer.prepare_submission_with_face_validity(
        &backend.device,
        &warm_frame.submission,
        warm_frame.source_bytes(),
        bitmap_atlas_page_size(),
        warm_frame.atlas_layer_count(),
        atlas_format,
        warm_frame.face_validity(),
        &mut buffer_uploads,
        &mut texture_uploads,
        false,
    );
    let warm_prepare_report = renderer.prepare_report();
    assert_eq!(
        warm_prepare_report.storage_pass_visible_glyph_count,
        warm_report.visible_raster_glyph_count
    );
    assert!(warm_prepare_report.upload_request_count > 0);
    assert_eq!(warm_prepare_report.upload_plan_build_count, 1);
    assert_eq!(warm_prepare_report.upload_plan_skip_count, 0);
    assert!(warm_prepare_report.upload_ready_to_write_texture);
    assert_eq!(warm_prepare_report.upload_requeued_count, 0);
    assert_eq!(warm_prepare_report.upload_failure_count, 0);
    assert_eq!(warm_prepare_report.instance_buffer_reallocation_count, 1);
    assert!(
        warm_prepare_report.instance_buffer_capacity_byte_len
            >= warm_prepare_report.vertex_buffer_byte_len,
        "the warm layout frame must allocate enough persistent instance capacity"
    );
    let warm_instance_buffer_capacity = warm_prepare_report.instance_buffer_capacity_byte_len;

    let mut committed_warm_atlas = warm_frame.submission.run.atlas.clone();
    committed_warm_atlas.commit_bitmap_page_shadow(warm_shadow_commit);
    assert!(
        warm_frame
            .submission
            .run
            .upload_copies
            .iter()
            .all(|copy| committed_warm_atlas.has_bitmap_page_shadow(copy.page_key)),
        "accepted warm atlas uploads must persist CPU page shadows before the stable frame"
    );

    let stable_frame = native_bitmap_atlas_frame(
        &font_database,
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        committed_warm_atlas,
        viewport_size,
        3,
        &glyph_runs,
    );
    let stable_report = stable_frame.prepare_report();
    assert!(stable_frame.submission.run.slot_cache_hit_count > 0);
    assert_eq!(stable_frame.submission.run.slot_cache_miss_count, 0);
    assert!(stable_frame.submission.run.upload_copies.is_empty());
    for report in [&warm_report, &stable_report] {
        assert!(report.native_submission_ready);
        assert_eq!(report.missing_raster_image_count, 0);
        assert_eq!(report.source_cache.pending_worker_count, 0);
        assert_eq!(report.source_cache.worker_request_failed_count, 0);
        assert_eq!(report.source_cache.worker_completion_failed_count, 0);
        assert_eq!(
            report.source_cache.worker_completion_invalid_bitmap_count,
            0
        );
        assert_eq!(report.submission.visible_placeholder_count, 0);
    }
    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&stable_report),
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    );

    let _stable_shadow_commit = renderer.prepare_submission_with_face_validity(
        &backend.device,
        &stable_frame.submission,
        stable_frame.source_bytes(),
        bitmap_atlas_page_size(),
        stable_frame.atlas_layer_count(),
        atlas_format,
        stable_frame.face_validity(),
        &mut buffer_uploads,
        &mut texture_uploads,
        false,
    );
    let stable_prepare_report = renderer.prepare_report();
    assert_eq!(
        stable_prepare_report.storage_pass_visible_glyph_count,
        stable_report.visible_raster_glyph_count
    );
    assert_eq!(stable_prepare_report.upload_request_count, 0);
    assert_eq!(stable_prepare_report.upload_plan_build_count, 0);
    assert_eq!(stable_prepare_report.upload_plan_skip_count, 1);
    assert_eq!(stable_prepare_report.upload_requeued_count, 0);
    assert_eq!(stable_prepare_report.upload_failure_count, 0);
    assert_eq!(stable_prepare_report.instance_buffer_reallocation_count, 0);
    assert_eq!(
        stable_prepare_report.instance_buffer_capacity_byte_len, warm_instance_buffer_capacity,
        "the stable layout frame must retain its warm instance buffer allocation"
    );
    backend
        .enqueue_copy_resource_upload_batch(zr_rhi_wgpu::WgpuResourceUploadBatch::from_batches(
            buffer_uploads,
            texture_uploads,
        ))
        .expect("native layout resource upload batch");

    let target = backend.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-runtime-native-bitmap-layout-product-target"),
        size: wgpu::Extent3d {
            width: PROOF_WIDTH,
            height: PROOF_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-runtime-native-bitmap-layout-product-encoder"),
        });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-runtime-native-bitmap-layout-product-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.012,
                        g: 0.018,
                        b: 0.032,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        renderer.render(&mut pass);
    }
    backend
        .submit_graphics_command_buffers(vec![encoder.finish()])
        .expect("submit native bitmap layout product frame");
    let rgba = read_texture_rgba(&backend.device, &backend.queue, &target, viewport_size)
        .expect("read native bitmap text layout product framebuffer");
    assert_native_bitmap_text_layout_pixels(&rgba, layout_line_rects.as_slice());

    let output = native_layout_proof_path();
    assert!(output.components().all(|part| part.as_os_str() != "target"));
    let workspace_target = workspace_root().join("target");
    assert_product_proof_is_outside_target(&output, &workspace_target);
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR") {
        let target_dir = super::require_absolute_cargo_target_dir(PathBuf::from(target_dir));
        assert_product_proof_is_outside_target(&output, &target_dir);
    }
    std::fs::create_dir_all(output.parent().expect("text proof output parent"))
        .expect("create text proof output directory");
    image::save_buffer(
        &output,
        &rgba,
        PROOF_WIDTH,
        PROOF_HEIGHT,
        image::ColorType::Rgba8,
    )
    .expect("save native bitmap text layout product framebuffer");
    assert!(output.is_file());
    eprintln!(
        "runtime native bitmap text layout framebuffer={}",
        output.display()
    );
}

fn native_layout_glyph_runs(
    viewport_size: UVec2,
) -> (Vec<NativeBitmapAtlasGlyphRun>, Vec<GlyphAtlasScreenRect>) {
    let frame = UiFrame::new(
        NATIVE_LAYOUT_ORIGIN_X,
        NATIVE_LAYOUT_ORIGIN_Y,
        NATIVE_LAYOUT_WIDTH,
        154.0,
    );
    let clip_frame = UiFrame::new(24.0, 26.0, 208.0, 154.0);
    let style = canonical_native_layout_style();
    let layout = canonical_native_layout(&style, frame, clip_frame);
    let layout_line_count = layout.lines.len();
    assert!(
        layout_line_count > 1,
        "CJK product text must use actual line wrapping"
    );
    assert!(
        layout.rich_text_artifact.is_some(),
        "CJK product text must preserve the Text03 shaped layout artifact"
    );
    let batches = native_text_batches_for_product_proof(
        &UiRenderExtract {
            tree_id: UiTreeId::new("runtime.text.native.layout.product"),
            list: UiRenderList {
                commands: vec![UiRenderCommand {
                    node_id: UiNodeId::new(700),
                    kind: UiRenderCommandKind::Text,
                    frame,
                    clip_frame: Some(clip_frame),
                    z_index: 0,
                    style,
                    text_layout: Some(layout),
                    text: Some(NATIVE_LAYOUT_TEXT.to_string()),
                    image: None,
                    opacity: 1.0,
                }],
            },
            raster_scale: 1.0,
        },
        viewport_size,
    );
    let line_rects = batches
        .iter()
        .map(|batch| {
            GlyphAtlasScreenRect::new(
                batch.frame.x,
                batch.frame.y,
                batch.frame.width,
                batch.frame.height,
            )
        })
        .collect::<Vec<_>>();
    let projection = native_bitmap_atlas_glyph_runs(viewport_size, batches.as_slice());

    assert_eq!(batches.len(), layout_line_count);
    assert_eq!(projection.glyph_runs.len(), layout_line_count);
    assert!(
        batches
            .iter()
            .all(|batch| batch.glyph_artifact_line.is_some()),
        "CJK product proof must preserve the Text03 glyph artifact into native projection"
    );
    assert!(
        projection.font_ids.glyph_count >= 12,
        "product proof must preserve canonical CJK glyph identities"
    );
    assert_eq!(projection.font_ids.unmapped_glyph_count, 0);
    assert!(
        projection
            .glyph_runs
            .iter()
            .all(|glyph_run| !glyph_run.glyphs.is_empty()),
        "each laid-out CJK line must reach the native glyph-run projection"
    );
    (projection.glyph_runs, line_rects)
}

fn canonical_native_layout_style() -> UiResolvedStyle {
    UiResolvedStyle {
        foreground_color: Some("#e0f4ff".to_string()),
        font_family: Some(NATIVE_LAYOUT_CJK_FAMILY.to_string()),
        language: Some("zh-Hans".to_string()),
        font_size: NATIVE_LAYOUT_FONT_SIZE,
        line_height: NATIVE_LAYOUT_LINE_HEIGHT,
        text_align: UiTextAlign::Left,
        wrap: UiTextWrap::Word,
        text_render_mode: UiTextRenderMode::Native,
        ..UiResolvedStyle::default()
    }
}

fn canonical_native_layout(
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: UiFrame,
) -> zircon_runtime_interface::ui::surface::UiResolvedTextLayout {
    layout_text(NATIVE_LAYOUT_TEXT, style, frame, Some(clip_frame))
}

fn assert_native_bitmap_text_layout_pixels(rgba: &[u8], line_rects: &[GlyphAtlasScreenRect]) {
    assert_eq!(rgba.len(), (PROOF_WIDTH * PROOF_HEIGHT * 4) as usize);
    let background = [rgba[0], rgba[1], rgba[2], rgba[3]];
    let changed = changed_pixels(
        rgba,
        GlyphAtlasScreenRect::new(24.0, 26.0, 208.0, 154.0),
        background,
    );
    assert!(
        changed > 1_000,
        "real swash-rasterized text pixels must reach the WGPU target; changed={changed}"
    );
    for (line_index, line_rect) in line_rects.iter().enumerate() {
        let changed = changed_pixels(rgba, *line_rect, background);
        assert!(
            changed > 80,
            "laid-out CJK line {line_index} must retain rasterized pixels at its planned location; changed={changed}"
        );
    }
}

fn native_layout_proof_path() -> PathBuf {
    proof_path_for(NATIVE_LAYOUT_PROOF_FILE_NAME)
}
