use crate::asset::assets::FontAsset;
use crate::ui::text::{
    raster_path_for, resolve_text_layout, UiFontRegistry, UiGlyphRasterPath, UiGlyphRasterPolicy,
    UiPreeditSpan, UiTextLayoutRequest, UiTextMeasureCache, UiWidthBucket,
};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiTextRange, UiTextRenderMode, UiTextWrap},
};

#[test]
fn text_font_registry_registers_assets_and_fallback_chain() {
    let mut registry = UiFontRegistry::default();
    let asset = FontAsset {
        source: "assets/fonts/NotoSansCJK-Regular.otf".to_string(),
        family: Some("Noto Sans CJK SC".to_string()),
        render_mode: Some(UiTextRenderMode::Native),
    };

    let id = registry.register_font_asset(&asset).unwrap();

    assert_eq!(id.value(), 1);
    assert_eq!(registry.families().len(), 1);
    assert_eq!(registry.families()[0].family, "Noto Sans CJK SC");
    assert_eq!(
        registry.families()[0].render_mode,
        Some(UiTextRenderMode::Native)
    );
    assert!(registry
        .fallback_chain()
        .iter()
        .any(|family| family == "Noto Sans CJK SC"));

    registry.set_fallback_chain(vec!["Inter".to_string(), " ".to_string()]);
    assert_eq!(registry.fallback_chain(), &["Inter".to_string()]);
}

#[test]
fn text_layout_request_injects_preedit_without_mutating_source() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::None,
        ..UiResolvedStyle::default()
    };
    let preedit = UiPreeditSpan {
        range: UiTextRange { start: 6, end: 6 },
        text: "中文".to_string(),
    };
    let request =
        UiTextLayoutRequest::new("hello ", &style, UiFrame::new(0.0, 0.0, 80.0, 20.0), None)
            .with_preedit(&preedit);

    let resolution = resolve_text_layout(&request);

    assert_eq!(request.text, "hello ");
    assert_eq!(resolution.layout.source_range.end, "hello 中文".len());
    assert_eq!(resolution.layout.lines[0].text, "hello 中文");
    assert_eq!(resolution.first_baseline, 8.0);
}

#[test]
fn text_measure_cache_hits_same_content_style_and_width_bucket() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::Word,
        ..UiResolvedStyle::default()
    };
    let request = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(0.0, 0.0, 60.0, 20.0),
        None,
    );
    let mut cache = UiTextMeasureCache::default();

    assert_eq!(cache.resolve_or_shape(&request).layout.lines.len(), 1);
    assert_eq!(cache.resolve_or_shape(&request).layout.lines.len(), 1);

    assert_eq!(cache.frame_shape_count(), 1);
    assert_eq!(UiWidthBucket::from_request(&request).value(), 12);
}

#[test]
fn text_measure_cache_reshapes_when_wrap_bucket_changes() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::Word,
        ..UiResolvedStyle::default()
    };
    let narrow = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(0.0, 0.0, 25.0, 40.0),
        None,
    );
    let wide = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(0.0, 0.0, 60.0, 20.0),
        None,
    );
    let mut cache = UiTextMeasureCache::default();

    assert_eq!(cache.resolve_or_shape(&narrow).layout.lines.len(), 2);
    assert_eq!(cache.resolve_or_shape(&wide).layout.lines.len(), 1);

    assert_eq!(cache.frame_shape_count(), 2);
}

#[test]
fn text_raster_path_prefers_bitmap_for_small_static_ui_text() {
    assert_eq!(raster_path_for(12.0, false), UiGlyphRasterPath::Bitmap);
    assert_eq!(raster_path_for(32.0, false), UiGlyphRasterPath::Sdf);
    assert_eq!(raster_path_for(12.0, true), UiGlyphRasterPath::Sdf);

    let policy = UiGlyphRasterPolicy {
        sdf_min_size_px: 18.0,
        scalable_prefers_sdf: false,
    };
    assert_eq!(policy.path_for(17.0, true), UiGlyphRasterPath::Bitmap);
    assert_eq!(policy.path_for(18.0, true), UiGlyphRasterPath::Sdf);
}
