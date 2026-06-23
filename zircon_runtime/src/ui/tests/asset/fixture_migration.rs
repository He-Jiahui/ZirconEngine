use super::*;

#[test]
fn ui_source_template_fixture_conversion_converts_template_documents_into_asset_documents() {
    let asset_document = UiAssetSchemaMigrator::migrate_source_template_fixture_str(
        "source.workbench",
        "Source Workbench",
        LEGACY_TEMPLATE_TOML,
    )
    .unwrap()
    .document;

    assert_eq!(asset_document.asset.kind, UiAssetKind::Layout);
    assert_eq!(asset_document.asset.id, "source.workbench");
    assert_eq!(asset_document.asset.display_name, "Source Workbench");
    assert_eq!(asset_document.root.as_ref().unwrap().node_id, "root");
    assert_eq!(asset_document.root.as_ref().unwrap().children.len(), 1);
    assert_eq!(
        asset_document.root.as_ref().unwrap().children[0]
            .node
            .node_id,
        "root_0"
    );

    let compiler = UiDocumentCompiler::default();
    let compiled = compiler.compile(&asset_document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(instance.root.component.as_deref(), Some("VerticalBox"));
    assert_eq!(instance.root.control_id.as_deref(), Some("LegacyRoot"));
    assert_eq!(instance.root.children.len(), 1);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("Button")
    );
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("LegacyButton")
    );
    assert_eq!(
        instance.root.children[0].attributes.get("text"),
        Some(&Value::String("Open".to_string()))
    );
    assert_eq!(instance.root.children[0].bindings[0].id, "Legacy/Button");
}

#[test]
fn ui_source_template_fixture_conversion_emits_canonical_asset_source_that_roundtrips() {
    let source = toml::to_string_pretty(
        &UiAssetSchemaMigrator::migrate_source_template_fixture_str(
            "source.workbench",
            "Source Workbench",
            LEGACY_TEMPLATE_TOML,
        )
        .unwrap()
        .document,
    )
    .unwrap();
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(document.asset.id, "source.workbench");
    assert_eq!(instance.root.component.as_deref(), Some("VerticalBox"));
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("LegacyButton")
    );
}

#[test]
fn ui_flat_fixture_migration_converts_flat_assets_into_tree_authority_source() {
    let migrated = UiAssetSchemaMigrator::migrate_toml_str(FLAT_LAYOUT_ASSET_TOML).unwrap();
    let source = toml::to_string_pretty(&migrated.document).unwrap();
    let document = UiAssetLoader::load_toml_str(&source).unwrap();
    assert!(
        !source.contains("[nodes."),
        "migrated source should stop emitting flat [nodes.*] tables"
    );
    let root = document.root.as_ref().expect("migrated root");
    assert_eq!(root.node_id, "editor_root");
    assert_eq!(root.children.len(), 1);
    assert_eq!(root.children[0].node.node_id, "toolbar");
    assert_eq!(root.children[0].node.children.len(), 1);
    assert_eq!(
        root.children[0].node.children[0].node.node_id,
        "open_button"
    );

    let button_asset = UiAssetLoader::load_toml_str(IMPORTED_BUTTON_ASSET_TOML).unwrap();
    let toolbar_asset = UiAssetLoader::load_toml_str(IMPORTED_TOOLBAR_ASSET_TOML).unwrap();
    let style_asset = UiAssetLoader::load_toml_str(IMPORTED_STYLE_ASSET_TOML).unwrap();

    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_widget_import("asset://ui/common/buttons.ui#ToolbarButton", button_asset)
        .unwrap();
    compiler
        .register_widget_import("asset://ui/common/toolbar.ui#ToolbarShell", toolbar_asset)
        .unwrap();
    compiler
        .register_style_import("asset://ui/theme/editor_base.ui", style_asset)
        .unwrap();

    let compiled = compiler.compile(&document).unwrap();
    let instance = compiled.into_template_instance();

    assert_eq!(document.asset.id, "editor.ui_asset_editor");
    assert_eq!(instance.root.control_id.as_deref(), Some("EditorRoot"));
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("ToolbarHost")
    );
    assert_eq!(
        instance.root.children[0].children[0].control_id.as_deref(),
        Some("OpenButton")
    );
}

#[test]
fn ui_asset_loader_migrates_flat_asset_documents_on_formal_path() {
    let document = UiAssetLoader::load_toml_str(FLAT_LAYOUT_ASSET_TOML)
        .expect("formal loader should migrate supported flat authority documents");

    let root = document.root.as_ref().expect("migrated root");
    assert_eq!(root.node_id, "editor_root");
    assert_eq!(root.children[0].node.node_id, "toolbar");
    assert_eq!(
        root.children[0].node.children[0].node.node_id,
        "open_button"
    );
}

#[test]
fn ui_asset_compiler_is_split_into_folder_backed_pipeline_modules() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("template")
        .join("asset")
        .join("compiler");

    for relative in [
        "mod.rs",
        "ui_document_compiler.rs",
        "compile.rs",
        "node_expander.rs",
        "component_instance_expander.rs",
        "ui_style_resolver.rs",
        "style_apply.rs",
        "value_normalizer.rs",
        "component_props.rs",
        "shape_validator.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected compiler pipeline module {relative} under {:?}",
            root
        );
    }
}
