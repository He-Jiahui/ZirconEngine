use crate::ui::theme::UiThemeRegistry;
use zircon_runtime_interface::ui::style::{
    UiRgbaColor, UiStyleColor, UiThemeDocument, UiThemeTokenRef,
};

#[test]
fn ui_theme_registry_resolves_palette_tokens() {
    let registry = UiThemeRegistry::default();

    assert_eq!(
        registry.resolve_token(&UiThemeTokenRef::new("palette.accent")),
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(60, 199, 214, 255)))
    );
    assert_eq!(
        registry.resolve_token(&UiThemeTokenRef::new("palette.surface.2")),
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(27, 31, 35, 255)))
    );
    assert_eq!(
        registry.resolve_token(&UiThemeTokenRef::new("palette.missing")),
        None
    );
}

#[test]
fn ui_theme_registry_resolves_style_color_roles_without_touching_literals() {
    let registry = UiThemeRegistry::default();

    assert_eq!(
        registry.resolve_role("theme.palette.accent"),
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(60, 199, 214, 255)))
    );
    assert_eq!(
        registry.resolve_role("$theme.palette.text.primary"),
        Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(232, 236, 238, 255)))
    );
    assert_eq!(registry.resolve_role("palette.unknown"), None);

    let role = UiStyleColor::Role("theme.palette.separator".to_string());
    assert_eq!(
        registry.resolve_style_color(&role),
        UiStyleColor::Rgba(UiRgbaColor::from_u8(57, 65, 71, 255))
    );

    let rgba = UiStyleColor::Rgba(UiRgbaColor::from_u8(1, 2, 3, 255));
    assert_eq!(registry.resolve_style_color(&rgba), rgba);

    let unknown = UiStyleColor::Role("editor.local.unresolved".to_string());
    assert_eq!(registry.resolve_style_color(&unknown), unknown);
}

#[test]
fn ui_theme_registry_reports_fingerprint_changes() {
    let mut registry = UiThemeRegistry::default();
    let same = registry.apply_document(UiThemeDocument::dark());
    assert!(!same.changed);

    let mut changed = UiThemeDocument::dark();
    changed.id = "zircon.dark.variant".to_string();
    changed.palette.accent = UiRgbaColor::from_u8(80, 210, 190, 255);
    let outcome = registry.apply_document(changed);

    assert!(outcome.changed);
    assert_ne!(outcome.previous_fingerprint, outcome.new_fingerprint);
    assert_eq!(registry.active().id, "zircon.dark.variant");
}
