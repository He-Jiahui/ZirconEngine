use crate::ui::component::UiComponentDescriptorRegistry;
use crate::ui::template::{
    UiAssetCompileCache, UiAssetLoader, UiCompiledArtifactKey, UiCompiledArtifactStore,
    UiDocumentCompiler, BROAD_SELECTOR_WARNING_THRESHOLD,
};
use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiDefaultNodeTemplate,
};
use zircon_runtime_interface::ui::template::{
    UiAssetChange, UiCompiledAssetPackageProfile, UiInvalidationStage,
};

const SIMPLE_LAYOUT_A: &str = r#"
[asset]
kind = "layout"
id = "editor.compile_cache"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "A" }
"#;

const SIMPLE_LAYOUT_B: &str = r#"
[asset]
kind = "layout"
id = "editor.compile_cache"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "B" }
"#;

const CARD_WIDGET_V1: &str = r##"
[asset]
kind = "widget"
id = "ui.cache.card"
version = 1

[root]
node_id = "widget_preview_root"
kind = "native"
type = "VerticalBox"

[components.Card]

[components.Card.contract]
api_version = "1.0.0"

[components.Card.root]
node_id = "card_root"
kind = "native"
type = "VerticalBox"
"##;

const CARD_WIDGET_V2: &str = r##"
[asset]
kind = "widget"
id = "ui.cache.card"
version = 1

[root]
node_id = "widget_preview_root"
kind = "native"
type = "VerticalBox"

[components.Card]

[components.Card.contract]
api_version = "1.1.0"

[components.Card.root]
node_id = "card_root"
kind = "native"
type = "VerticalBox"
"##;

const CARD_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.card_cache"
version = 1

[imports]
widgets = ["asset://ui/cache/card.ui#Card"]

[root]
node_id = "root"
kind = "reference"
component_ref = "asset://ui/cache/card.ui#Card"
component_api_version = "1.0.0"
"##;

const STYLE_LAYOUT: &str = r#"
[asset]
kind = "layout"
id = "editor.style_cache"
version = 1

[imports]
styles = ["asset://ui/cache/style.ui"]

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "Styled" }
"#;

const STYLE_ASSET_A: &str = r#"
[asset]
kind = "style"
id = "ui.cache.style"
version = 1

[[stylesheets]]
id = "cache_style"

[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "A" } }
"#;

const STYLE_ASSET_B: &str = r#"
[asset]
kind = "style"
id = "ui.cache.style"
version = 1

[[stylesheets]]
id = "cache_style"

[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "B" } }
"#;

const UNUSED_WIDGET_IMPORT_LAYOUT_A: &str = r##"
[asset]
kind = "layout"
id = "editor.unused_widget_cache"
version = 1

[imports]
widgets = ["asset://ui/cache/card.ui#Card"]

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "Unused Widget Import" }
"##;

const UNUSED_WIDGET_IMPORT_LAYOUT_B: &str = r##"
[asset]
kind = "layout"
id = "editor.unused_widget_cache"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "Unused Widget Import" }
"##;

const UNUSED_STYLE_IMPORT_LAYOUT_A: &str = r##"
[asset]
kind = "layout"
id = "editor.unused_style_cache"
version = 1

[imports]
styles = ["asset://ui/cache/style.ui"]

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "Unused Style Import" }
"##;

const UNUSED_STYLE_IMPORT_LAYOUT_B: &str = r##"
[asset]
kind = "layout"
id = "editor.unused_style_cache"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "Unused Style Import" }
"##;

const RESOURCE_LAYOUT_A: &str = r##"
[asset]
kind = "layout"
id = "editor.resource_cache"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { image = "asset://images/cache-a.png" }
"##;

const RESOURCE_LAYOUT_B: &str = r##"
[asset]
kind = "layout"
id = "editor.resource_cache"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { image = "asset://images/cache-b.png" }
"##;

const MISSING_ROOT_WITH_INVALID_RESOURCE_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.cache_invalid_shape"
version = 3

[imports]
resources = [
  { kind = "image", uri = "res://images/self.png", fallback = { mode = "placeholder", uri = "res://images/self.png" } },
]
"##;

