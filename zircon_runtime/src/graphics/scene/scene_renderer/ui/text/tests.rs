use super::super::sdf_atlas::{SdfAtlasDirtyPageReport, SdfAtlasRect};
use super::super::sdf_upload::SdfAtlasUploadPageReport;
use super::*;
use zircon_runtime_interface::ui::surface::UiTextWritingMode;

#[test]
fn text_backend_routing_keeps_explicit_native_out_of_sdf_atlas_batches() {
    let native = text_batch("Normal", UiTextRenderMode::Native);
    let sdf = text_batch("Signed", UiTextRenderMode::Sdf);

    let routed = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&[native], &[sdf]);

    assert_eq!(routed.native_texts().len(), 1);
    assert_eq!(routed.native_texts()[0].text, "Normal");
    assert_eq!(routed.sdf_texts().len(), 1);
    assert_eq!(routed.sdf_texts()[0].text, "Signed");
    assert_eq!(routed.sdf_atlas_texts().len(), 1);
    assert_eq!(routed.sdf_atlas_texts()[0].text, "Signed");
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
    assert_eq!(routed.sdf_atlas_texts()[0].text, "SdfAuto");
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
            page_key: crate::graphics::text::atlas::GlyphAtlasPageKey::new(
                crate::graphics::text::atlas::GlyphAtlasFormat::Sdf,
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
                page_key: crate::graphics::text::atlas::GlyphAtlasPageKey::new(
                    crate::graphics::text::atlas::GlyphAtlasFormat::Sdf,
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
    };

    let report = text_prepare_report(
        &auto,
        &native,
        &sdf,
        &resolved,
        ScreenSpaceUiTextSdfFallbackReport::default(),
        ScreenSpaceUiNativePrepareReport {
            font_ids: ScreenSpaceUiTextFontIdReport::default(),
            bitmap_atlas: NativeBitmapAtlasPrepareReport {
                visible_raster_glyph_count: 2,
                source_image_count: 1,
                unsupported_glyph_count: 1,
                clipped_glyph_count: 0,
                atlas_storage_format: Some(GlyphAtlasStorageFormat::R8Unorm),
                mixed_atlas_storage_format: false,
                storage_submission_count: 1,
                storage_submission_visible_glyph_count: 1,
                mixed_storage_replacement_ready: false,
                requires_background_composite: false,
                replaces_glyphon: false,
                submission: Default::default(),
            },
        },
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
            native_bitmap_atlas: NativeBitmapAtlasPrepareReport {
                visible_raster_glyph_count: 2,
                source_image_count: 1,
                unsupported_glyph_count: 1,
                clipped_glyph_count: 0,
                atlas_storage_format: Some(GlyphAtlasStorageFormat::R8Unorm),
                mixed_atlas_storage_format: false,
                storage_submission_count: 1,
                storage_submission_visible_glyph_count: 1,
                mixed_storage_replacement_ready: false,
                requires_background_composite: false,
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
fn native_bitmap_atlas_handoff_uses_single_storage_replacement() {
    let report = NativeBitmapAtlasPrepareReport {
        replaces_glyphon: true,
        mixed_storage_replacement_ready: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::SingleStorageReplacement
    );
}

#[test]
fn native_bitmap_atlas_handoff_routes_mixed_storage_to_renderer_submissions() {
    let report = NativeBitmapAtlasPrepareReport {
        mixed_storage_replacement_ready: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::MixedStorageReplacement
    );
}

#[test]
fn native_bitmap_atlas_handoff_keeps_glyphon_for_background_composite() {
    let report = NativeBitmapAtlasPrepareReport {
        requires_background_composite: true,
        ..NativeBitmapAtlasPrepareReport::default()
    };

    assert_eq!(
        native_bitmap_atlas_handoff_for_report(&report),
        NativeBitmapAtlasHandoff::GlyphonFallback
    );
}

#[test]
fn auto_text_mode_uses_font_asset_default_when_present() {
    let resolved = effective_text_render_mode(
        UiTextRenderMode::Auto,
        Some(&LoadedUiFontAsset {
            family: Some("Studio Mono".to_string()),
            render_mode: Some(UiTextRenderMode::Sdf),
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
fn text_attrs_maps_shared_rich_run_style_to_glyphon_attrs() {
    let attrs = text_attrs(
        Some("Zircon Sans"),
        650,
        UiTextRunPaintStyle {
            strong: true,
            emphasis: true,
            code: false,
        },
    );

    assert_eq!(attrs.family, Family::Name("Zircon Sans"));
    assert_eq!(attrs.weight, Weight::BOLD);
    assert_eq!(attrs.style, Style::Italic);

    let medium_attrs = text_attrs(Some("Zircon Sans"), 500, UiTextRunPaintStyle::default());

    assert_eq!(medium_attrs.weight, Weight(500));

    let code_attrs = text_attrs(
        Some("Zircon Sans"),
        450,
        UiTextRunPaintStyle {
            strong: false,
            emphasis: false,
            code: true,
        },
    );

    assert_eq!(code_attrs.family, Family::Monospace);
    assert_eq!(code_attrs.weight, Weight(450));
}

#[test]
fn native_text_align_maps_start_end_through_text_direction() {
    assert_eq!(
        native_text_align(UiTextAlign::Start, UiTextDirection::LeftToRight),
        Align::Left
    );
    assert_eq!(
        native_text_align(UiTextAlign::End, UiTextDirection::LeftToRight),
        Align::Right
    );
    assert_eq!(
        native_text_align(UiTextAlign::Start, UiTextDirection::RightToLeft),
        Align::Right
    );
    assert_eq!(
        native_text_align(UiTextAlign::End, UiTextDirection::RightToLeft),
        Align::Left
    );
    assert_eq!(
        native_text_align(UiTextAlign::Justify, UiTextDirection::LeftToRight),
        Align::Justified
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

fn text_batch(text: &str, _mode: UiTextRenderMode) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        text: text.to_string(),
        frame: UiFrame::new(0.0, 0.0, 128.0, 24.0),
        clip_frame: None,
        source_range: None,
        glyph_advances: Vec::new(),
        color: [1.0, 1.0, 1.0, 1.0],
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 16.0,
        line_height: 20.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        wrap: UiTextWrap::None,
        style: Default::default(),
    }
}
