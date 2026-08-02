use std::path::{Path, PathBuf};

use glyphon::{Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, TextArea, TextBounds};

use crate::core::math::UVec2;
use crate::graphics::backend::{RenderBackend, read_texture_rgba};
use crate::text::atlas::render_plan::GlyphAtlasScreenRect;
use crate::text::atlas::{GlyphAtlasBitmapRetryFrameState, GlyphAtlasSet};
use crate::text::font::FontDatabase;
use crate::text::native_bitmap_atlas::{
    NativeBitmapAtlasHandoff, NativeBitmapAtlasSourceCache, NativeBitmapAtlasTextArea,
    bitmap_atlas_page_size, native_bitmap_atlas_frame, native_bitmap_atlas_handoff_for_report,
};
use crate::text::parallel::raster_pool::{TextRasterWorkerPool, TextRasterWorkerPoolOptions};

use super::{
    GlyphAtlasBitmapRenderer, PROOF_HEIGHT, PROOF_WIDTH, assert_product_proof_is_outside_target,
    changed_pixels, proof_path_for, workspace_root,
};

const NATIVE_LAYOUT_PROOF_FILE_NAME: &str =
    "runtime_text_native_bitmap_layout_product_framebuffer_20260802.png";
pub(super) const NATIVE_LAYOUT_TEXT: &str = "中文排版引擎文本与布局 中文排版引擎文本与布局";
pub(super) const NATIVE_LAYOUT_CJK_FAMILY: &str = "Zircon Noto Sans CJK SC Proof";

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
#[ignore = "exports an explicit runtime WGPU native bitmap text layout framebuffer proof"]
fn render_text_native_bitmap_layout_product_framebuffer() {
    let viewport_size = UVec2::new(PROOF_WIDTH, PROOF_HEIGHT);
    let font_source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fonts")
        .join("ZirconDefaultComposite-subset.ttc");
    let mut font_database = FontDatabase::default();
    let primary_face = font_database
        .register_font_file(&font_source, Some(NATIVE_LAYOUT_CJK_FAMILY), 1)
        .expect("register product proof font");
    let mut font_system = FontSystem::new();
    font_database
        .load_face_into_font_system(primary_face, &mut font_system)
        .expect("synchronize product proof font database");
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(24.0, 30.0));
    buffer.set_size(&mut font_system, Some(192.0), Some(132.0));
    buffer.set_text(
        &mut font_system,
        NATIVE_LAYOUT_TEXT,
        &Attrs::new().family(Family::Name(NATIVE_LAYOUT_CJK_FAMILY)),
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    let layout_glyph_count = buffer
        .layout_runs()
        .map(|run| run.glyphs.len())
        .sum::<usize>();
    assert!(layout_glyph_count > 0);
    assert!(
        buffer
            .layout_runs()
            .flat_map(|run| run.glyphs.iter())
            .all(|glyph| font_database.font_face_id(glyph.font_id) == Some(primary_face)),
        "the layout glyphs must retain the FontDatabase backend identity used for persistent raster keys"
    );

    let layout_line_rects = buffer
        .layout_runs()
        .filter(|run| !run.glyphs.is_empty())
        .map(|run| {
            GlyphAtlasScreenRect::new(32.0, 34.0 + run.line_top, 192.0, run.line_height.max(1.0))
        })
        .collect::<Vec<_>>();
    assert!(
        layout_line_rects.len() >= 2,
        "the product proof text must exercise Buffer soft wrapping before rasterization"
    );

    let text_area = TextArea {
        buffer: &buffer,
        left: 32.0,
        top: 34.0,
        scale: 1.0,
        bounds: TextBounds {
            left: 24,
            top: 26,
            right: 232,
            bottom: 180,
        },
        default_color: Color::rgba(224, 244, 255, 255),
        custom_glyphs: &[],
    };
    let bitmap_text_area = NativeBitmapAtlasTextArea::new(&text_area, None);
    let worker_pool = TextRasterWorkerPool::new_without_workers_for_test(
        TextRasterWorkerPoolOptions::new(1).with_queue_depth(128),
    );
    let mut source_cache = NativeBitmapAtlasSourceCache::with_capacity(256);
    let mut retry_state = GlyphAtlasBitmapRetryFrameState::new();

    let cold_frame = native_bitmap_atlas_frame(
        &mut font_system,
        &font_database,
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        viewport_size,
        1,
        &[bitmap_text_area],
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
        &mut font_system,
        &font_database,
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        GlyphAtlasSet::default(),
        viewport_size,
        2,
        &[bitmap_text_area],
    );
    let warm_report = warm_frame.prepare_report();
    assert!(warm_frame.replaces_glyphon());
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
    let warm_shadow_commit = renderer.prepare_submission_with_face_validity(
        &backend.device,
        &backend.queue,
        &warm_frame.submission,
        warm_frame.source_bytes(),
        bitmap_atlas_page_size(),
        warm_frame.atlas_layer_count(),
        atlas_format,
        warm_frame.face_validity(),
    );
    let warm_prepare_report = renderer.prepare_report();
    assert_eq!(
        warm_prepare_report.storage_pass_visible_glyph_count,
        warm_report.visible_raster_glyph_count
    );
    assert!(warm_prepare_report.upload_request_count > 0);
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
        &mut font_system,
        &font_database,
        Some(&worker_pool),
        &mut source_cache,
        &mut retry_state,
        committed_warm_atlas,
        viewport_size,
        3,
        &[bitmap_text_area],
    );
    let stable_report = stable_frame.prepare_report();
    assert!(stable_frame.submission.run.slot_cache_hit_count > 0);
    assert_eq!(stable_frame.submission.run.slot_cache_miss_count, 0);
    assert!(stable_frame.submission.run.upload_copies.is_empty());
    for report in [&warm_report, &stable_report] {
        assert!(report.replaces_glyphon);
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
        &backend.queue,
        &stable_frame.submission,
        stable_frame.source_bytes(),
        bitmap_atlas_page_size(),
        stable_frame.atlas_layer_count(),
        atlas_format,
        stable_frame.face_validity(),
    );
    let stable_prepare_report = renderer.prepare_report();
    assert_eq!(
        stable_prepare_report.storage_pass_visible_glyph_count,
        stable_report.visible_raster_glyph_count
    );
    assert_eq!(stable_prepare_report.upload_request_count, 0);
    assert_eq!(stable_prepare_report.upload_requeued_count, 0);
    assert_eq!(stable_prepare_report.upload_failure_count, 0);
    assert_eq!(stable_prepare_report.instance_buffer_reallocation_count, 0);
    assert_eq!(
        stable_prepare_report.instance_buffer_capacity_byte_len, warm_instance_buffer_capacity,
        "the stable layout frame must retain its warm instance buffer allocation"
    );

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
    backend.queue.submit([encoder.finish()]);
    let rgba = read_texture_rgba(&backend.device, &backend.queue, &target, viewport_size)
        .expect("read native bitmap text layout product framebuffer");
    assert_native_bitmap_text_layout_pixels(&rgba, layout_line_rects.as_slice());

    let output = native_layout_proof_path();
    assert!(output.components().all(|part| part.as_os_str() != "target"));
    let workspace_target = workspace_root().join("target");
    assert_product_proof_is_outside_target(&output, &workspace_target);
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR") {
        let target_dir = PathBuf::from(target_dir);
        let target_dir = target_dir
            .is_absolute()
            .then_some(target_dir.clone())
            .unwrap_or_else(|| workspace_root().join(target_dir));
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
            "wrapped layout line {line_index} must retain rasterized pixels at its planned location; changed={changed}"
        );
    }
}

fn native_layout_proof_path() -> PathBuf {
    proof_path_for(NATIVE_LAYOUT_PROOF_FILE_NAME)
}