#[test]
fn asset_compile_cache_reuses_exact_document_import_registry_and_contract_key() {
    let document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_A).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut cache = UiAssetCompileCache::new();

    let first = compiler.compile_with_cache(&document, &mut cache).unwrap();
    let second = compiler.compile_with_cache(&document, &mut cache).unwrap();

    assert!(!first.cache_hit);
    assert!(second.cache_hit);
    assert!(second.invalidation_report.stages.is_empty());
    assert_eq!(cache.len(), 1);
}

#[test]
fn asset_compile_cache_misses_when_document_fingerprint_changes() {
    let first_document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_A).unwrap();
    let second_document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_B).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut cache = UiAssetCompileCache::new();

    compiler
        .compile_with_cache(&first_document, &mut cache)
        .unwrap();
    let second = compiler
        .compile_with_cache(&second_document, &mut cache)
        .unwrap();

    assert!(!second.cache_hit);
    assert!(second
        .invalidation_report
        .changes
        .contains(&UiAssetChange::Document));
    assert!(second
        .invalidation_report
        .stages
        .contains(&UiInvalidationStage::DocumentShape));
}

#[test]
fn asset_compile_cache_misses_when_imported_component_contract_changes() {
    let layout = UiAssetLoader::load_toml_str(CARD_LAYOUT).unwrap();
    let widget_v1 = UiAssetLoader::load_toml_str(CARD_WIDGET_V1).unwrap();
    let widget_v2 = UiAssetLoader::load_toml_str(CARD_WIDGET_V2).unwrap();
    let mut first_compiler = UiDocumentCompiler::default();
    first_compiler
        .register_widget_import("asset://ui/cache/card.ui#Card", widget_v1)
        .unwrap();
    let mut second_compiler = UiDocumentCompiler::default();
    second_compiler
        .register_widget_import("asset://ui/cache/card.ui#Card", widget_v2)
        .unwrap();
    let mut cache = UiAssetCompileCache::new();

    first_compiler
        .compile_with_cache(&layout, &mut cache)
        .unwrap();
    let second = second_compiler
        .compile_with_cache(&layout, &mut cache)
        .unwrap();

    assert!(!second.cache_hit);
    assert!(second
        .invalidation_report
        .changes
        .contains(&UiAssetChange::WidgetImport));
    assert!(second
        .invalidation_report
        .changes
        .contains(&UiAssetChange::ComponentContract));
    assert!(second
        .invalidation_report
        .stages
        .contains(&UiInvalidationStage::ComponentContract));
}

#[test]
fn asset_compile_cache_misses_when_style_import_fingerprint_changes() {
    let layout = UiAssetLoader::load_toml_str(STYLE_LAYOUT).unwrap();
    let style_a = UiAssetLoader::load_toml_str(STYLE_ASSET_A).unwrap();
    let style_b = UiAssetLoader::load_toml_str(STYLE_ASSET_B).unwrap();
    let mut first_compiler = UiDocumentCompiler::default();
    first_compiler
        .register_style_import("asset://ui/cache/style.ui", style_a)
        .unwrap();
    let mut second_compiler = UiDocumentCompiler::default();
    second_compiler
        .register_style_import("asset://ui/cache/style.ui", style_b)
        .unwrap();
    let mut cache = UiAssetCompileCache::new();

    first_compiler
        .compile_with_cache(&layout, &mut cache)
        .unwrap();
    let second = second_compiler
        .compile_with_cache(&layout, &mut cache)
        .unwrap();

    assert!(!second.cache_hit);
    assert!(second
        .invalidation_report
        .changes
        .contains(&UiAssetChange::StyleImport));
    assert!(second
        .invalidation_report
        .stages
        .contains(&UiInvalidationStage::StyleValue));
}

