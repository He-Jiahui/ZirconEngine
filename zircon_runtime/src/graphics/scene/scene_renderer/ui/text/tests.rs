use super::super::sdf_atlas::SdfAtlasDirtyPageReport;
use super::super::sdf_upload::SdfAtlasUploadPageReport;
use super::font_assets::{effective_text_render_mode, ensure_font_asset_record};
use super::resolved_batches::ResolvedScreenSpaceUiTextBatches;
use super::*;
use crate::text::sdf::SdfAtlasRect;
use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextWritingMode};

#[cfg(target_os = "windows")]
#[test]
fn screen_space_ui_font_initialization_discovers_system_faces_from_empty_snapshot() {
    let mut font_system = FontSystem::new();
    let mut font_database = FontDatabase::with_default_fallbacks();

    let discovered = initialize_screen_space_ui_font_system(&mut font_system, &mut font_database);

    assert!(discovered > 0);
    assert!(font_database
        .match_face(&crate::text::FontQuery::single_family("Segoe UI"))
        .is_some());
    assert!(font_system.db().faces().next().is_some());
}

#[test]
fn text_backend_routing_keeps_explicit_native_out_of_sdf_atlas_batches() {
    let native = text_batch("Normal", UiTextRenderMode::Native);
    let sdf = text_batch("Signed", UiTextRenderMode::Sdf);

    let routed = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&[native], &[sdf]);

    assert_eq!(routed.native_texts().len(), 1);
    assert_eq!(routed.native_texts()[0].text, "Normal");
    assert_eq!(routed.sdf_texts().len(), 1);
    assert_eq!(routed.sdf_texts()[0].text, "Signed");
}

#[test]
fn text_backend_routing_respects_auto_font_mode_without_crossing_backends() {
    let mut routed = ResolvedScreenSpaceUiTextBatches::default();

    routed.push_resolved_auto_text(
        text_batch("NormalAuto", UiTextRenderMode::Auto),
        UiTextRenderMode::Native,
    );
    routed.push_resolved_auto_text(
        text_batch("SdfAuto", UiTextRenderMode::Auto),
        UiTextRenderMode::Sdf,
    );

    assert_eq!(routed.native_texts().len(), 1);
    assert_eq!(routed.native_texts()[0].text, "NormalAuto");
    assert_eq!(routed.sdf_texts().len(), 1);
    assert_eq!(routed.sdf_texts()[0].text, "SdfAuto");
}

#[test]
fn text_font_asset_load_failure_does_not_create_a_negative_cache_record() {
    let mut text_state = TextRenderState::new(NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT);
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    let missing = "res://fonts/late-project-font.font.toml";

    for _ in 0..2 {
        let ensured =
            ensure_font_asset_record(&mut text_state, &mut font_assets, &asset_manager, missing);
        assert!(ensured.record.is_none());
        assert!(!ensured.loaded);
        assert!(!ensured.faces_changed);
    }
    assert!(font_assets.is_empty());
}

#[test]
fn text_font_refresh_recomputes_internal_vertical_advances() {
    let mut text = text_batch("AB", UiTextRenderMode::Sdf);
    text.writing_mode = UiTextWritingMode::VerticalRl;
    text.glyph_advances = vec![999.0, 999.0];

    super::super::render::text_advances::refresh_screen_space_text_batch_glyphs(&mut text);

    assert_eq!(text.glyph_advances.len(), 2);
    assert!(text.glyph_advances.iter().all(|advance| *advance < 999.0));
}

#[test]
fn text_font_refresh_preserves_resolved_layout_vertical_advances() {
    let mut text = text_batch("AB", UiTextRenderMode::Sdf);
    text.writing_mode = UiTextWritingMode::VerticalRl;
    text.source_range = Some(UiTextRange { start: 0, end: 2 });
    text.glyph_advances = vec![11.0, 13.0];

    super::super::render::text_advances::refresh_screen_space_text_batch_glyphs(&mut text);

    assert_eq!(text.glyph_advances, vec![11.0, 13.0]);
}

