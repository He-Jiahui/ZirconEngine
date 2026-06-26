use super::*;

#[test]
fn mui_x_tree_view_utility_classes_match_retained_targets() {
    let style = UiAssetLoader::load_toml_str(MUI_X_STYLE_TOML).unwrap();
    let layout = UiAssetLoader::load_toml_str(MUI_X_LAYOUT_TOML).unwrap();
    let mut compiler = UiDocumentCompiler::default();
    compiler
        .register_style_import("asset://ui/tests/mui_web_mui_x_style.ui", style)
        .unwrap();
    let compiled = compiler.compile(&layout).unwrap();
    let root = &compiled.template_instance().root;

    let tree = find_node(root, "TreeViewRoot");
    assert_eq!(
        str_attr(tree, "surface_variant"),
        Some("tree-view-multiselect")
    );
    assert_eq!(
        str_attr(tree, "validation_level"),
        Some("tree-view-data-bound")
    );
    assert_classes(
        tree,
        &[
            "MuiMaterialTreeView-root",
            "MuiTreeView-root",
            "MuiTreeView-multiSelect",
            "MuiTreeView-checkboxSelection",
            "MuiTreeView-disabledItemsFocusable",
            "MuiTreeView-editable",
            "MuiTreeView-hasExpandedItems",
            "MuiTreeView-hasSelectedItems",
            "MuiTreeView-hasItemIndentation",
        ],
    );

    let tree_item = find_node(root, "TreeViewItem");
    assert_eq!(
        str_attr(tree_item, "validation_level"),
        Some("tree-item-state")
    );
    assert_classes(
        tree_item,
        &[
            "MuiTreeItem-root",
            "MuiTreeItem-expanded",
            "MuiTreeItem-selected",
            "MuiTreeItem-editable",
            "MuiTreeItem-disabledItemsFocusable",
        ],
    );

    let tree_content = find_node(root, "TreeViewContent");
    assert_eq!(
        str_attr(tree_content, "text_tone"),
        Some("tree-item-content")
    );
    assert_classes(
        tree_content,
        &[
            "MuiTreeItem-content",
            "MuiTreeItem-expanded",
            "MuiTreeItem-selected",
        ],
    );

    let tree_label = find_node(root, "TreeViewLabel");
    assert_eq!(
        str_attr(tree_label, "text_tone"),
        Some("tree-item-label-input")
    );
    assert_classes(tree_label, &["MuiTreeItem-label", "MuiTreeItem-labelInput"]);

    let tree_icon = find_node(root, "TreeViewIcon");
    assert_eq!(
        str_attr(tree_icon, "surface_variant"),
        Some("tree-item-icon")
    );
    assert_classes(tree_icon, &["MuiTreeItem-iconContainer"]);

    let tree_checkbox = find_node(root, "TreeViewCheckbox");
    assert_eq!(
        str_attr(tree_checkbox, "surface_variant"),
        Some("tree-item-checkbox")
    );
    assert_classes(
        tree_checkbox,
        &["MuiTreeItem-checkbox", "MuiTreeItem-checkboxSelection"],
    );

    let tree_features = find_node(root, "TreeViewFeatureFlagsRoot");
    assert_eq!(
        str_attr(tree_features, "text_tone"),
        Some("tree-view-feature-flags")
    );
    assert_classes(
        tree_features,
        &[
            "MuiMaterialTreeView-root",
            "MuiTreeView-root",
            "MuiTreeView-checkboxSelection",
            "MuiTreeView-disabledItemsFocusable",
            "MuiTreeView-editable",
        ],
    );
}