#[test]
fn asset_compile_cache_misses_when_declared_widget_import_list_changes() {
    let first_document = UiAssetLoader::load_toml_str(UNUSED_WIDGET_IMPORT_LAYOUT_A).unwrap();
    let second_document = UiAssetLoader::load_toml_str(UNUSED_WIDGET_IMPORT_LAYOUT_B).unwrap();
    let widget = UiAssetLoader::load_toml_str(CARD_WIDGET_V1).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/cache/card.ui#Card", widget)
        .unwrap();
    let mut cache = UiAssetCompileCache::new();

    compiler
        .compile_with_cache(&first_document, &mut cache)
        .unwrap();
    let second = compiler
        .compile_with_cache(&second_document, &mut cache)
        .unwrap();

    assert!(!second.cache_hit);
    assert!(second
        .invalidation_report
        .changes
        .contains(&UiAssetChange::WidgetImport));
    assert!(second
        .invalidation_report
        .stages
        .contains(&UiInvalidationStage::ImportGraph));
}

#[test]
fn asset_compile_cache_misses_when_declared_style_import_list_changes() {
    let first_document = UiAssetLoader::load_toml_str(UNUSED_STYLE_IMPORT_LAYOUT_A).unwrap();
    let second_document = UiAssetLoader::load_toml_str(UNUSED_STYLE_IMPORT_LAYOUT_B).unwrap();
    let style = UiAssetLoader::load_toml_str(STYLE_ASSET_A).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/cache/style.ui", style)
        .unwrap();
    let mut cache = UiAssetCompileCache::new();

    compiler
        .compile_with_cache(&first_document, &mut cache)
        .unwrap();
    let second = compiler
        .compile_with_cache(&second_document, &mut cache)
        .unwrap();

    assert!(!second.cache_hit);
    assert!(second
        .invalidation_report
        .changes
        .contains(&UiAssetChange::StyleImport));
    assert!(second
        .invalidation_report
        .stages
        .contains(&UiInvalidationStage::ImportGraph));
}

#[test]
fn asset_compile_cache_misses_when_resource_dependencies_change() {
    let first_document = UiAssetLoader::load_toml_str(RESOURCE_LAYOUT_A).unwrap();
    let second_document = UiAssetLoader::load_toml_str(RESOURCE_LAYOUT_B).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut cache = UiAssetCompileCache::new();

    compiler
        .compile_with_cache(&first_document, &mut cache)
        .unwrap();
    let second = compiler
        .compile_with_cache(&second_document, &mut cache)
        .unwrap();

    assert!(!second.cache_hit);
    assert!(second
        .invalidation_report
        .changes
        .contains(&UiAssetChange::ResourceDependency));
    assert!(second
        .invalidation_report
        .stages
        .contains(&UiInvalidationStage::ResourceDependency));
}

#[test]
fn asset_compile_cache_prioritizes_shape_errors_before_resource_fingerprints() {
    let document = UiAssetLoader::load_toml_str(MISSING_ROOT_WITH_INVALID_RESOURCE_LAYOUT).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut cache = UiAssetCompileCache::new();

    let error = compiler
        .compile_with_cache(&document, &mut cache)
        .expect_err("cache path must preserve compiler precondition errors");

    assert!(
        error.to_string().contains("layout/widget assets require"),
        "unexpected error: {error:?}"
    );
    assert_eq!(cache.len(), 0);
}

#[test]
fn asset_compile_cache_reports_misses_against_the_same_asset_snapshot() {
    let first_document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_A).unwrap();
    let second_document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_B).unwrap();
    let interleaved_layout = UiAssetLoader::load_toml_str(STYLE_LAYOUT).unwrap();
    let interleaved_style = UiAssetLoader::load_toml_str(STYLE_ASSET_A).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut interleaved_compiler = UiDocumentCompiler::default();
    interleaved_compiler
        .register_style_import("asset://ui/cache/style.ui", interleaved_style)
        .unwrap();
    let mut cache = UiAssetCompileCache::new();

    compiler
        .compile_with_cache(&first_document, &mut cache)
        .unwrap();
    interleaved_compiler
        .compile_with_cache(&interleaved_layout, &mut cache)
        .unwrap();
    let second = compiler
        .compile_with_cache(&second_document, &mut cache)
        .unwrap();

    assert!(!second.cache_hit);
    assert_eq!(
        second.invalidation_report.changes,
        [UiAssetChange::Document]
    );
}