#[test]
fn text_prepare_report_summarizes_input_routing_and_sdf_reports() {
    let auto = [text_batch("Auto", UiTextRenderMode::Auto)];
    let native = [text_batch("Native", UiTextRenderMode::Native)];
    let sdf = [text_batch("Sdf", UiTextRenderMode::Sdf)];
    let mut resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&native, &sdf);
    resolved.push_resolved_auto_text(auto[0].clone(), UiTextRenderMode::Sdf);
    let atlas_report = SdfAtlasCacheReport {
        previous_slot_count: 1,
        current_slot_count: 2,
        retained_slot_count: 1,
        stable_slot_count: 1,
        relocated_slot_count: 0,
        added_slot_count: 1,
        evicted_slot_count: 0,
        atlas_resized: false,
        dirty_rect: Some(SdfAtlasRect {
            x: 64,
            y: 0,
            width: 64,
            height: 64,
        }),
        dirty_pages: vec![SdfAtlasDirtyPageReport {
            page_key: crate::text::atlas::GlyphAtlasPageKey::new(
                crate::text::atlas::GlyphAtlasFormat::Sdf,
                0,
            ),
            dirty_rect: SdfAtlasRect {
                x: 64,
                y: 0,
                width: 64,
                height: 64,
            },
        }],
    };
    let sdf_report = ScreenSpaceUiSdfPrepareReport {
        text_batch_count: 2,
        atlas_slot_count: 2,
        atlas_size: crate::core::math::UVec2::splat(512),
        atlas_page_count: 1,
        msdf_atlas_page_count: 1,
        atlas_allocation_failure_count: 0,
        atlas_page_limit_failure_count: 0,
        atlas_oversized_failure_count: 0,
        atlas_resized: false,
        bake: Default::default(),
        atlas_upload_byte_len: 512 * 512,
        atlas_upload_full_texture: true,
        atlas_upload: SdfAtlasUploadReport {
            mode: SdfAtlasUploadMode::FullTexture,
            byte_len: 512 * 512,
            full_texture: true,
            dirty_slot_count: 1,
            dirty_rect: Some(SdfAtlasRect {
                x: 64,
                y: 0,
                width: 64,
                height: 64,
            }),
            dirty_byte_len: 4096,
            dirty_pages: vec![SdfAtlasUploadPageReport {
                page_key: crate::text::atlas::GlyphAtlasPageKey::new(
                    crate::text::atlas::GlyphAtlasFormat::Sdf,
                    0,
                ),
                dirty_rect: SdfAtlasRect {
                    x: 64,
                    y: 0,
                    width: 64,
                    height: 64,
                },
                byte_len: 4096,
            }],
        },
        vertex_count: 12,
        decoration_vertex_count: 0,
        material_count: 2,
        draw_count: 2,
        outline_batch_count: 0,
        shadow_batch_count: 0,
        glow_batch_count: 0,
    };

    let report = text_prepare_report(
        &auto,
        &native,
        &sdf,
        &resolved,
        ScreenSpaceUiTextSdfFallbackReport::default(),
        ScreenSpaceUiNativePrepareReport {
            font_faces_changed: false,
            font_ids: ScreenSpaceUiTextFontIdReport::default(),
            bitmap_atlas: NativeBitmapAtlasPrepareReport {
                frame_index: 0,
                visible_raster_glyph_count: 2,
                source_image_count: 1,
                missing_raster_image_count: 0,
                approximate_raster_image_count: 0,
                unsupported_glyph_count: 1,
                clipped_glyph_count: 0,
                atlas_storage_format: Some(GlyphAtlasStorageFormat::R8Unorm),
                mixed_atlas_storage_format: false,
                storage_submission_count: 1,
                storage_submission_visible_glyph_count: 1,
                mixed_storage_replacement_ready: false,
                requires_background_composite: false,
                background_composite_replacement_ready: false,
                background_composite_glyph_count: 0,
                missing_background_composite_glyph_count: 0,
                source_cache: Default::default(),
                retry_submission: Default::default(),
                retry_state: Default::default(),
                discarded_stale_retry_glyph_count: 0,
                glyphon_fallback_reason: Some(
                    native_bitmap_atlas::NativeBitmapAtlasGlyphonFallbackReason::UnsupportedGlyphFormat,
                ),
                first_frame_degradation: None,
                replaces_glyphon: false,
                submission: Default::default(),
            },
        },
        MissingGlyphDiagnosticsReport::default(),
        GlyphAtlasBitmapRendererPrepareReport::default(),
        atlas_report.clone(),
        sdf_report.clone(),
    );

    assert_eq!(
        report,
        ScreenSpaceUiTextPrepareReport {
            input_auto_text_batch_count: 1,
            input_native_text_batch_count: 1,
            input_sdf_text_batch_count: 1,
            resolved_native_text_batch_count: 1,
            resolved_sdf_text_batch_count: 2,
            sdf_fallback: ScreenSpaceUiTextSdfFallbackReport::default(),
            native_font_ids: ScreenSpaceUiTextFontIdReport::default(),
            missing_glyphs: MissingGlyphDiagnosticsReport::default(),
            layout_fallbacks: crate::text::TextLayoutFallbackReport::default(),
            raster_upload: ScreenSpaceUiTextRasterUploadReport {
                visible_raster_glyph_count: 2,
                source_image_count: 1,
                ..ScreenSpaceUiTextRasterUploadReport::default()
            },
            native_bitmap_atlas: NativeBitmapAtlasPrepareReport {
                frame_index: 0,
                visible_raster_glyph_count: 2,
                source_image_count: 1,
                missing_raster_image_count: 0,
                approximate_raster_image_count: 0,
                unsupported_glyph_count: 1,
                clipped_glyph_count: 0,
                atlas_storage_format: Some(GlyphAtlasStorageFormat::R8Unorm),
                mixed_atlas_storage_format: false,
                storage_submission_count: 1,
                storage_submission_visible_glyph_count: 1,
                mixed_storage_replacement_ready: false,
                requires_background_composite: false,
                background_composite_replacement_ready: false,
                background_composite_glyph_count: 0,
                missing_background_composite_glyph_count: 0,
                source_cache: Default::default(),
                retry_submission: Default::default(),
                retry_state: Default::default(),
                discarded_stale_retry_glyph_count: 0,
                glyphon_fallback_reason: Some(
                    native_bitmap_atlas::NativeBitmapAtlasGlyphonFallbackReason::UnsupportedGlyphFormat,
                ),
                first_frame_degradation: None,
                replaces_glyphon: false,
                submission: Default::default(),
            },
            bitmap_atlas_renderer: GlyphAtlasBitmapRendererPrepareReport::default(),
            sdf_atlas: atlas_report,
            sdf_renderer: sdf_report,
        }
    );
}

