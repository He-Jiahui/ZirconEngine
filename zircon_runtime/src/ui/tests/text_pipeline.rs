use crate::asset::assets::{FontAsset, FontAssetRenderStrategy};
use crate::graphics::text::font::default_runtime_font_families;
use crate::ui::text::{
    resolve_text_layout, UiFontRegistry, UiPreeditSpan, UiTextLayoutRequest, UiTextMeasureCache,
    UiWidthBucket,
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
        face_index: 0,
        family_members: Vec::new(),
        variable_instances: Vec::new(),
        fallback_families: vec![
            "Project Emoji".to_string(),
            "Inter".to_string(),
            " ".to_string(),
        ],
        render_strategy: FontAssetRenderStrategy::default(),
        metadata: None,
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
    assert!(registry
        .fallback_chain()
        .iter()
        .any(|family| family == "Project Emoji"));
    assert_eq!(
        registry
            .fallback_chain()
            .iter()
            .filter(|family| family.as_str() == "Inter")
            .count(),
        1
    );

    registry.set_fallback_chain(vec!["Inter".to_string(), " ".to_string()]);
    assert_eq!(registry.fallback_chain(), &["Inter".to_string()]);
}

#[test]
fn text_font_registry_uses_asset_render_strategy_default_mode() {
    let mut registry = UiFontRegistry::default();
    let asset = FontAsset {
        source: "assets/fonts/ProjectUiSans.ttf".to_string(),
        family: Some("Project UI Sans".to_string()),
        render_mode: None,
        face_index: 0,
        family_members: Vec::new(),
        variable_instances: Vec::new(),
        fallback_families: Vec::new(),
        render_strategy: FontAssetRenderStrategy {
            default_mode: Some(UiTextRenderMode::Auto),
            allow_native: Some(false),
            allow_sdf: Some(true),
        },
        metadata: None,
    };

    let id = registry.register_font_asset(&asset).unwrap();

    assert_eq!(id.value(), 1);
    assert_eq!(
        registry.families()[0].render_mode,
        Some(UiTextRenderMode::Sdf)
    );
    assert!(registry
        .fallback_chain()
        .iter()
        .any(|family| family == "Project UI Sans"));
}

#[test]
fn text_font_registry_default_chain_comes_from_runtime_font_database() {
    let registry = UiFontRegistry::default();
    let expected: Vec<String> = default_runtime_font_families()
        .iter()
        .map(|family| family.as_str().to_string())
        .collect();

    assert_eq!(registry.fallback_chain(), expected.as_slice());
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
    assert!(resolution.first_baseline > 0.0);
    assert!(resolution.first_baseline <= style.line_height);
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
    assert!(UiWidthBucket::from_request(&request).value() >= 1);
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