#[test]
fn asset_compile_cache_misses_when_descriptor_registry_revision_changes() {
    let document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_A).unwrap();
    let first_compiler = UiDocumentCompiler::default();
    let mut registry = UiComponentDescriptorRegistry::editor_showcase();
    registry
        .register(
            UiComponentDescriptor::new(
                "CacheOnlyWidget",
                "Cache Only Widget",
                UiComponentCategory::Visual,
                "cache-only",
            )
            .default_node_template(UiDefaultNodeTemplate::native("CacheOnlyWidget")),
        )
        .unwrap();
    let second_compiler = UiDocumentCompiler::default().with_component_registry(registry);
    let mut cache = UiAssetCompileCache::new();

    first_compiler
        .compile_with_cache(&document, &mut cache)
        .unwrap();
    let second = second_compiler
        .compile_with_cache(&document, &mut cache)
        .unwrap();

    assert!(!second.cache_hit);
    assert!(second
        .invalidation_report
        .changes
        .contains(&UiAssetChange::DescriptorRegistry));
    assert!(second
        .invalidation_report
        .stages
        .contains(&UiInvalidationStage::DescriptorRegistry));
}

#[test]
fn asset_compile_cache_reports_diagnostics_on_miss() {
    let first_document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_A).unwrap();
    let second_document = UiAssetLoader::load_toml_str(&broad_selector_layout()).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut cache = UiAssetCompileCache::new();

    compiler
        .compile_with_cache(&first_document, &mut cache)
        .unwrap();
    let second = compiler
        .compile_with_cache(&second_document, &mut cache)
        .unwrap();

    assert!(!second.cache_hit);
    assert!(second
        .invalidation_report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "broad_selector"));
}

