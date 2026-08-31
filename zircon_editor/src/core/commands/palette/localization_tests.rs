use super::*;
use crate::core::commands::EditorCommandRegistry;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::i18n::{EditorI18nService, EditorLocale};

#[test]
fn locale_projection_reuses_neutral_catalog_and_rebuilds_only_locale_index() {
    let registry = EditorCommandRegistry::default_workbench();
    let catalog = registry.command_palette_catalog();
    let i18n = EditorI18nService::default();
    let context = CommandEvalCtx::interactive();
    let english = EditorLocale::parse("en").unwrap();
    let chinese = EditorLocale::parse("zh-CN").unwrap();

    let english_window = catalog.query_window(&i18n, &english, &context, "open project", 0, 8);
    let chinese_window = catalog.query_window(&i18n, &chinese, &context, "打开工程", 0, 8);

    assert_eq!(english_window.catalog_generation(), catalog.generation());
    assert_eq!(chinese_window.catalog_generation(), catalog.generation());
    assert_eq!(
        english_window
            .entries()
            .next()
            .map(|entry| entry.id.as_str()),
        Some("file.project.open")
    );
    assert_eq!(
        english_window
            .entries()
            .next()
            .map(|entry| entry.label.as_str()),
        Some("Open Project")
    );
    assert_eq!(
        chinese_window
            .entries()
            .next()
            .map(|entry| entry.label.as_str()),
        Some("打开工程")
    );
    assert_eq!(catalog.cached_locale_projection_count(), 2);

    let repeated = catalog.query_window(&i18n, &chinese, &context, "打开工程", 0, 8);
    assert_eq!(repeated.catalog_generation(), catalog.generation());
    assert_eq!(catalog.cached_locale_projection_count(), 2);
}

#[test]
fn missing_command_translation_projects_the_validated_raw_key() {
    let mut registry = EditorCommandRegistry::default();
    registry
        .register(EditorCommandDescriptor::operation(
            EditorOperationPath::parse("fixture.translation.missing").unwrap(),
        ))
        .unwrap();
    let catalog = registry.command_palette_catalog();
    let i18n = EditorI18nService::default();
    let locale = EditorLocale::parse("zh-CN").unwrap();

    let window = catalog.query_window(
        &i18n,
        &locale,
        &CommandEvalCtx::interactive(),
        "command.fixture.translation.missing.label",
        0,
        8,
    );

    assert_eq!(
        window.entries().next().map(|entry| entry.label.as_str()),
        Some("command.fixture.translation.missing.label")
    );
}
