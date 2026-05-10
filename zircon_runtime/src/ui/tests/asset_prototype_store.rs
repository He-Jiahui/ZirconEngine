use crate::ui::template::{
    UiAssetLoader, UiDocumentCompiler, UiPrototypeStoreBuilder, UiPrototypeStoreFileCache,
    UiTemplateSurfaceBuilder,
};
use std::sync::Arc;
use zircon_runtime_interface::ui::template::{UiAssetError, UiNodeDefinitionKind};
use zircon_runtime_interface::ui::{event_ui::UiTreeId, layout::UiSize};

const FLAT_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "asset://ui/tests/flat_layout.ui"
version = 3
display_name = "Flat Layout"

[imports]
widgets = ["asset://ui/tests/card.ui#Card"]
styles = ["asset://ui/tests/theme.ui"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "card", mount = "content" }]

[nodes.card]
kind = "reference"
component_ref = "asset://ui/tests/card.ui#Card"
control_id = "CardInstance"
params = { title = "Stats" }
layout = { width = { min = 80.0, preferred = 80.0, max = 80.0, stretch = "Fixed" }, height = { min = 20.0, preferred = 20.0, max = 20.0, stretch = "Fixed" } }
children = [{ child = "body", mount = "content", slot = { layout = { height = { min = 24.0, preferred = 24.0, max = 24.0, stretch = "Fixed" } } } }]

[nodes.body]
kind = "native"
type = "Label"
control_id = "CardBody"
props = { text = "Body" }
"##;

const FLAT_WIDGET_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "asset://ui/tests/card.ui"
version = 3
display_name = "Card"

[components.Card]
root = "card_root"

[components.Card.params.title]
type = "string"
default = "Untitled"

[components.Card.slots.content]
multiple = true

[nodes.card_root]
kind = "native"
type = "Panel"
control_id = "CardRoot"
classes = ["card"]
children = [{ child = "title" }, { child = "content_slot" }]

[nodes.title]
kind = "native"
type = "Label"
props = { text = "$param.title" }

[nodes.content_slot]
kind = "slot"
slot_name = "content"
"##;

const FLAT_STYLE_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "asset://ui/tests/theme.ui"
version = 3
display_name = "Theme"

[[stylesheets]]
id = "theme"

[[stylesheets.rules]]
selector = ".card"
set = { self = { style_marker = "Styled" } }
"##;

#[test]
fn flat_asset_loader_materializes_heap_prototype_without_recursive_tree() {
    let prototype = UiAssetLoader::load_flat_prototype_toml_str(FLAT_LAYOUT_ASSET_TOML).unwrap();

    assert_eq!(prototype.asset.id, "asset://ui/tests/flat_layout.ui");
    assert!(prototype.document.root.is_some());
    assert_eq!(prototype.node_count(), 3);

    let root = prototype.node(prototype.document.root.unwrap()).unwrap();
    assert_eq!(root.node_id, "root");
    assert_eq!(root.kind, UiNodeDefinitionKind::Native);
    assert_eq!(root.children.len(), 1);
    let child = prototype.node(root.children[0].child).unwrap();
    assert_eq!(child.node_id, "card");
    assert_eq!(child.kind, UiNodeDefinitionKind::Reference);
    assert_eq!(child.control_id.as_deref(), Some("CardInstance"));
    assert_eq!(child.children.len(), 1);
}

#[test]
fn prototype_compiler_instantiates_native_flat_root_without_component_imports() {
    let prototype = UiAssetLoader::load_flat_prototype_toml_str(
        r##"
[asset]
kind = "layout"
id = "asset://ui/tests/native_root.ui"
version = 3

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Panel"
control_id = "NativeRoot"
children = [{ child = "label" }]

[nodes.label]
kind = "native"
type = "Label"
props = { text = "Native" }
"##,
    )
    .unwrap();
    let mut builder = UiPrototypeStoreBuilder::new();
    let _ = builder.insert(prototype);
    let store = builder.build().unwrap();

    let compiled = UiDocumentCompiler::default()
        .compile_prototype_asset("asset://ui/tests/native_root.ui", &store)
        .unwrap();
    let root = &compiled.template_instance().root;

    assert_eq!(root.control_id.as_deref(), Some("NativeRoot"));
    assert_eq!(root.children.len(), 1);
    assert_eq!(
        root.children[0]
            .attributes
            .get("text")
            .and_then(toml::Value::as_str),
        Some("Native")
    );
}