#[test]
fn persistent_cache_round_trips_compiled_artifact_with_fingerprint_key() {
    let temp_dir = persistent_cache_temp_dir("roundtrip");
    let document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_A).unwrap();
    let compiler = UiDocumentCompiler::default();
    let store = UiCompiledArtifactStore::new(temp_dir.clone());
    let artifact = compiler
        .compile_package_artifact(&document, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let key = UiCompiledArtifactKey::from_artifact(&artifact);
    let expected_fingerprint = UiCompiledArtifactKey::fingerprint_compile_cache_key(
        &artifact.report.header.compile_cache_key,
    );

    let artifact_path = store.store(&key, &artifact).unwrap();
    let loaded = store.load(&key).unwrap().unwrap();
    let loaded_bytes = store.load_bytes(&key).unwrap().unwrap();

    assert_eq!(key.asset_id, "editor.compile_cache");
    assert_eq!(key.fingerprint, expected_fingerprint);
    assert!(artifact_path.exists());
    assert_eq!(loaded, artifact);
    assert_eq!(loaded_bytes, artifact.to_bytes().unwrap());

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn persistent_cache_misses_stale_schema_compiler_and_fingerprint_keys() {
    let temp_dir = persistent_cache_temp_dir("stale_keys");
    let document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_A).unwrap();
    let compiler = UiDocumentCompiler::default();
    let store = UiCompiledArtifactStore::new(temp_dir.clone());
    let artifact = compiler
        .compile_package_artifact(&document, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let key = UiCompiledArtifactKey::from_artifact(&artifact);
    store.store(&key, &artifact).unwrap();

    let stale_schema = UiCompiledArtifactKey::new(
        key.asset_id.clone(),
        key.fingerprint,
        key.schema_version + 1,
        key.compiler_version,
    );
    let stale_compiler = UiCompiledArtifactKey::new(
        key.asset_id.clone(),
        key.fingerprint,
        key.schema_version,
        key.compiler_version + 1,
    );
    let stale_fingerprint = UiCompiledArtifactKey::new(
        key.asset_id.clone(),
        key.fingerprint.wrapping_add(1),
        key.schema_version,
        key.compiler_version,
    );

    assert!(store.load(&stale_schema).unwrap().is_none());
    assert!(store.load(&stale_compiler).unwrap().is_none());
    assert!(store.load(&stale_fingerprint).unwrap().is_none());
    assert!(store.load(&key).unwrap().is_some());

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn persistent_cache_treats_corrupt_records_as_misses() {
    let temp_dir = persistent_cache_temp_dir("corrupt_record");
    let document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_A).unwrap();
    let compiler = UiDocumentCompiler::default();
    let store = UiCompiledArtifactStore::new(temp_dir.clone());
    let artifact = compiler
        .compile_package_artifact(&document, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let key = UiCompiledArtifactKey::from_artifact(&artifact);
    let artifact_path = store.store(&key, &artifact).unwrap();

    std::fs::write(&artifact_path, b"not a persistent ui cache record").unwrap();

    assert!(store.load(&key).unwrap().is_none());
    assert!(store.load_bytes(&key).unwrap().is_none());

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn persistent_cache_evicts_all_versions_for_asset_id() {
    let temp_dir = persistent_cache_temp_dir("evict_asset");
    let first_document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_A).unwrap();
    let second_document = UiAssetLoader::load_toml_str(SIMPLE_LAYOUT_B).unwrap();
    let compiler = UiDocumentCompiler::default();
    let store = UiCompiledArtifactStore::new(temp_dir.clone());
    let first_artifact = compiler
        .compile_package_artifact(&first_document, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let second_artifact = compiler
        .compile_package_artifact(&second_document, UiCompiledAssetPackageProfile::Runtime)
        .unwrap();
    let first_key = UiCompiledArtifactKey::from_artifact(&first_artifact);
    let second_key = UiCompiledArtifactKey::from_artifact(&second_artifact);
    store.store(&first_key, &first_artifact).unwrap();
    store.store(&second_key, &second_artifact).unwrap();

    let report = store.evict_asset("editor.compile_cache").unwrap();

    assert_eq!(report.files_removed, 2);
    assert!(report.bytes_removed > 0);
    assert!(store.load(&first_key).unwrap().is_none());
    assert!(store.load(&second_key).unwrap().is_none());

    let _ = std::fs::remove_dir_all(temp_dir);
}

fn broad_selector_layout() -> String {
    let mut source = String::from(
        r#"
[asset]
kind = "layout"
id = "editor.broad_cache"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"

[[stylesheets]]
id = "cache_diagnostics"
"#,
    );
    for _ in 0..BROAD_SELECTOR_WARNING_THRESHOLD {
        source.push_str(
            r#"
[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "Diagnostic" } }
"#,
        );
    }
    source
}

fn persistent_cache_temp_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zircon_ui_persistent_cache_{test_name}_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn template_asset_hot_paths_cache_miss_does_not_repeat_full_precondition_validation() {
    let source = include_str!("../template/asset/compiler/compile.rs");
    let cached_compile = source
        .split_once("pub fn compile_with_cache")
        .expect("compile_with_cache must remain available")
        .1;

    assert!(
        cached_compile.contains("self.compile_validated(document)?"),
        "a cache miss must enter the already-validated compile path"
    );
    assert!(
        !cached_compile.contains("self.compile(document)?"),
        "a cache miss must not repeat the full compiler precondition scan"
    );
}

#[test]
fn template_asset_hot_paths_package_does_not_repeat_full_precondition_validation() {
    let source = include_str!("../template/asset/compiler/package/validate.rs");

    assert!(
        source.contains("self.compile_validated(document)?"),
        "package compilation must reuse its already-completed precondition validation"
    );
    assert!(
        !source.contains("self.compile(document)?"),
        "package compilation must not repeat the full compiler precondition scan"
    );
}

#[test]
fn template_asset_hot_paths_tree_validation_borrows_nodes_instead_of_cloning_subtrees() {
    let source = include_str!("../template/asset/document/validation.rs");

    assert!(
        source.contains("BTreeMap<&'a str, &'a UiNodeDefinition>"),
        "tree authority validation must retain borrowed node identities"
    );
    assert!(
        !source.contains("seen.insert(node.node_id.clone(), node.clone())"),
        "tree authority validation must not clone every visited subtree"
    );
}
