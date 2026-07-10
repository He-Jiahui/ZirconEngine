use crate::asset::assets::{FontAsset, FontAssetRenderStrategy};
use crate::graphics::text::font::default_runtime_font_families;
use crate::ui::text::UiFontRegistry;
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

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
        composite_font: None,
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
        composite_font: None,
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
