use crate::ui::template::{resource_dependencies_fingerprint, UiAssetLoader, UiDocumentCompiler};
use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiResourceDependencySource, UiResourceFallbackMode, UiResourceFallbackPolicy,
    UiResourceKind, UiResourceRef,
};

const RESOURCE_IMPORTS_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.resource_refs"
version = 1

[imports]
widgets = ["asset://ui/common/button.ui#Button"]
styles = ["asset://ui/theme/base.ui"]
resources = [
  { kind = "font", uri = "res://fonts/inter.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
  { kind = "image", uri = "asset://images/logo.png", fallback = { mode = "optional" } },
]
"##;

const RESOURCE_COLLECTION_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.resource_collection"
version = 3

[imports]
widgets = ["asset://ui/resource/card.ui#Card"]
styles = ["asset://ui/resource/theme.ui"]
resources = [
  { kind = "font", uri = "res://fonts/inter.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
]

[tokens]
hero_image = "asset://images/hero.png"

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { image = { kind = "image", uri = "asset://images/logo.png", fallback = { mode = "optional" } } }

[[stylesheets]]
id = "local"

[[stylesheets.rules]]
selector = "Label"
set = { self = { font = "res://fonts/local.font.toml" } }
"##;

const RESOURCE_COLLECTION_WIDGET: &str = r##"
[asset]
kind = "widget"
id = "ui.resource.card"
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
props = { icon = "asset://images/card-icon.png" }
"##;

const RESOURCE_COLLECTION_STYLE: &str = r##"
[asset]
kind = "style"
id = "ui.resource.theme"
version = 3

[[stylesheets]]
id = "theme"

[[stylesheets.rules]]
selector = "Label"
set = { self = { background_image = "asset://images/theme-bg.png" } }
"##;

#[test]
fn ui_asset_imports_resources_roundtrip_typed_refs() {
    let document = UiAssetLoader::load_toml_str(RESOURCE_IMPORTS_ASSET).unwrap();

    assert_eq!(document.imports.resources.len(), 2);
    assert_eq!(document.imports.resources[0].kind, UiResourceKind::Font);
    assert_eq!(
        document.imports.resources[0].uri,
        "res://fonts/inter.font.toml"
    );
    assert_eq!(
        document.imports.resources[0].fallback.mode,
        UiResourceFallbackMode::Placeholder
    );
    assert_eq!(
        document.imports.resources[0].fallback.uri.as_deref(),
        Some("res://fonts/system.ttf")
    );
    assert_eq!(document.imports.resources[1].kind, UiResourceKind::Image);
    assert_eq!(
        document.imports.resources[1].fallback.mode,
        UiResourceFallbackMode::Optional
    );

    let encoded = toml::to_string(&document).unwrap();
    let decoded: UiAssetDocument = toml::from_str(&encoded).unwrap();

    assert_eq!(decoded.imports.resources, document.imports.resources);
}

#[test]
fn ui_resource_ref_rejects_empty_uri() {
    let reference = UiResourceRef {
        kind: UiResourceKind::GenericAsset,
        uri: "  ".to_string(),
        fallback: UiResourceFallbackPolicy::default(),
    };

    let diagnostic = reference.validate("imports.resources[0]").unwrap_err();

    assert_eq!(diagnostic.code, "empty_resource_uri");
    assert_eq!(diagnostic.path, "imports.resources[0]");
}

#[test]
fn ui_resource_ref_rejects_unsupported_scheme() {
    let reference = UiResourceRef {
        kind: UiResourceKind::Image,
        uri: "http://cdn.example/logo.png".to_string(),
        fallback: UiResourceFallbackPolicy::default(),
    };

    let diagnostic = reference.validate("root.props.icon").unwrap_err();

    assert_eq!(diagnostic.code, "unsupported_resource_scheme");
}

#[test]
fn ui_resource_ref_rejects_placeholder_without_uri() {
    let reference = UiResourceRef {
        kind: UiResourceKind::Image,
        uri: "asset://images/logo.png".to_string(),
        fallback: UiResourceFallbackPolicy {
            mode: UiResourceFallbackMode::Placeholder,
            uri: None,
        },
    };

    let diagnostic = reference.validate("root.props.icon").unwrap_err();

    assert_eq!(diagnostic.code, "placeholder_fallback_missing_uri");
}

#[test]
fn ui_resource_ref_rejects_self_referential_placeholder() {
    let reference = UiResourceRef {
        kind: UiResourceKind::Image,
        uri: "asset://images/logo.png".to_string(),
        fallback: UiResourceFallbackPolicy {
            mode: UiResourceFallbackMode::Placeholder,
            uri: Some("asset://images/logo.png".to_string()),
        },
    };

    let diagnostic = reference.validate("root.props.icon").unwrap_err();

    assert_eq!(diagnostic.code, "placeholder_fallback_self_reference");
}

#[test]
fn ui_resource_ref_rejects_placeholder_kind_mismatch() {
    let reference = UiResourceRef {
        kind: UiResourceKind::Image,
        uri: "asset://images/logo.png".to_string(),
        fallback: UiResourceFallbackPolicy {
            mode: UiResourceFallbackMode::Placeholder,
            uri: Some("res://fonts/system.ttf".to_string()),
        },
    };

    let diagnostic = reference.validate("root.props.icon").unwrap_err();

    assert_eq!(diagnostic.code, "placeholder_fallback_kind_mismatch");
}

#[test]
fn ui_resource_kind_infers_from_property_path_before_extension() {
    let kind =
        UiResourceKind::infer_from_path_and_uri("root.props.icon", "asset://audio/click.mp3");

    assert_eq!(kind, UiResourceKind::Image);
}

#[test]
fn compiler_collects_explicit_node_style_and_imported_resource_refs() {
    let layout = UiAssetLoader::load_toml_str(RESOURCE_COLLECTION_LAYOUT).unwrap();
    let widget = UiAssetLoader::load_toml_str(RESOURCE_COLLECTION_WIDGET).unwrap();
    let style = UiAssetLoader::load_toml_str(RESOURCE_COLLECTION_STYLE).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/resource/card.ui#Card", widget)
        .unwrap()
        .register_style_import("asset://ui/resource/theme.ui", style)
        .unwrap();

    let compiled = compiler.compile(&layout).unwrap();
    let dependencies = compiled.resource_dependencies();
    let uris = dependencies
        .iter()
        .map(|dependency| dependency.reference.uri.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        uris,
        vec![
            "res://fonts/inter.font.toml",
            "res://fonts/local.font.toml",
            "asset://images/card-icon.png",
            "asset://images/hero.png",
            "asset://images/logo.png",
            "asset://images/theme-bg.png",
        ]
    );
    assert!(dependencies.iter().any(|dependency| {
        dependency.source == UiResourceDependencySource::ImportedWidget
            && dependency
                .path
                .contains("imported_widget:asset://ui/resource/card.ui#Card")
    }));
    assert!(dependencies.iter().any(|dependency| {
        dependency.source == UiResourceDependencySource::ImportedStyle
            && dependency
                .path
                .contains("imported_style:asset://ui/resource/theme.ui")
    }));
    assert!(compiled.resource_diagnostics().is_empty());
}

#[test]
fn resource_collector_ignores_localized_text_fallback_strings() {
    let layout = UiAssetLoader::load_toml_str(
        r##"
[asset]
kind = "layout"
id = "editor.resource_localized_text"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = { text_key = "editor.title", fallback = "Editor" } }
"##,
    )
    .unwrap();

    let compiled = UiDocumentCompiler::default().compile(&layout).unwrap();

    assert!(compiled.resource_dependencies().is_empty());
    assert!(compiled.resource_diagnostics().is_empty());
}

#[test]
fn resource_collector_ignores_non_resource_tables_with_kind_fields() {
    let layout = UiAssetLoader::load_toml_str(
        r##"
[asset]
kind = "layout"
id = "editor.resource_non_resource_kind_table"
version = 3

[root]
node_id = "root"
kind = "native"
type = "VerticalBox"
layout = { container = { kind = "VerticalBox", gap = 8.0 } }
"##,
    )
    .unwrap();

    let compiled = UiDocumentCompiler::default().compile(&layout).unwrap();

    assert!(compiled.resource_dependencies().is_empty());
    assert!(compiled.resource_diagnostics().is_empty());
}

#[test]
fn resource_dependency_fingerprint_ignores_source_path_order() {
    let first = UiAssetLoader::load_toml_str(
        r##"
[asset]
kind = "layout"
id = "editor.resource_fingerprint"
version = 3

[imports]
resources = [
  { kind = "image", uri = "asset://images/shared.png" },
]

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { icon = "asset://images/shared.png" }
"##,
    )
    .unwrap();
    let second = UiAssetLoader::load_toml_str(
        r##"
[asset]
kind = "layout"
id = "editor.resource_fingerprint"
version = 3

[imports]
resources = [
  { kind = "image", uri = "asset://images/shared.png" },
]

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { background_image = "asset://images/shared.png" }
"##,
    )
    .unwrap();

    let first_fingerprint =
        resource_dependencies_fingerprint(&first, &Default::default(), &Default::default())
            .unwrap();
    let second_fingerprint =
        resource_dependencies_fingerprint(&second, &Default::default(), &Default::default())
            .unwrap();

    assert_eq!(first_fingerprint, second_fingerprint);
}
