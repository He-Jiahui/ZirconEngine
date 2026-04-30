fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn component_showcase_structure_and_collection_state_is_rust_owned() {
    let template_nodes = source("src/ui/slint_host/host_contract/data/template_nodes.rs");
    let showcase_asset = source("assets/ui/editor/component_showcase.ui.toml");

    for required in [
        "pub(crate) struct TemplatePaneCollectionFieldData",
        "pub row_id: SharedString",
        "pub key_component_role: SharedString",
        "pub value_component_role: SharedString",
        "pub value_checked: bool",
        "pub validation_message: SharedString",
        "pub key_edit_action_id: SharedString",
        "pub edit_action_id: SharedString",
        "pub remove_action_id: SharedString",
        "pub collection_items: ModelRc<SharedString>",
        "pub collection_fields: ModelRc<TemplatePaneCollectionFieldData>",
        "pub expanded: bool",
    ] {
        assert!(
            template_nodes.contains(required),
            "collection DTO missing `{required}`"
        );
    }
    for required in ["ArrayFieldDemo", "MapFieldDemo", "InspectorSectionDemo"] {
        assert!(
            showcase_asset.contains(required),
            "component showcase asset missing `{required}`"
        );
    }
}

#[test]
fn component_showcase_option_menu_and_tree_state_is_rust_owned() {
    let template_nodes = source("src/ui/slint_host/host_contract/data/template_nodes.rs");
    let showcase_asset = source("assets/ui/editor/component_showcase.ui.toml");

    for required in [
        "pub(crate) struct TemplatePaneOptionData",
        "pub focused: bool",
        "pub hovered: bool",
        "pub pressed: bool",
        "pub matched: bool",
        "pub(crate) struct TemplatePaneMenuItemData",
        "pub shortcut: SharedString",
        "pub separator: bool",
        "pub structured_options: ModelRc<TemplatePaneOptionData>",
        "pub structured_menu_items: ModelRc<TemplatePaneMenuItemData>",
        "pub tree_depth: i32",
        "pub tree_indent_px: f32",
        "pub search_query: SharedString",
    ] {
        assert!(
            template_nodes.contains(required),
            "template node DTO missing `{required}`"
        );
    }
    for required in [
        "DropdownDemo",
        "SearchSelectDemo",
        "ContextActionMenuDemo",
        "TreeRowDemo",
    ] {
        assert!(
            showcase_asset.contains(required),
            "component showcase asset missing `{required}`"
        );
    }
}
