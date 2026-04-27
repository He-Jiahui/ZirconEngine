#[test]
fn component_showcase_template_materializes_structure_and_collection_rows() {
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));

    assert!(
        template.contains("property <bool> structure_value_primitive"),
        "TemplatePane should classify inspector structure and collection rows"
    );
    assert!(
        template.contains("root.node.component_role == \"group\""),
        "Group rows should render retained inspector structure chrome"
    );
    assert!(
        template.contains("root.node.component_role == \"foldout\""),
        "Foldout rows should render retained disclosure chrome"
    );
    assert!(
        template.contains("root.node.component_role == \"array-field\""),
        "ArrayField rows should render a collection summary primitive"
    );
    assert!(
        template.contains("root.node.component_role == \"map-field\""),
        "MapField rows should render a key/value summary primitive"
    );
    assert!(
        template.contains("root.node.expanded ? \"v\" : \">\""),
        "Structure rows should show retained expanded/collapsed state"
    );
    assert!(
        template.contains("root.node.value_text != \"\" ? root.node.value_text"),
        "Collection rows should surface retained value summaries such as element counts"
    );
    assert!(
        template.contains("root.structure_value_primitive ? parent.width * 0.72"),
        "Collection action chips should move beside the summary instead of covering it"
    );
}

#[test]
fn component_showcase_template_materializes_deep_collection_and_menu_state() {
    let template = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/ui/workbench/template_pane.slint"
    ));

    assert!(
        template.contains("collection_items"),
        "TemplatePane should render retained ArrayField and MapField child rows"
    );
    assert!(
        template.contains("root.node.selection_state == \"focused\""),
        "List and tree rows should have a focused retained visual state"
    );
    assert!(
        template.contains("root.node.dragging"),
        "Dragging state should be projected into the retained generic host"
    );
    assert!(
        template.contains("root.node.drop_hovered"),
        "Drop-hover state should be projected into reference drop wells"
    );
    assert!(
        template.contains("menu_items"),
        "ContextActionMenu should render retained menu-row metadata"
    );
    assert!(
        template.contains("root.node.menu_items.length"),
        "ContextActionMenu rows should surface shortcut-capable retained menu metadata"
    );
}