#[test]
fn prototype_store_resolves_component_handles_from_declared_import_assets() {
    let layout = UiAssetLoader::load_flat_prototype_toml_str(FLAT_LAYOUT_ASSET_TOML).unwrap();
    let widget = UiAssetLoader::load_flat_prototype_toml_str(FLAT_WIDGET_ASSET_TOML).unwrap();
    let style = UiAssetLoader::load_flat_prototype_toml_str(FLAT_STYLE_ASSET_TOML).unwrap();
    let mut builder = UiPrototypeStoreBuilder::new();

    let _ = builder.insert(layout);
    let _ = builder.insert(widget);
    let _ = builder.insert(style);
    let store = builder.build().unwrap();
    let (widget_asset, component_name) = store
        .component_prototype("asset://ui/tests/card.ui#Card")
        .unwrap();
    let component = widget_asset.components.get(&component_name).unwrap();
    let root = widget_asset.node(component.root).unwrap();

    assert_eq!(store.len(), 3);
    assert_eq!(component_name, "Card");
    assert_eq!(root.node_id, "card_root");
    assert_eq!(component.params["title"].r#type, "string");
    assert!(component.slots["content"].multiple);
}

#[test]
fn prototype_compiler_instantiates_imported_components_with_explicit_work_stack() {
    let store = prototype_store();

    let compiled = UiDocumentCompiler::default()
        .compile_prototype_asset("asset://ui/tests/flat_layout.ui", &store)
        .unwrap();
    let root = &compiled.template_instance().root;
    let card = &root.children[0];
    let title = &card.children[0];
    let body = &card.children[1];

    assert_eq!(root.control_id.as_deref(), Some("Root"));
    assert_eq!(card.control_id.as_deref(), Some("CardInstance"));
    assert_eq!(
        title.attributes.get("text").and_then(toml::Value::as_str),
        Some("Stats")
    );
    assert_eq!(
        card.attributes
            .get("layout")
            .and_then(|layout| layout.get("width"))
            .and_then(|width| width.get("preferred"))
            .and_then(toml::Value::as_float),
        Some(80.0)
    );
    assert_eq!(
        card.attributes
            .get("style_marker")
            .and_then(toml::Value::as_str),
        Some("Styled")
    );
    assert_eq!(
        body.attributes.get("text").and_then(toml::Value::as_str),
        Some("Body")
    );
    assert_eq!(
        body.slot_attributes
            .get("layout")
            .and_then(|layout| layout.get("height"))
            .and_then(|height| height.get("preferred"))
            .and_then(toml::Value::as_float),
        Some(24.0)
    );
}

#[test]
fn prototype_compiler_projects_surface_from_flat_prototype_instance() {
    let store = prototype_store();
    let compiled = UiDocumentCompiler::default()
        .compile_prototype_asset("asset://ui/tests/flat_layout.ui", &store)
        .unwrap();

    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("prototype.surface"),
        &compiled,
    )
    .unwrap();
    surface.compute_layout(UiSize::new(200.0, 60.0)).unwrap();

    assert_eq!(surface.tree.nodes.len(), 4);
    assert_eq!(surface.render_extract.list.commands.len(), 4);
    assert!(surface.tree.nodes.values().any(|node| node
        .template_metadata
        .as_ref()
        .and_then(|metadata| metadata.control_id.as_deref())
        == Some("CardInstance")));
}