#[test]
fn text_prepare_report_exposes_raster_upload_scroll_counters() {
    let native = [text_batch("Native", UiTextRenderMode::Native)];
    let resolved = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&native, &[]);
    let native_bitmap_atlas = NativeBitmapAtlasPrepareReport {
        frame_index: 7,
        visible_raster_glyph_count: 5,
        source_image_count: 4,
        missing_raster_image_count: 1,
        approximate_raster_image_count: 2,
        source_cache: native_bitmap_atlas::NativeBitmapAtlasSourceCacheFrameReport {
            hit_count: 6,
            approximate_hit_count: 2,
            miss_count: 3,
            insert_count: 4,
            worker_request_submitted_count: 2,
            worker_request_pending_count: 1,
            worker_request_unavailable_count: 1,
            worker_request_failed_count: 2,
            ..Default::default()
        },
        submission: crate::text::atlas::GlyphAtlasBitmapRenderSubmissionReport {
            upload_command_count: 3,
            upload_copy_count: 3,
            upload_byte_len: 384,
            ..Default::default()
        },
        ..NativeBitmapAtlasPrepareReport::default()
    };
    let bitmap_renderer = GlyphAtlasBitmapRendererPrepareReport::default()
        .with_upload_counters_for_test(3, 384, 1, 1, false);

    let report = text_prepare_report(
        &[],
        &native,
        &[],
        &resolved,
        ScreenSpaceUiTextSdfFallbackReport::default(),
        ScreenSpaceUiNativePrepareReport {
            font_faces_changed: false,
            font_ids: ScreenSpaceUiTextFontIdReport::default(),
            bitmap_atlas: native_bitmap_atlas,
        },
        MissingGlyphDiagnosticsReport::default(),
        bitmap_renderer,
        SdfAtlasCacheReport::default(),
        ScreenSpaceUiSdfPrepareReport::default(),
    );

    assert_eq!(
        report.raster_upload,
        ScreenSpaceUiTextRasterUploadReport {
            visible_raster_glyph_count: 5,
            source_image_count: 4,
            missing_raster_image_count: 1,
            approximate_raster_image_count: 2,
            source_cache_hit_count: 6,
            source_cache_approximate_hit_count: 2,
            source_cache_miss_count: 3,
            source_cache_insert_count: 4,
            worker_request_submitted_count: 2,
            worker_request_pending_count: 1,
            worker_request_unavailable_count: 1,
            worker_request_failed_count: 2,
            upload_command_count: 3,
            upload_copy_count: 3,
            upload_byte_len: 384,
            renderer_upload_request_count: 3,
            renderer_upload_byte_len: 384,
            renderer_upload_requeued_count: 1,
            renderer_upload_failure_count: 1,
            renderer_upload_ready_to_write_texture: false,
        }
    );
}

