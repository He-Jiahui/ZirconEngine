use crate::ui::component::UiComponentDescriptorRegistry;
use crate::ui::template::{
    UiAssetCompileCache, UiAssetLoader, UiDocumentCompiler, BROAD_SELECTOR_WARNING_THRESHOLD,
};
use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiDefaultNodeTemplate,
};
use zircon_runtime_interface::ui::template::{UiAssetChange, UiInvalidationStage};

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
