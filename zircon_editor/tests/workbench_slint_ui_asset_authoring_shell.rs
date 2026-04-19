use std::fs;
use std::path::PathBuf;

fn panes_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench/panes.slint");
    fs::read_to_string(path).expect("panes.slint should be readable")
}

fn block_after<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker `{marker}`"));
    &source[start..]
}

#[test]
fn ui_asset_editor_pane_declares_preview_mock_subject_and_expression_controls() {
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let preview_struct_block = block_after(&panes, "export struct UiAssetPreviewMockData {");

    assert!(preview_struct_block.contains("subject_collection: UiAssetStringSelectionData,"));
    assert!(preview_struct_block.contains("subject_node_id: string,"));
    assert!(preview_struct_block.contains("expression_result: string,"));
    assert!(preview_struct_block.contains("schema_items: [string],"));
    assert!(pane_block.contains(
        "property <UiAssetPreviewPanelData> preview_panel: root.pane.preview;"
    ));
    assert!(panes.contains("title: \"Mock Subjects\";"));
    assert!(panes.contains("items: root.preview_panel.mock.subject_collection.items;"));
    assert!(panes.contains(
        "root.collection_event(\"preview_mock_subject\", \"selected\", item_index);"
    ));
    assert!(panes.contains("text: root.preview_panel.mock.expression_result;"));
    assert!(panes.contains("title: \"Mock Schema\";"));
    assert!(panes.contains("items: root.preview_panel.mock.schema_items;"));
}

#[test]
fn ui_asset_editor_pane_declares_binding_target_suggestion_controls() {
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let binding_struct_block = block_after(&panes, "export struct UiAssetInspectorBindingData {");

    assert!(binding_struct_block.contains("route_suggestion_collection: UiAssetStringSelectionData,"));
    assert!(binding_struct_block.contains("action_suggestion_collection: UiAssetStringSelectionData,"));
    assert!(binding_struct_block.contains("schema_items: [string],"));
    assert!(pane_block.contains(
        "property <UiAssetInspectorPanelData> inspector_panel: root.pane.inspector;"
    ));
    assert!(panes.contains("title: \"Route Suggestions\";"));
    assert!(panes.contains("items: root.inspector_panel.binding.route_suggestion_collection.items;"));
    assert!(panes.contains(
        "root.detail_event(\"binding_route_suggestion\", \"binding.route.suggestion.apply\", item_index, \"\", \"\");"
    ));
    assert!(panes.contains("title: \"Action Suggestions\";"));
    assert!(panes.contains("items: root.inspector_panel.binding.action_suggestion_collection.items;"));
    assert!(panes.contains(
        "root.detail_event(\"binding_action_suggestion\", \"binding.action.suggestion.apply\", item_index, \"\", \"\");"
    ));
    assert!(panes.contains("title: \"Binding Schema\";"));
    assert!(panes.contains("items: root.inspector_panel.binding.schema_items;"));
}
