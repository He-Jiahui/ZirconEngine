use crate::ui::template::{
    collect_document_localization_report, compiled_asset_package_manifest_from_artifact_bytes,
    localization_table_keys_from_toml_str, validate_localization_report_against_catalog,
    UiAssetLoader, UiDocumentCompiler, UiLocalizationTableCatalog,
};
use zircon_runtime_interface::ui::template::{
    UiCompiledAssetPackageManifest, UiCompiledAssetPackageProfile,
    UiLocalizationDiagnosticSeverity, UiTextDirection,
};

const LOCALIZED_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.localization"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Label"

[root.props]
text = { text_key = "editor.localization.title", table = "editor", fallback = "Localization", direction = "ltr" }
title = "Literal tooltip"

[[root.children]]
[root.children.node]
node_id = "status"
kind = "native"
type = "Label"
props = { text = "Ready" }
"##;

const INVALID_LOCALIZED_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.localization.invalid"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Label"

[root.props]
text = { text_key = "" }
"##;

#[test]
fn localization_collector_reports_dependencies_and_literal_extraction_candidates() {
    let document = UiAssetLoader::load_toml_str(LOCALIZED_LAYOUT).unwrap();

    let report = collect_document_localization_report(&document);

    assert_eq!(report.dependencies.len(), 1);
    assert_eq!(report.dependencies[0].path, "nodes.root.props.text");
    assert_eq!(
        report.dependencies[0].reference.key,
        "editor.localization.title"
    );
    assert_eq!(
        report.dependencies[0].reference.table.as_deref(),
        Some("editor")
    );
    assert_eq!(
        report.dependencies[0].direction,
        UiTextDirection::LeftToRight
    );

    let candidates = report
        .extraction_candidates
        .iter()
        .map(|candidate| (candidate.path.as_str(), candidate.text.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        candidates,
        vec![
            ("nodes.root.props.title", "Literal tooltip"),
            ("nodes.status.props.text", "Ready")
        ]
    );
    assert!(report.diagnostics.is_empty());
}

#[test]
fn localization_collector_diagnoses_empty_text_keys() {
    let document = UiAssetLoader::load_toml_str(INVALID_LOCALIZED_LAYOUT).unwrap();

    let report = collect_document_localization_report(&document);

    assert_eq!(report.dependencies.len(), 0);
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(report.diagnostics[0].path, "nodes.root.props.text");
    assert!(report.diagnostics[0].message.contains("empty key"));
}

#[test]
fn compiler_rejects_invalid_localization_refs_before_package_report() {
    let document = UiAssetLoader::load_toml_str(INVALID_LOCALIZED_LAYOUT).unwrap();

    let error = UiDocumentCompiler::default()
        .validate_package(&document, UiCompiledAssetPackageProfile::Runtime)
        .expect_err("empty localization keys must be package-validation errors");

    assert!(
        error.to_string().contains("empty key"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn compiler_accepts_localized_text_refs_as_string_component_props() {
    let document = UiAssetLoader::load_toml_str(LOCALIZED_LAYOUT).unwrap();

    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let text = compiled
        .template_instance()
        .root
        .attributes
        .get("text")
        .and_then(toml::Value::as_table)
        .expect("localized text refs stay available as structured compiled attributes");

    assert_eq!(
        text.get("text_key").and_then(toml::Value::as_str),
        Some("editor.localization.title")
    );
    assert_eq!(
        text.get("fallback").and_then(toml::Value::as_str),
        Some("Localization")
    );
}

#[test]
fn package_validation_reports_localization_and_manifest_rows() {
    let document = UiAssetLoader::load_toml_str(LOCALIZED_LAYOUT).unwrap();

    let artifact = UiDocumentCompiler::default()
        .compile_package_artifact(&document, UiCompiledAssetPackageProfile::Editor)
        .unwrap();
    let bytes = artifact.to_bytes().unwrap();
    let manifest = compiled_asset_package_manifest_from_artifact_bytes(&artifact, &bytes);
    let imported =
        UiCompiledAssetPackageManifest::import_toml(&manifest.write_toml().unwrap()).unwrap();

    assert_eq!(artifact.report.localization_report.dependencies.len(), 1);
    assert_eq!(
        artifact
            .report
            .localization_report
            .extraction_candidates
            .len(),
        2
    );
    assert_eq!(manifest.dependencies.localization_dependencies.len(), 1);
    assert_eq!(
        imported.dependencies.localization_dependencies[0]
            .reference
            .key,
        "editor.localization.title"
    );
}

#[test]
fn localization_resolver_reports_missing_tables_and_keys_for_selected_locale() {
    let document = UiAssetLoader::load_toml_str(LOCALIZED_LAYOUT).unwrap();
    let report = collect_document_localization_report(&document);

    let missing_table =
        validate_localization_report_against_catalog(&report, "en-US", &Default::default());

    assert_eq!(missing_table.len(), 1);
    assert_eq!(missing_table[0].code, "missing_locale_table");
    assert_eq!(
        missing_table[0].severity,
        UiLocalizationDiagnosticSeverity::Error
    );
    assert_eq!(missing_table[0].path, "nodes.root.props.text");
    assert!(missing_table[0].message.contains("en-US/editor"));

    let mut catalog = UiLocalizationTableCatalog::default();
    catalog.register_table_keys(
        "en-US",
        "editor",
        Some("res://locales/en-US/editor.toml".to_string()),
        ["editor.localization.subtitle"],
    );

    let missing_key = validate_localization_report_against_catalog(&report, "en-US", &catalog);

    assert_eq!(missing_key.len(), 1);
    assert_eq!(missing_key[0].code, "missing_locale_key");
    assert_eq!(
        missing_key[0].severity,
        UiLocalizationDiagnosticSeverity::Warning
    );
    assert!(missing_key[0].message.contains("editor.localization.title"));
    assert!(missing_key[0]
        .message
        .contains("res://locales/en-US/editor.toml"));
}

#[test]
fn localization_table_toml_keys_are_flattened_for_dotted_key_lookup() {
    let keys = localization_table_keys_from_toml_str(
        r#"
"editor.localization.title" = "Localization"

[menu]
play = "Play"
stop = "Stop"
"#,
    )
    .unwrap();

    assert!(keys.contains("editor.localization.title"));
    assert!(keys.contains("menu.play"));
    assert!(keys.contains("menu.stop"));
}
