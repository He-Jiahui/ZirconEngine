use super::*;
use zircon_runtime_interface::ui::template::UiLocalizedTextRef;

#[test]
fn ui_reference_uses_canonical_catalog_locale_fallback_and_explicit_fallback() {
    let service = EditorI18nService::default();
    let mut reference = UiLocalizedTextRef {
        key: "editor.workbench.details.title".to_owned(),
        table: Some("editor".to_owned()),
        fallback: None,
    };
    for locale in [
        EditorLocale::english(),
        EditorLocale::parse("zh-CN").unwrap(),
        EditorLocale::parse("fr-FR").unwrap(),
    ] {
        assert_eq!(
            service
                .resolve_localized_text_for_locale(&locale, &reference)
                .unwrap(),
            service.translate_for_locale(&locale, &reference.key)
        );
    }
    reference.key = "editor.missing.localized.heading".to_owned();
    assert!(matches!(
        service.resolve_localized_text_for_locale(&EditorLocale::english(), &reference),
        Err(EditorI18nError::MissingLocalizedTextKey { .. })
    ));
    reference.fallback = Some("Authored fallback".to_owned());
    assert_eq!(
        service
            .resolve_localized_text_for_locale(&EditorLocale::english(), &reference)
            .unwrap()
            .as_ref(),
        "Authored fallback"
    );
    reference.key.clear();
    assert!(matches!(
        service.resolve_localized_text_for_locale(&EditorLocale::english(), &reference),
        Err(EditorI18nError::InvalidTranslationKey(_))
    ));
}

#[test]
fn ui_reference_rejects_unknown_tables_and_empty_fallback() {
    let service = EditorI18nService::default();
    let mut reference = UiLocalizedTextRef {
        key: "editor.missing.localized.heading".to_owned(),
        table: Some("plugin.unavailable".to_owned()),
        fallback: None,
    };
    assert!(matches!(
        service.resolve_localized_text_for_locale(&EditorLocale::english(), &reference),
        Err(EditorI18nError::UnknownLocalizedTextTable(_))
    ));
    reference.table = Some("editor".to_owned());
    reference.fallback = Some(String::new());
    assert!(matches!(
        service.resolve_localized_text_for_locale(&EditorLocale::english(), &reference),
        Err(EditorI18nError::EmptyTranslation(_))
    ));
}
