//! Search shortcut text through the real registry, locale projection, and postings index.

use std::collections::BTreeMap;

use super::{CommandEvalCtx, EditorCommandPaletteMru};
use crate::core::commands::{
    EditorCommandDescriptor, EditorCommandPresentation, EditorCommandRegistry, EditorKeyChord,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::{EditorI18nService, EditorLocale, EditorLocalizationBundle};

#[test]
fn palette_search_and_display_follow_overrides_and_unbinding() {
    use crate::core::commands::EditorKeymap;
    use crate::core::settings::EditorKeymapOverrides;
    let registry = EditorCommandRegistry::default_workbench();
    let overrides = EditorKeymapOverrides::new(BTreeMap::from([
        (EditorOperationPath::parse("runtime.play_mode.enter").unwrap(),
            Some("Ctrl+Alt+P".parse().unwrap())),
        (EditorOperationPath::parse("file.project.save").unwrap(), None),
    ]));
    let keymap = EditorKeymap::default_workbench().with_overrides(&overrides);
    let catalog = registry.command_palette_catalog_with_keymap(&keymap);
    assert!(std::sync::Arc::ptr_eq(&catalog,
        &registry.command_palette_catalog_with_keymap(&keymap)));
    let i18n = EditorI18nService::default();
    let locale = i18n.active_locale();
    let context = CommandEvalCtx::interactive().with_project_open(true);
    let window = catalog.query_window(&i18n, &locale, &context, "Ctrl+Alt+P", 0, 10);
    let play = window.entries().find(|entry| entry.id == "runtime.play_mode.enter").unwrap();
    assert_eq!(play.shortcut, "Ctrl+Alt+P");
    let all = catalog.query_window(&i18n, &locale, &context, "", 0, 100);
    assert!(all.entries().find(|entry| entry.id == "file.project.save").unwrap().shortcut.is_empty());
}

fn command(id: &str, label: &str, shortcut: Option<&str>) -> EditorCommandDescriptor {
    let bundle = EditorLocalizationBundle::from_locale_maps(
        "fixture.palette.shortcut",
        BTreeMap::from([(
            "en".to_string(),
            BTreeMap::from([
                ("command.shortcut.label".to_string(), label.to_string()),
                (
                    "command.shortcut.description".to_string(),
                    "Shortcut regression fixture".to_string(),
                ),
            ]),
        )]),
    )
    .unwrap();
    let presentation = EditorCommandPresentation::localized(
        "fixture.palette.shortcut",
        "command.shortcut.label",
        "command.shortcut.description",
    )
    .unwrap();
    let mut command = EditorCommandDescriptor::localized_operation(
        EditorOperationPath::parse(id).unwrap(),
        presentation,
    );
    if let Some(shortcut) = shortcut {
        command = command.with_default_chord(shortcut.parse::<EditorKeyChord>().unwrap());
    }
    command.bind_localization_bundle(&bundle).unwrap();
    command
}

#[test]
fn palette_matches_shortcut_only_query_and_preserves_presentation() {
    let registry = EditorCommandRegistry::new(vec![
        command("fixture.shortcut.target", "ÉCOLE", Some("Ctrl+Shift+P")),
        command("fixture.shortcut.decoy", "OTHER", None),
    ])
    .unwrap();
    let i18n = EditorI18nService::default();
    let locale = EditorLocale::parse("en").unwrap();
    let context = CommandEvalCtx::interactive();
    let catalog = registry.command_palette_catalog();

    for query in ["Ctrl+Shift+P", " \tcTrL+sHiFt+p\t "] {
        let window = catalog.query_window(&i18n, &locale, &context, query, 0, 1);
        assert_eq!(window.total_match_count(), 1);
        assert_eq!(window.catalog_generation(), catalog.generation());
        let entry = window.entries().next().unwrap();
        assert_eq!(entry.id, "fixture.shortcut.target");
        assert_eq!(entry.shortcut, "Ctrl+Shift+P");
        assert_eq!(entry.label, "ÉCOLE");
        assert_eq!(window.metrics().retained_handles, 1);
    }
    let unicode = catalog.query_window(&i18n, &locale, &context, "école", 0, 1);
    assert_eq!(unicode.total_match_count(), 1);
    assert_eq!(
        unicode.entries().next().unwrap().id,
        "fixture.shortcut.target"
    );
}

#[test]
fn palette_without_shortcut_preserves_existing_fields_and_empty_window() {
    let registry = EditorCommandRegistry::new(vec![
        command("fixture.shortcut.alpha", "ALPHA", None).with_keywords(["BÉTA"]),
        command("fixture.shortcut.beta", "OTHER", None),
    ])
    .unwrap();
    let i18n = EditorI18nService::default();
    let locale = EditorLocale::parse("en").unwrap();
    let context = CommandEvalCtx::interactive();
    let catalog = registry.command_palette_catalog();

    for query in ["alpha", "béTa"] {
        let window = catalog.query_window(&i18n, &locale, &context, query, 0, 1);
        assert_eq!(window.total_match_count(), 1);
        let entry = window.entries().next().unwrap();
        assert_eq!(entry.id, "fixture.shortcut.alpha");
        assert!(entry.shortcut.is_empty());
    }
    let missing = catalog.query_window(&i18n, &locale, &context, "Ctrl+Shift+P", 0, 1);
    assert_eq!(missing.total_match_count(), 0);
    assert!(missing.is_empty());
    let empty = catalog.query_window(&i18n, &locale, &context, " \t ", 1, 1);
    assert_eq!(empty.total_match_count(), 2);
    assert_eq!(empty.offset(), 1);
    assert_eq!(empty.entries().next().unwrap().id, "fixture.shortcut.beta");
}

#[test]
fn palette_shortcut_query_preserves_mru_ties_and_paged_match_count() {
    let registry = EditorCommandRegistry::new(vec![
        command("fixture.shortcut.gamma", "THIRD", Some("Ctrl+P")),
        command("fixture.shortcut.beta", "SECOND", Some("Ctrl+P")),
        command("fixture.shortcut.alpha", "FIRST", Some("Ctrl+P")),
    ])
    .unwrap();
    let i18n = EditorI18nService::default();
    let locale = EditorLocale::parse("en").unwrap();
    let context = CommandEvalCtx::interactive();
    let catalog = registry.command_palette_catalog();
    let mru =
        EditorCommandPaletteMru::new(
            [EditorOperationPath::parse("fixture.shortcut.beta").unwrap()],
        )
        .unwrap();
    let first = catalog.query_window_with_mru(&i18n, &locale, &context, "ctrl+p", 0, 2, &mru);
    assert_eq!(first.total_match_count(), 3);
    assert_eq!(
        first
            .entries()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        ["fixture.shortcut.beta", "fixture.shortcut.alpha"]
    );
    assert_eq!(first.metrics().retained_handles, 2);
    let next = catalog.query_window_with_mru(&i18n, &locale, &context, "CTRL+P", 2, 2, &mru);
    assert_eq!(next.total_match_count(), 3);
    assert_eq!(next.offset(), 2);
    assert_eq!(
        next.entries()
            .map(|entry| entry.id.as_str())
            .collect::<Vec<_>>(),
        ["fixture.shortcut.gamma"]
    );
    assert_eq!(next.metrics().retained_handles, 1);
    assert_eq!(catalog.cached_locale_projection_count(), 1);
}
