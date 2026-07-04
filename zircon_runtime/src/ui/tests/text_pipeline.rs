use crate::asset::assets::{FontAsset, FontAssetRenderStrategy};
use crate::graphics::text::font::default_runtime_font_families;
use crate::ui::surface::UiSurface;
use crate::ui::text::{
    resolve_text_layout, UiFontRegistry, UiPreeditSpan, UiTextLayoutRequest, UiTextMeasureCache,
    UiWidthBucket,
};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{AxisConstraint, BoxConstraints, StretchMode, UiContainerKind, UiFrame, UiSize},
    surface::{UiResolvedStyle, UiTextRange, UiTextRenderMode, UiTextWrap},
    tree::{UiTemplateNodeMetadata, UiTreeNode},
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
fn text_measure_cache_hits_same_layout_request() {
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
fn text_measure_cache_reshapes_when_frame_origin_changes() {
    let style = UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap: UiTextWrap::Word,
        ..UiResolvedStyle::default()
    };
    let first = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(8.0, 0.0, 60.0, 20.0),
        None,
    );
    let shifted = UiTextLayoutRequest::new(
        "Alpha Beta",
        &style,
        UiFrame::new(24.0, 0.0, 60.0, 20.0),
        None,
    );
    let mut cache = UiTextMeasureCache::default();

    assert_eq!(cache.resolve_or_shape(&first).layout.lines[0].frame.x, 8.0);
    assert_eq!(
        cache.resolve_or_shape(&shifted).layout.lines[0].frame.x,
        24.0
    );

    assert_eq!(cache.frame_shape_count(), 2);
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
fn text_measure_cache_is_consumed_by_surface_render_rebuild() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.text.measure_cache.surface"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root"))
            .with_container(UiContainerKind::BlockBox)
            .with_constraints(fixed_constraints(160.0, 48.0)),
    );
    for (node_id, path) in [
        (UiNodeId::new(2), "root/first"),
        (UiNodeId::new(3), "root/second"),
    ] {
        surface
            .tree
            .insert_child(
                UiNodeId::new(1),
                UiTreeNode::new(node_id, UiNodePath::new(path))
                    .with_constraints(fixed_constraints(120.0, 18.0))
                    .with_template_metadata(repeated_text_metadata()),
            )
            .expect("text child should be inserted");
    }

    surface
        .compute_layout(UiSize::new(160.0, 48.0))
        .expect("surface layout should compute");

    assert_eq!(text_layout_command_count(&surface), 2);
    assert_eq!(
        surface.text_measure_cache.frame_shape_count(),
        2,
        "distinct text node frames should not reuse absolute text line geometry"
    );

    surface.rebuild();

    assert_eq!(text_layout_command_count(&surface), 2);
    assert_eq!(
        surface.text_measure_cache.frame_shape_count(),
        0,
        "unchanged surface rebuild should hit retained text measure cache entries"
    );
}

fn repeated_text_metadata() -> UiTemplateNodeMetadata {
    UiTemplateNodeMetadata {
        component: "Text".to_string(),
        attributes: toml::from_str(
            r#"
text = "Repeated text label"
font_size = 10.0
line_height = 12.0
wrap = "Word"
"#,
        )
        .expect("text metadata should parse"),
        ..Default::default()
    }
}

fn fixed_constraints(width: f32, height: f32) -> BoxConstraints {
    BoxConstraints {
        width: fixed_axis(width),
        height: fixed_axis(height),
    }
}

fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size,
        max: size,
        preferred: size,
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}

fn text_layout_command_count(surface: &UiSurface) -> usize {
    surface
        .render_extract
        .list
        .commands
        .iter()
        .filter(|command| command.text_layout.is_some())
        .count()
}