#[test]
fn auto_text_mode_uses_font_asset_default_when_present() {
    let resolved = effective_text_render_mode(
        UiTextRenderMode::Auto,
        Some(&LoadedUiFontAsset {
            family: Some("Studio Mono".to_string()),
            render_mode: Some(UiTextRenderMode::Sdf),
            composite_font: None,
        }),
    );

    assert_eq!(resolved, UiTextRenderMode::Sdf);
}

#[test]
fn explicit_text_mode_overrides_font_asset_default() {
    let resolved = effective_text_render_mode(
        UiTextRenderMode::Native,
        Some(&LoadedUiFontAsset {
            family: Some("Studio Mono".to_string()),
            render_mode: Some(UiTextRenderMode::Sdf),
            composite_font: None,
        }),
    );

    assert_eq!(resolved, UiTextRenderMode::Native);
}

#[test]
fn auto_text_mode_falls_back_to_native_without_font_asset_default() {
    let resolved = effective_text_render_mode(UiTextRenderMode::Auto, None);

    assert_eq!(resolved, UiTextRenderMode::Native);
}

#[test]
fn native_text_align_maps_start_end_through_text_direction() {
    assert_eq!(
        native_text_align(UiTextAlign::Start, UiTextDirection::LeftToRight),
        NativeTextAlign::Left
    );
    assert_eq!(
        native_text_align(UiTextAlign::End, UiTextDirection::LeftToRight),
        NativeTextAlign::Right
    );
    assert_eq!(
        native_text_align(UiTextAlign::Start, UiTextDirection::RightToLeft),
        NativeTextAlign::Right
    );
    assert_eq!(
        native_text_align(UiTextAlign::End, UiTextDirection::RightToLeft),
        NativeTextAlign::Left
    );
    assert_eq!(
        native_text_align(UiTextAlign::Justify, UiTextDirection::LeftToRight),
        NativeTextAlign::Justified
    );
}

#[test]
fn native_text_area_placement_snaps_fractional_origin_to_device_pixels() {
    let mut text = text_batch("editor base.zui", UiTextRenderMode::Native);
    text.frame = UiFrame::new(12.49, 7.51, 120.0, 20.0);
    text.clip_frame = Some(UiFrame::new(12.2, 7.2, 80.0, 20.0));

    let placement = native_text_area_placement(crate::core::math::UVec2::new(200, 80), &text);

    assert_eq!(placement.left, 12.0);
    assert_eq!(placement.top, 8.0);
    assert_eq!(placement.bounds.left, 12);
    assert_eq!(placement.bounds.top, 7);
    assert_eq!(placement.bounds.right, 93);
    assert_eq!(placement.bounds.bottom, 28);
}

#[test]
fn native_text_area_placement_drops_non_finite_origin_values() {
    let mut text = text_batch("folder-open.svg", UiTextRenderMode::Native);
    text.frame = UiFrame::new(f32::NAN, f32::INFINITY, 120.0, 20.0);

    let placement = native_text_area_placement(crate::core::math::UVec2::new(200, 80), &text);

    assert_eq!(placement.left, 0.0);
    assert_eq!(placement.top, 0.0);
    assert_eq!(placement.bounds.left, 0);
    assert_eq!(placement.bounds.top, 0);
}

fn text_batch(text: &str, mode: UiTextRenderMode) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        text: text.to_string(),
        frame: UiFrame::new(0.0, 0.0, 128.0, 24.0),
        clip_frame: None,
        source_range: None,
        glyph_advances: Vec::new(),
        shaped_glyphs: Vec::new(),
        layout_error: None,
        color: [1.0, 1.0, 1.0, 1.0],
        background_color: None,
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        language: None,
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 16.0,
        line_height: 20.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        wrap: UiTextWrap::None,
        style: Default::default(),
        distance_field_mode: match mode {
            UiTextRenderMode::Msdf => crate::text::sdf::SdfMode::Msdf,
            UiTextRenderMode::Mtsdf => crate::text::sdf::SdfMode::Mtsdf,
            UiTextRenderMode::Auto | UiTextRenderMode::Native | UiTextRenderMode::Sdf => {
                crate::text::sdf::SdfMode::Sdf
            }
        },
        text_effects: Default::default(),
        text_decorations: Default::default(),
        text_decoration_baseline: None,
        clip_transform: None,
    }
}
