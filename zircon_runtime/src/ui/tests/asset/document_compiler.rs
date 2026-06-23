use super::*;

#[test]
fn ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets() {
    let button_asset = UiAssetLoader::load_toml_str(IMPORTED_BUTTON_ASSET_TOML).unwrap();
    let toolbar_asset = UiAssetLoader::load_toml_str(IMPORTED_TOOLBAR_ASSET_TOML).unwrap();
    let style_asset = UiAssetLoader::load_toml_str(IMPORTED_STYLE_ASSET_TOML).unwrap();
    let layout_asset = UiAssetLoader::load_toml_str(LAYOUT_ASSET_TOML).unwrap();

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

    let compiled = compiler.compile(&layout_asset).unwrap();
    assert_eq!(compiled.asset.kind, UiAssetKind::Layout);
    assert_eq!(compiled.asset.id, "editor.ui_asset_editor");

    let instance = compiled.clone().into_template_instance();
    assert_eq!(instance.root.component.as_deref(), Some("VerticalBox"));
    assert_eq!(instance.root.control_id.as_deref(), Some("EditorRoot"));
    assert_eq!(instance.root.children.len(), 1);
    assert_eq!(
        instance.root.children[0].component.as_deref(),
        Some("HorizontalBox")
    );
    assert_eq!(
        instance.root.children[0].control_id.as_deref(),
        Some("ToolbarHost")
    );

    let open_button = instance.root.children[0]
        .children
        .iter()
        .find(|child| child.control_id.as_deref() == Some("OpenButton"))
        .unwrap();
    assert_eq!(open_button.component.as_deref(), Some("Button"));
    assert_eq!(
        open_button.attributes.get("text").unwrap().as_str(),
        Some("Open Override")
    );
    assert_eq!(
        open_button.attributes.get("icon").unwrap().as_str(),
        Some("folder-open-outline")
    );
    assert_eq!(open_button.bindings.len(), 1);
    assert_eq!(open_button.bindings[0].id, "Toolbar/Open");
    assert_eq!(
        open_button.bindings[0].route.as_deref(),
        Some("Toolbar.Open")
    );
    assert_eq!(
        open_button.classes,
        vec![
            "toolbar-button",
            "primary",
            "MuiHorizontalBox-leading",
            "MuiButton-root",
        ]
    );
    assert_eq!(
        open_button
            .attributes
            .get("background")
            .unwrap()
            .get("color")
            .unwrap()
            .as_str(),
        Some("#4488ff")
    );
    assert_eq!(
        open_button
            .attributes
            .get("layout")
            .unwrap()
            .get("width")
            .unwrap()
            .get("preferred")
            .unwrap()
            .as_float(),
        Some(144.0)
    );
    assert_eq!(
        open_button
            .slot_attributes
            .get("layout")
            .unwrap()
            .get("height")
            .unwrap()
            .get("preferred")
            .unwrap()
            .as_float(),
        Some(40.0)
    );

    let mut surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("ui.asset.layout"),
        &compiled,
    )
    .unwrap();
    surface.compute_layout(UiSize::new(800.0, 600.0)).unwrap();

    let open_button_node = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OpenButton")
        })
        .unwrap();
    assert_eq!(open_button_node.layout_cache.frame.width, 144.0);
    assert_eq!(open_button_node.layout_cache.frame.height, 40.0);

    let open_button_render = surface
        .render_extract
        .list
        .commands
        .iter()
        .find(|command| command.node_id == open_button_node.node_id)
        .unwrap();
    assert_eq!(open_button_render.kind, UiRenderCommandKind::Quad);
    assert_eq!(open_button_render.text.as_deref(), Some("Open Override"));
    assert_eq!(
        open_button_render.image,
        Some(UiVisualAssetRef::Icon("folder-open-outline".to_string()))
    );
    assert_eq!(open_button_render.opacity, 1.0);
    assert_eq!(
        open_button_render.style.background_color.as_deref(),
        Some("#4488ff")
    );
}

#[test]
fn ui_asset_loader_materializes_recursive_tree_authority_in_memory() {
    let document = UiAssetLoader::load_toml_str(LAYOUT_ASSET_TOML).unwrap();
    let root = document.root.as_ref().expect("layout root");

    assert_eq!(root.node_id, "editor_root");
    assert_eq!(root.control_id.as_deref(), Some("EditorRoot"));
    assert_eq!(root.children.len(), 1);

    let toolbar = &root.children[0];
    assert_eq!(toolbar.mount.as_deref(), None);
    assert_eq!(toolbar.node.node_id, "toolbar");
    assert_eq!(
        toolbar.node.component_ref.as_deref(),
        Some("asset://ui/common/toolbar.ui#ToolbarShell")
    );
    assert_eq!(toolbar.node.children.len(), 1);

    let open_button = &toolbar.node.children[0];
    assert_eq!(open_button.mount.as_deref(), Some("leading"));
    assert_eq!(open_button.node.node_id, "open_button");
    assert_eq!(
        open_button.node.component_ref.as_deref(),
        Some("asset://ui/common/buttons.ui#ToolbarButton")
    );
}