#[test]
fn prototype_store_file_cache_reuses_loaded_flat_store_for_projection_hot_path() {
    let temp_dir = prototype_cache_temp_dir("projection_hot_path");
    let layout_path = temp_dir.join("flat_layout.ui.toml");
    let widget_path = temp_dir.join("card.ui.toml");
    let style_path = temp_dir.join("theme.ui.toml");
    std::fs::write(&layout_path, FLAT_LAYOUT_ASSET_TOML).unwrap();
    std::fs::write(&widget_path, FLAT_WIDGET_ASSET_TOML).unwrap();
    std::fs::write(&style_path, FLAT_STYLE_ASSET_TOML).unwrap();
    let mut cache = UiPrototypeStoreFileCache::new();
    let sources = vec![layout_path, widget_path, style_path];

    let first = cache.load_flat_store(sources.clone()).unwrap();
    let second = cache.load_flat_store(sources).unwrap();

    assert!(!first.cache_hit);
    assert!(second.cache_hit);
    assert_eq!(first.root_asset_id, "asset://ui/tests/flat_layout.ui");
    assert!(Arc::ptr_eq(&first.store, &second.store));
    assert_eq!(cache.len(), 1);

    let compiled = UiDocumentCompiler::default()
        .compile_prototype_asset(&second.root_asset_id, second.store.as_ref())
        .unwrap();
    assert_eq!(
        compiled.template_instance().root.control_id.as_deref(),
        Some("Root")
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn prototype_store_file_cache_resolves_res_aliases_and_transitive_style_imports() {
    let temp_dir = prototype_cache_temp_dir("res_alias_imports");
    let assets_root = temp_dir.join("assets");
    let layout_path = assets_root.join("ui/editor/layout.ui.toml");
    let widget_path = assets_root.join("ui/editor/card.ui.toml");
    let base_style_path = assets_root.join("ui/theme/base.ui.toml");
    let material_style_path = assets_root.join("ui/theme/material.ui.toml");
    std::fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(base_style_path.parent().unwrap()).unwrap();
    std::fs::write(
        &layout_path,
        r##"
[asset]
kind = "layout"
id = "editor.alias_layout"
version = 1

[imports]
widgets = ["res://ui/editor/card.ui.toml#Card"]
styles = ["res://ui/theme/base.ui.toml"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "AliasRoot"
children = [{ child = "card" }]

[nodes.card]
kind = "reference"
component_ref = "res://ui/editor/card.ui.toml#Card"
"##,
    )
    .unwrap();
    std::fs::write(
        &widget_path,
        r##"
[asset]
kind = "widget"
id = "ui.editor.card"
version = 1

[components.Card]
root = "card_root"

[nodes.card_root]
kind = "native"
type = "Panel"
control_id = "CardRoot"
classes = ["card"]
"##,
    )
    .unwrap();
    std::fs::write(
        &base_style_path,
        r##"
[asset]
kind = "style"
id = "ui.theme.base"
version = 1

[imports]
styles = ["res://ui/theme/material.ui.toml"]

[[stylesheets]]
id = "base"

[[stylesheets.rules]]
selector = ".card"
set = { self = { style_marker = "BaseStyled" } }
"##,
    )
    .unwrap();
    std::fs::write(
        &material_style_path,
        r##"
[asset]
kind = "style"
id = "ui.theme.material"
version = 1
"##,
    )
    .unwrap();
    let mut cache = UiPrototypeStoreFileCache::new();

    let first = cache.load_flat_store(vec![layout_path.clone()]).unwrap();
    let second = cache.load_flat_store(vec![layout_path]).unwrap();

    assert!(!first.cache_hit);
    assert!(second.cache_hit);
    assert!(second.store.get("res://ui/editor/card.ui.toml").is_some());
    assert!(second
        .store
        .get("res://ui/theme/material.ui.toml")
        .is_some());
    let compiled = UiDocumentCompiler::default()
        .compile_prototype_asset(&second.root_asset_id, second.store.as_ref())
        .unwrap();
    assert_eq!(
        compiled.template_instance().root.children[0]
            .attributes
            .get("style_marker")
            .and_then(toml::Value::as_str),
        Some("BaseStyled")
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn prototype_store_file_cache_reports_tree_schema_as_non_flat_without_caching() {
    let temp_dir = prototype_cache_temp_dir("tree_schema_fallback");
    let layout_path = temp_dir.join("tree_layout.ui.toml");
    std::fs::write(
        &layout_path,
        r#"
[asset]
kind = "layout"
id = "asset://ui/tests/tree_layout.ui"
version = 1

[root]
node_id = "root"
kind = "native"
type = "Label"
props = { text = "Tree" }
"#,
    )
    .unwrap();
    let mut cache = UiPrototypeStoreFileCache::new();

    let outcome = cache.try_load_flat_store(vec![layout_path]).unwrap();

    assert!(outcome.is_none());
    assert!(cache.is_empty());

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn prototype_compiler_instantiates_deep_flat_chain_with_explicit_work_stack() {
    const NODE_COUNT: usize = 10_000;
    let prototype =
        UiAssetLoader::load_flat_prototype_toml_str(&deep_flat_chain_asset_toml(NODE_COUNT))
            .unwrap();
    let mut builder = UiPrototypeStoreBuilder::new();
    let _ = builder.insert(prototype);
    let store = builder.build().unwrap();

    let compiled = UiDocumentCompiler::default()
        .compile_prototype_asset("asset://ui/tests/deep_chain.ui", &store)
        .unwrap();
    let mut depth = 1usize;
    let mut node = &compiled.template_instance().root;
    while let Some(child) = node.children.first() {
        depth += 1;
        node = child;
    }

    assert_eq!(depth, NODE_COUNT);
    assert_eq!(node.control_id.as_deref(), Some("N9999"));
    // This test targets non-recursive instantiation; do not make Rust's recursive
    // drop of the transitional UiTemplateNode tree part of the stack-safety signal.
    std::mem::forget(compiled);
}

#[test]
fn prototype_store_reports_missing_declared_import_before_hot_path_projection() {
    let layout = UiAssetLoader::load_flat_prototype_toml_str(FLAT_LAYOUT_ASSET_TOML).unwrap();
    let mut builder = UiPrototypeStoreBuilder::new();

    let _ = builder.insert(layout);
    let error = builder
        .build()
        .expect_err("missing widget/style prototypes must not be hidden until projection");

    assert_eq!(
        error,
        UiAssetError::UnknownImport {
            reference: "asset://ui/tests/card.ui".to_string(),
        }
    );
}

#[test]
fn flat_prototype_loader_rejects_cycles_iteratively() {
    let error = UiAssetLoader::load_flat_prototype_toml_str(
        r##"
[asset]
kind = "layout"
id = "asset://ui/tests/cycle.ui"
version = 3

[root]
node = "a"

[nodes.a]
kind = "native"
type = "VerticalBox"
children = [{ child = "b" }]

[nodes.b]
kind = "native"
type = "VerticalBox"
children = [{ child = "a" }]
"##,
    )
    .expect_err("flat prototype cycles must be rejected without recursive descent");

    assert!(
        error.to_string().contains("cycle"),
        "unexpected error: {error:?}"
    );
}

fn prototype_store() -> crate::ui::template::UiPrototypeStore {
    let layout = UiAssetLoader::load_flat_prototype_toml_str(FLAT_LAYOUT_ASSET_TOML).unwrap();
    let widget = UiAssetLoader::load_flat_prototype_toml_str(FLAT_WIDGET_ASSET_TOML).unwrap();
    let style = UiAssetLoader::load_flat_prototype_toml_str(FLAT_STYLE_ASSET_TOML).unwrap();
    let mut builder = UiPrototypeStoreBuilder::new();
    let _ = builder.insert(layout);
    let _ = builder.insert(widget);
    let _ = builder.insert(style);
    builder.build().unwrap()
}

fn prototype_cache_temp_dir(test_name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "zircon_ui_prototype_store_{test_name}_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn deep_flat_chain_asset_toml(node_count: usize) -> String {
    let mut document = String::from(
        r#"
[asset]
kind = "layout"
id = "asset://ui/tests/deep_chain.ui"
version = 3

[root]
node = "n0"
"#,
    );
    for index in 0..node_count {
        document.push_str(&format!(
            "\n[nodes.n{index}]\nkind = \"native\"\ntype = \"Panel\"\ncontrol_id = \"N{index}\"\n"
        ));
        if index + 1 < node_count {
            document.push_str(&format!("children = [{{ child = \"n{}\" }}]\n", index + 1));
        }
    }
    document
}
