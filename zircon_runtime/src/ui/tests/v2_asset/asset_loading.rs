use super::*;

#[test]
fn ui_v2_parses_flat_view_asset() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/project_overview.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
classes = ["editor-pane"]

[[nodes.root.children]]
node = "title"

[nodes.title]
component = "Text"
control_id = "ProjectTitle"

[nodes.title.props]
text = "Project"
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();

    assert_eq!(compiled.arena.node_count(), 2);
    assert_eq!(compiled.component_graph.node_count(), 2);
    let root_handle = compiled.arena.root.unwrap();
    let root_graph = &compiled.component_graph.nodes[root_handle.index()];
    assert_eq!(root_graph.source_id, "root");
    assert_eq!(root_graph.children.len(), 1);
    assert_eq!(
        compiled.arena.node(root_handle).unwrap().component,
        "VerticalGroup"
    );
}

#[test]
fn ui_zui_loader_accepts_single_component_asset() {
    let document = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/primary_toolbar.zui"
version = 2

[components.PrimaryToolbar]
root = "root"

[nodes.root]
component = "HorizontalGroup"
classes = ["toolbar"]

[[nodes.root.children]]
node = "run_button"

[nodes.run_button]
component = "Button"
control_id = "RunButton"

[nodes.run_button.props]
text = "Run"
"#,
    )
    .unwrap();

    assert_eq!(document.asset.kind, UiV2AssetKind::Component);
    assert!(document.root.is_none());
    assert_eq!(document.components.len(), 1);
    assert!(document.components.contains_key("PrimaryToolbar"));
    assert!(document.nodes.contains_key("root"));
}

#[test]
fn ui_v2_repeat_declaration_is_preserved_in_compiled_surface_metadata() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/repeat.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
control_id = "Rows"
repeat = { kind = "virtual_rows", prototype = "RowPrototype", virtual_control_prefix = "VirtualRow", authored_count = 1, node_path_namespace = "v2" }
children = [{ node = "row" }]

[nodes.row]
component = "Text"
control_id = "RowPrototype"
props = { text = "Row" }
"#,
    )
    .unwrap();

    let compiled = UiV2DocumentCompiler::compile(&document).unwrap();
    let root = compiled
        .arena
        .node(compiled.arena.root.expect("repeat root"))
        .unwrap();
    let repeat = root.repeat.as_ref().expect("compiled repeat declaration");
    assert_eq!(repeat.kind, UI_V2_REPEAT_KIND_VIRTUAL_ROWS);
    assert_eq!(repeat.prototype, "RowPrototype");
    assert_eq!(repeat.virtual_control_prefix, "VirtualRow");

    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.repeat_metadata"),
        &document,
        &compiled,
    )
    .unwrap();
    let root_id = node_id_by_control_id(&surface, "Rows");
    let repeat = surface
        .tree
        .nodes
        .get(&root_id)
        .unwrap()
        .template_metadata
        .as_ref()
        .unwrap()
        .attributes
        .get(UI_V2_REPEAT_ATTRIBUTE)
        .and_then(Value::as_table)
        .expect("surface repeat metadata");

    assert_eq!(
        repeat.get(UI_V2_REPEAT_FIELD_KIND).and_then(Value::as_str),
        Some(UI_V2_REPEAT_KIND_VIRTUAL_ROWS)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_AUTHORED_COUNT)
            .and_then(Value::as_integer),
        Some(1)
    );
    assert_eq!(
        repeat
            .get(UI_V2_REPEAT_FIELD_NODE_PATH_NAMESPACE)
            .and_then(Value::as_str),
        Some("v2")
    );
}

#[test]
fn ui_v2_rejects_invalid_repeat_declaration_before_surface_build() {
    let document = UiV2AssetLoader::load_toml_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/tests/repeat_invalid.v2.ui"
version = 2

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
control_id = "Rows"
repeat = { kind = "virtual_rows", prototype = "", virtual_control_prefix = "VirtualRow", authored_count = 1, node_path_namespace = "v2" }
children = [{ node = "row" }]

[nodes.row]
component = "Text"
control_id = "RowPrototype"
props = { text = "Row" }
"#,
    )
    .unwrap();

    let error = UiV2DocumentCompiler::compile(&document).unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. }
            if detail.contains("repeat.prototype") && detail.contains("must not be empty")
    ));
}

#[test]
fn ui_zui_loader_accepts_view_root_assets() {
    let document = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "view"
id = "asset://ui/editor/workbench.zui"
version = 2

[root]
node = "root"

[nodes.root]
component = "Container"
"#,
    )
    .unwrap();

    assert_eq!(document.asset.kind, UiV2AssetKind::View);
    assert_eq!(
        document.root.as_ref().map(|root| root.node.as_str()),
        Some("root")
    );
    assert!(document.nodes.contains_key("root"));
}

#[test]
fn ui_zui_loader_rejects_view_root_on_component_assets() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_root.zui"
version = 2

[root]
node = "root"

[components.InvalidRoot]
root = "root"

[nodes.root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("[root]")
    ));
}

#[test]
fn ui_zui_loader_rejects_multiple_components() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_multiple.zui"
version = 2

[components.LeftPanel]
root = "left_root"

[components.RightPanel]
root = "right_root"

[nodes.left_root]
component = "Container"

[nodes.right_root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("exactly one component")
    ));
}

#[test]
fn ui_zui_loader_rejects_missing_component() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_missing_component.zui"
version = 2

[nodes.root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("exactly one component")
    ));
}

#[test]
fn ui_zui_loader_rejects_empty_component_root() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_empty_root.zui"
version = 2

[components.EmptyRoot]
root = ""

[nodes.root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("non-empty root node")
    ));
}

#[test]
fn ui_zui_loader_rejects_missing_component_root_node() {
    let error = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "component"
id = "asset://ui/editor/invalid_missing_root_node.zui"
version = 2

[components.MissingRoot]
root = "missing"

[nodes.root]
component = "Container"
"#,
    )
    .unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::MissingNode { node_id, .. } if node_id == "missing"
    ));
}

#[test]
fn ui_zui_loader_accepts_style_assets() {
    let document = UiZuiAssetLoader::load_zui_str(
        r#"
[asset]
kind = "style"
id = "asset://ui/editor/theme.zui"
version = 2

[[stylesheets]]
id = "editor_theme"
"#,
    )
    .unwrap();

    assert_eq!(document.asset.kind, UiV2AssetKind::Style);
    assert_eq!(document.stylesheets.len(), 1);
}

#[test]
fn ui_v2_rejects_cycles_before_surface_build() {
    let mut document = v2_document("asset://ui/tests/cycle.v2.ui", "a");
    document.nodes.insert(
        "a".to_string(),
        UiV2NodeDefinition {
            component: "Container".to_string(),
            children: vec![UiV2ChildMount {
                node: "b".to_string(),
                slot: BTreeMap::new(),
            }],
            ..Default::default()
        },
    );
    document.nodes.insert(
        "b".to_string(),
        UiV2NodeDefinition {
            component: "Container".to_string(),
            children: vec![UiV2ChildMount {
                node: "a".to_string(),
                slot: BTreeMap::new(),
            }],
            ..Default::default()
        },
    );

    let error = UiV2DocumentCompiler::compile(&document).unwrap_err();

    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { detail, .. } if detail.contains("cycle")
    ));
}
