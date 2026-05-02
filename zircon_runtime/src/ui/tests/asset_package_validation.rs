use crate::ui::template::{
    fingerprint_document, UiAssetLoader, UiCompileCacheKey, UiCompiledAssetArtifact,
    UiCompiledAssetPackageManifest, UiDocumentCompiler,
};
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetFingerprint, UiAssetKind, UiCompiledAssetPackageProfile,
    UiCompiledAssetPackageSection, UiInvalidationStage, UiResourceDependencySource,
    UiResourceFallbackMode, UiResourceKind, UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION,
    UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION, UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION,
    UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION,
};

const COMPILED_ASSET_BINARY_MAGIC: &[u8; 8] = b"ZRUIA016";
const COMPILED_ASSET_BINARY_HEADER_LEN: usize = COMPILED_ASSET_BINARY_MAGIC.len() + 4 + 8;

const PACKAGE_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.package_validation"
version = 3

[imports]
widgets = [
  "asset://ui/package/button.ui#Button",
  "asset://ui/package/card.ui#Card",
]
styles = ["asset://ui/package/theme.ui"]
resources = [
  { kind = "font", uri = "res://fonts/package.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
  { kind = "image", uri = "asset://images/package-logo.png", fallback = { mode = "optional" } },
]

[root]
node_id = "root"
kind = "native"
type = "Label"
control_id = "PackageRoot"
props = { text = "Package" }
"##;

const PACKAGE_CARD_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.package.card"
version = 3

[root]
node_id = "card_preview_root"
kind = "native"
type = "VerticalBox"

[components.Card]

[components.Card.root]
node_id = "card_root"
kind = "native"
type = "Label"
control_id = "CardRoot"
props = { text = "Card" }
"##;

const PACKAGE_BUTTON_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.package.button"
version = 3

[root]
node_id = "button_preview_root"
kind = "native"
type = "VerticalBox"

[components.Button]

[components.Button.root]
node_id = "button_root"
kind = "native"
type = "Button"
control_id = "ButtonRoot"
props = { text = "Button" }
"##;

const PACKAGE_STYLE: &str = r##"
[asset]
kind = "style"
id = "ui.package.theme"
version = 3

[[stylesheets]]
id = "package_theme"

[[stylesheets.rules]]
selector = "Label"
set = { self = { background_image = "asset://images/package-theme-bg.png" } }
"##;

const PRIVATE_CARD_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.package.private_card"
version = 3

[root]
node_id = "private_card_preview_root"
kind = "native"
type = "VerticalBox"

[components.Card]

[components.Card.root]
node_id = "card_root"
kind = "native"
type = "VerticalBox"

[[components.Card.root.children]]
[components.Card.root.children.node]
node_id = "card_secret"
kind = "native"
type = "Label"
control_id = "SecretLabel"
"##;

const PRIVATE_SELECTOR_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.private_selector_package"
version = 3

[imports]
widgets = ["asset://ui/package/private_card.ui#Card"]

[root]
node_id = "root"
kind = "reference"
component_ref = "asset://ui/package/private_card.ui#Card"

[[stylesheets]]
id = "host_styles"

[[stylesheets.rules]]
selector = "#SecretLabel"
set = { self = { text = "Leaked" } }
"##;

const PRIVATE_SELECTOR_WITH_INVALID_RESOURCE_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.private_selector_package"
version = 3

[imports]
widgets = ["asset://ui/package/private_card.ui#Card"]
resources = [
  { kind = "image", uri = "res://images/self.png", fallback = { mode = "placeholder", uri = "res://images/self.png" } },
]

[root]
node_id = "root"
kind = "reference"
component_ref = "asset://ui/package/private_card.ui#Card"

[[stylesheets]]
id = "host_styles"

[[stylesheets.rules]]
selector = "#SecretLabel"
set = { self = { text = "Leaked" } }
"##;

#[test]
fn asset_package_validation_header_records_compiled_artifact_cache_inputs() {
    let layout = package_layout();
    let compiler = package_compiler();

    let report = compiler
        .validate_package(&layout, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let expected_root_fingerprint = fingerprint_document(&layout).unwrap();

    assert_eq!(report.header.asset.id, "editor.package_validation");
    assert_eq!(report.header.asset.kind, UiAssetKind::Layout);
    assert_eq!(
        report.header.source_schema_version,
        UI_ASSET_CURRENT_SOURCE_SCHEMA_VERSION
    );
    assert!(report.header.is_current_source_schema());
    assert_eq!(
        report.header.compiler_schema_version,
        UI_COMPILED_ASSET_COMPILER_SCHEMA_VERSION
    );
    assert_eq!(
        report.header.package_schema_version,
        UI_COMPILED_ASSET_PACKAGE_SCHEMA_VERSION
    );
    assert_eq!(
        report.header.root_document_fingerprint,
        expected_root_fingerprint
    );
    assert_eq!(
        report.header.compile_cache_key.root_document,
        expected_root_fingerprint
    );
    assert_eq!(
        report.header.descriptor_registry_revision,
        report.header.compile_cache_key.descriptor_registry_revision
    );
    assert_eq!(
        report.header.component_contract_revision,
        report.header.compile_cache_key.component_contract_revision
    );
}

#[test]
fn asset_package_validation_dependency_manifest_matches_cache_key_in_stable_order() {
    let layout = package_layout();
    let compiler = package_compiler();

    let report = compiler
        .validate_package(&layout, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();

    let widget_references = report
        .dependencies
        .widget_imports
        .iter()
        .map(|dependency| dependency.reference.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        widget_references,
        vec![
            "asset://ui/package/button.ui#Button",
            "asset://ui/package/card.ui#Card"
        ]
    );
    assert_eq!(
        report.dependencies.widget_imports[0].asset_id,
        "ui.package.button"
    );
    assert_eq!(
        report.dependencies.widget_imports[0].asset_kind,
        UiAssetKind::Widget
    );
    assert_eq!(
        report.dependencies.widget_imports[0].fingerprint,
        report.header.compile_cache_key.widget_imports["asset://ui/package/button.ui#Button"]
    );
    assert_eq!(
        report.dependencies.widget_imports[1].fingerprint,
        report.header.compile_cache_key.widget_imports["asset://ui/package/card.ui#Card"]
    );

    assert_eq!(report.dependencies.style_imports.len(), 1);
    assert_eq!(
        report.dependencies.style_imports[0].reference,
        "asset://ui/package/theme.ui"
    );
    assert_eq!(
        report.dependencies.style_imports[0].asset_kind,
        UiAssetKind::Style
    );
    assert_eq!(
        report.dependencies.style_imports[0].fingerprint,
        report.header.compile_cache_key.style_imports["asset://ui/package/theme.ui"]
    );

    assert_eq!(report.dependencies.resource_dependencies.len(), 3);
    assert_eq!(
        report.dependencies.resource_dependencies[0].source,
        UiResourceDependencySource::DocumentImport
    );
    assert_eq!(
        report.dependencies.resource_dependencies[0].path,
        "imports.resources[0]"
    );
    assert_eq!(
        report.dependencies.resource_dependencies[0].reference.kind,
        UiResourceKind::Font
    );
    assert_eq!(
        report.dependencies.resource_dependencies[0].reference.uri,
        "res://fonts/package.font.toml"
    );
    assert_eq!(
        report.dependencies.resource_dependencies[0]
            .reference
            .fallback
            .mode,
        UiResourceFallbackMode::Placeholder
    );
    assert_eq!(
        report.dependencies.resource_dependencies[1].path,
        "imports.resources[1]"
    );
    assert_eq!(
        report.dependencies.resource_dependencies[1].reference.kind,
        UiResourceKind::Image
    );
    assert_eq!(
        report.dependencies.resource_dependencies[2].source,
        UiResourceDependencySource::ImportedStyle
    );
    assert_eq!(
        report.dependencies.resource_dependencies[2].reference.uri,
        "asset://images/package-theme-bg.png"
    );
}

#[test]
fn asset_package_binary_artifact_roundtrips_deterministic_envelope() {
    let layout = package_layout();
    let compiler = package_compiler();

    let artifact = compiler
        .compile_package_artifact(&layout, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let first = artifact.to_bytes().unwrap();
    let second = artifact.to_bytes().unwrap();
    let decoded = UiCompiledAssetArtifact::from_bytes(&first).unwrap();
    let schema_version = u32::from_le_bytes(
        first[COMPILED_ASSET_BINARY_MAGIC.len()..COMPILED_ASSET_BINARY_MAGIC.len() + 4]
            .try_into()
            .unwrap(),
    );
    let payload_len = u64::from_le_bytes(
        first[COMPILED_ASSET_BINARY_MAGIC.len() + 4..COMPILED_ASSET_BINARY_HEADER_LEN]
            .try_into()
            .unwrap(),
    );
    let payload = std::str::from_utf8(&first[COMPILED_ASSET_BINARY_HEADER_LEN..]).unwrap();
    let payload_artifact = toml::from_str::<UiCompiledAssetArtifact>(payload).unwrap();

    assert_eq!(first, second);
    assert_eq!(
        &first[..COMPILED_ASSET_BINARY_MAGIC.len()],
        COMPILED_ASSET_BINARY_MAGIC
    );
    assert_eq!(
        schema_version,
        UI_COMPILED_ASSET_BINARY_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(payload_len, payload.len() as u64);
    assert!(payload.starts_with("[report]\n"));
    assert!(payload.contains("[compiled.root]\n"));
    assert!(payload.contains("id = \"editor.package_validation\""));
    assert!(payload.contains("control_id = \"PackageRoot\""));
    assert_eq!(payload_artifact, artifact);
    assert_eq!(decoded, artifact);
    assert_eq!(decoded.report.header.asset.id, "editor.package_validation");
    assert_eq!(
        decoded.compiled.root.control_id.as_deref(),
        Some("PackageRoot")
    );
}

#[test]
fn asset_package_cache_record_reuses_cache_key_and_invalidation_snapshot() {
    let layout = package_layout();
    let compiler = package_compiler();

    let artifact = compiler
        .compile_package_artifact(&layout, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let bytes = artifact.to_bytes().unwrap();
    let manifest = UiCompiledAssetPackageManifest::from_artifact_bytes(&artifact, &bytes);

    assert_eq!(
        manifest.cache.cache_key,
        artifact.report.header.compile_cache_key
    );
    assert_eq!(
        manifest.cache.invalidation_snapshot,
        artifact
            .report
            .header
            .compile_cache_key
            .invalidation_snapshot()
    );
    assert_eq!(
        manifest.cache.artifact_fingerprint,
        UiAssetFingerprint::from_bytes(&bytes)
    );
    assert_eq!(manifest.cache.artifact_byte_len, bytes.len() as u64);
}

#[test]
fn asset_package_manifest_writer_importer_preserves_resource_dependencies() {
    let layout = package_layout();
    let compiler = package_compiler();

    let artifact = compiler
        .compile_package_artifact(&layout, UiCompiledAssetPackageProfile::Editor)
        .unwrap();
    let bytes = artifact.to_bytes().unwrap();
    let manifest = UiCompiledAssetPackageManifest::from_artifact_bytes(&artifact, &bytes);
    let source = manifest.write_toml().unwrap();
    let imported = UiCompiledAssetPackageManifest::import_toml(&source).unwrap();

    assert_eq!(imported, manifest);
    assert_eq!(imported.dependencies.resource_dependencies.len(), 3);
    assert!(imported.dependencies.localization_dependencies.is_empty());
    assert_eq!(
        imported.dependencies.resource_dependencies[0].reference.uri,
        "res://fonts/package.font.toml"
    );
    assert_eq!(
        imported.dependencies.resource_dependencies[1].reference.uri,
        "asset://images/package-logo.png"
    );
    assert_eq!(
        imported.dependencies.resource_dependencies[2].reference.uri,
        "asset://images/package-theme-bg.png"
    );
    assert_eq!(imported.artifact.byte_len, bytes.len() as u64);
    assert_eq!(
        imported.artifact.fingerprint,
        UiAssetFingerprint::from_bytes(&bytes)
    );
}

#[test]
fn asset_package_validation_profiles_report_runtime_and_editor_stripping() {
    let layout = package_layout();
    let compiler = package_compiler();

    let runtime_report = compiler
        .validate_package(&layout, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let editor_report = compiler
        .validate_package(&layout, UiCompiledAssetPackageProfile::Editor)
        .unwrap();

    assert!(runtime_report
        .retained_sections
        .contains(&UiCompiledAssetPackageSection::RuntimeTemplateTree));
    assert!(runtime_report
        .stripped_sections
        .contains(&UiCompiledAssetPackageSection::SourceDocument));
    assert!(runtime_report
        .stripped_sections
        .contains(&UiCompiledAssetPackageSection::AuthoringDiagnostics));
    assert!(runtime_report.action_policy_report.is_allowed());

    assert!(editor_report
        .retained_sections
        .contains(&UiCompiledAssetPackageSection::SourceDocument));
    assert!(editor_report.stripped_sections.is_empty());
}

#[test]
fn asset_package_validation_preserves_existing_compiler_error_paths() {
    let widget = UiAssetLoader::load_toml_str(PRIVATE_CARD_WIDGET).unwrap();
    let layout = UiAssetLoader::load_toml_str(PRIVATE_SELECTOR_LAYOUT).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/package/private_card.ui#Card", widget)
        .unwrap();

    let error = compiler
        .validate_package(&layout, UiCompiledAssetPackageProfile::Runtime)
        .expect_err("package validation must not bypass component contract errors");

    assert!(
        error
            .to_string()
            .contains("private component internals SecretLabel"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn asset_package_validation_prioritizes_contract_errors_before_resource_fingerprints() {
    let widget = UiAssetLoader::load_toml_str(PRIVATE_CARD_WIDGET).unwrap();
    let layout =
        UiAssetLoader::load_toml_str(PRIVATE_SELECTOR_WITH_INVALID_RESOURCE_LAYOUT).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/package/private_card.ui#Card", widget)
        .unwrap();

    let error = compiler
        .validate_package(&layout, UiCompiledAssetPackageProfile::Runtime)
        .expect_err("package validation must preserve compiler precondition errors");

    assert!(
        error
            .to_string()
            .contains("private component internals SecretLabel"),
        "unexpected error: {error:?}"
    );
}

#[test]
fn asset_package_validation_report_generation_keeps_compiler_cache_inputs_stable() {
    let layout = package_layout();
    let compiler = package_compiler();
    let before = UiCompileCacheKey::from_compiler(&compiler, &layout).unwrap();

    let report = compiler
        .validate_package(&layout, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let after = UiCompileCacheKey::from_compiler(&compiler, &layout).unwrap();

    assert_eq!(before, after);
    assert_eq!(
        report.header.compile_cache_key.root_document,
        before.root_document
    );
    assert_eq!(
        report.header.compile_cache_key.widget_imports,
        before.widget_imports
    );
    assert_eq!(
        report.header.compile_cache_key.style_imports,
        before.style_imports
    );
    assert_eq!(
        report.header.compile_cache_key.descriptor_registry_revision,
        before.descriptor_registry_revision
    );
    assert_eq!(
        report.header.compile_cache_key.component_contract_revision,
        before.component_contract_revision
    );
    assert_eq!(
        report
            .header
            .compile_cache_key
            .resource_dependencies_revision,
        before.resource_dependencies_revision
    );
    assert!(report
        .invalidation_report
        .stages
        .contains(&UiInvalidationStage::SourceParse));
    assert!(report
        .invalidation_report
        .stages
        .contains(&UiInvalidationStage::DocumentShape));
}

fn package_layout() -> UiAssetDocument {
    UiAssetLoader::load_toml_str(PACKAGE_LAYOUT).unwrap()
}

fn package_compiler() -> UiDocumentCompiler {
    let card = UiAssetLoader::load_toml_str(PACKAGE_CARD_WIDGET).unwrap();
    let button = UiAssetLoader::load_toml_str(PACKAGE_BUTTON_WIDGET).unwrap();
    let style = UiAssetLoader::load_toml_str(PACKAGE_STYLE).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/package/card.ui#Card", card)
        .unwrap()
        .register_widget_import("asset://ui/package/button.ui#Button", button)
        .unwrap()
        .register_style_import("asset://ui/package/theme.ui", style)
        .unwrap();
    compiler
}
