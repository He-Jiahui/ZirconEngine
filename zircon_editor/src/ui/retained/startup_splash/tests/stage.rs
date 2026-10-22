use super::*;
use crate::core::i18n::{EditorI18nService, EditorLocale};
use zircon_runtime_interface::ui::template::UiLocalizedTextRef;

#[test]
fn startup_stages_are_ordered_and_only_known_messages_are_accepted() {
    for pair in EditorStartupStage::ALL.windows(2) {
        assert!(pair[0] < pair[1]);
    }
    for stage in EditorStartupStage::ALL {
        assert_eq!(EditorStartupStage::parse(stage.wire()), Some(stage));
    }
    for message in ["", "queued", "ready\nproject", "unknown"] {
        assert_eq!(EditorStartupStage::parse(message), None);
    }
}

#[test]
fn startup_and_welcome_copy_resolves_in_both_canonical_locales() {
    let i18n = EditorI18nService::default();
    let keys = [
        "editor.startup.brand",
        "editor.startup.product",
        "editor.startup.window_title",
        "editor.startup.title",
        "editor.startup.elapsed",
        "editor.startup.dismiss",
        "editor.welcome.project_heading",
        "editor.welcome.project_help",
        "editor.welcome.open_project",
        "editor.welcome.open_project_hint",
        "editor.welcome.create_project",
        "editor.welcome.create_project_hint",
        "editor.welcome.empty_title",
        "editor.welcome.empty_hint",
    ];
    for locale in [
        EditorLocale::english(),
        EditorLocale::parse("zh-CN").unwrap(),
    ] {
        for key in keys.into_iter().chain(
            EditorStartupStage::ALL
                .into_iter()
                .flat_map(|stage| [stage.text_key(false), stage.text_key(true)]),
        ) {
            let reference = UiLocalizedTextRef {
                key: key.to_owned(),
                table: Some("editor".to_owned()),
                fallback: None,
            };
            let text = i18n
                .resolve_localized_text_for_locale(&locale, &reference)
                .unwrap();
            assert!(!text.is_empty());
            assert_ne!(text.as_ref(), key);
            assert!(!text.to_ascii_lowercase().contains("queued"));
        }
    }
}
