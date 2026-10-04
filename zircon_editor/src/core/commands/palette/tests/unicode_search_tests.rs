//! Exercise lowercase search through the real registry, locale projection, and catalog index.

use std::collections::BTreeMap;

use super::CommandEvalCtx;
use crate::core::commands::{
    EditorCommandDescriptor, EditorCommandPresentation, EditorCommandRegistry,
};
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::{EditorI18nService, EditorLocale, EditorLocalizationBundle};

fn localized_command(id: &str, label: &str, keywords: &[&str]) -> EditorCommandDescriptor {
    let bundle = EditorLocalizationBundle::from_locale_maps(
        "fixture.palette.search",
        BTreeMap::from([(
            "en".to_string(),
            BTreeMap::from([
                ("command.search.label".to_string(), label.to_string()),
                (
                    "command.search.description".to_string(),
                    "Search regression fixture".to_string(),
                ),
            ]),
        )]),
    )
    .unwrap();
    let presentation = EditorCommandPresentation::localized(
        "fixture.palette.search",
        "command.search.label",
        "command.search.description",
    )
    .unwrap();
    let mut command = EditorCommandDescriptor::localized_operation(
        EditorOperationPath::parse(id).unwrap(),
        presentation,
    )
    .with_keywords(keywords.iter().copied());
    command.bind_localization_bundle(&bundle).unwrap();
    command
}

#[test]
fn palette_matches_unicode_lowercase_labels_and_preserves_original_presentation() {
    for (label, query) in [
        ("ÉCOLE", "école"),
        ("ΟΣ", "ος"),
        ("İSTANBUL", "i\u{307}stanbul"),
    ] {
        let registry = EditorCommandRegistry::new(vec![
            localized_command("fixture.search.target", label, &[]),
            localized_command("fixture.search.decoy", "OTHER", &[]),
        ])
        .unwrap();
        let i18n = EditorI18nService::default();
        let locale = EditorLocale::parse("en").unwrap();
        let catalog = registry.command_palette_catalog();
        let window =
            catalog.query_window(&i18n, &locale, &CommandEvalCtx::interactive(), query, 0, 1);

        assert_eq!(window.total_match_count(), 1, "label={label}");
        let entry = window.entries().next().unwrap();
        assert_eq!(entry.id, "fixture.search.target");
        assert_eq!(entry.label, label);
    }
}

#[test]
fn palette_matches_unicode_keywords_with_existing_query_lowercase_and_trim() {
    let registry = EditorCommandRegistry::new(vec![
        localized_command("fixture.search.target", "Target", &["BÉTA"]),
        localized_command("fixture.search.decoy", "OTHER", &[]),
    ])
    .unwrap();
    let i18n = EditorI18nService::default();
    let locale = EditorLocale::parse("en").unwrap();
    let catalog = registry.command_palette_catalog();
    let window = catalog.query_window(
        &i18n,
        &locale,
        &CommandEvalCtx::interactive(),
        " \tbéTa\t ",
        0,
        1,
    );

    assert_eq!(window.total_match_count(), 1);
    assert_eq!(window.entries().next().unwrap().id, "fixture.search.target");
}

#[test]
fn palette_preserves_ascii_matching_and_empty_query_window_bounds() {
    let registry = EditorCommandRegistry::new(vec![
        localized_command("fixture.search.alpha", "ALPHA", &[]),
        localized_command("fixture.search.beta", "BETA", &[]),
    ])
    .unwrap();
    let i18n = EditorI18nService::default();
    let locale = EditorLocale::parse("en").unwrap();
    let context = CommandEvalCtx::interactive();
    let catalog = registry.command_palette_catalog();
    let matching = catalog.query_window(&i18n, &locale, &context, "alpha", 0, 1);
    assert_eq!(matching.total_match_count(), 1);
    assert_eq!(
        matching.entries().next().unwrap().id,
        "fixture.search.alpha"
    );

    let empty_query = catalog.query_window(&i18n, &locale, &context, " \t ", 1, 1);
    assert_eq!(empty_query.total_match_count(), 2);
    assert_eq!(empty_query.entries().count(), 1);
    assert_eq!(
        empty_query.entries().next().unwrap().id,
        "fixture.search.beta"
    );
}
