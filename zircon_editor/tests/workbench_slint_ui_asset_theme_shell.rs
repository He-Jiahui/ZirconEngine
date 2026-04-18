use std::fs;
use std::path::PathBuf;

fn shell_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/workbench.slint");
    fs::read_to_string(path).expect("workbench.slint should be readable")
}

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
fn ui_asset_editor_theme_panel_declares_open_promote_and_local_detail_selection_controls() {
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(panes.contains("root.action(\"theme.source.open\");"));
    assert!(panes.contains("root.action(\"theme.local.promote\");"));
    assert!(panes.contains(
        "enabled: root.theme_source.selected_source_kind == \"Imported\" && root.theme_source.selected_source_available;"
    ));
    assert!(panes.contains(
        "enabled: root.theme_source.selected_source_kind == \"Local\" && root.theme_source.can_promote_local;"
    ));
    assert!(panes.contains("item_activated(item_index) => {"));
    assert!(panes.contains("root.action(\"theme.source.select.\" + item_index);"));
    assert!(panes.contains("selected_index: root.theme_source.selected_source_kind == \"Local\" ? root.style_token.selected_index : -1;"));
    assert!(panes.contains(
        "root.detail_event(\"style_token\", \"style.token.select\", item_index, \"\", \"\");"
    ));
    assert!(panes.contains("selected_index: root.theme_source.selected_source_kind == \"Local\" ? root.style_rule.selected_index : -1;"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule\", \"style.rule.select\", item_index, \"\", \"\");"
    ));

    assert!(theme_struct_block.contains("can_promote_local: bool,"));
    assert!(pane_block.contains(
        "property <UiAssetThemeSourceData> theme_source: root.pane.theme_source;"
    ));
}

#[test]
fn ui_asset_editor_theme_panel_declares_promote_draft_controls() {
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(panes.contains("text: root.theme_source.promote_asset_id;"));
    assert!(panes.contains("text: root.theme_source.promote_document_id;"));
    assert!(panes.contains("text: root.theme_source.promote_display_name;"));
    assert!(panes.contains(
        "edited(value) => { root.detail_event(\"theme_source\", \"theme.promote.asset_id.set\", -1, value, \"\"); }"
    ));
    assert!(panes.contains(
        "edited(value) => { root.detail_event(\"theme_source\", \"theme.promote.document_id.set\", -1, value, \"\"); }"
    ));
    assert!(panes.contains(
        "edited(value) => { root.detail_event(\"theme_source\", \"theme.promote.display_name.set\", -1, value, \"\"); }"
    ));

    assert!(theme_struct_block.contains("promote_asset_id: string,"));
    assert!(theme_struct_block.contains("promote_document_id: string,"));
    assert!(theme_struct_block.contains("promote_display_name: string,"));
    assert!(theme_struct_block.contains("can_edit_promote_draft: bool,"));
    assert!(pane_block.contains(
        "property <UiAssetThemeSourceData> theme_source: root.pane.theme_source;"
    ));
}

#[test]
fn ui_asset_editor_theme_panel_declares_detach_imported_theme_control() {
    let source = shell_source();
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );

    assert!(panes.contains("label: \"Detach\";"));
    assert!(panes.contains(
        "enabled: root.theme_source.selected_source_kind == \"Imported\" && root.theme_source.selected_source_available;"
    ));
    assert!(panes.contains(
        "clicked => { root.action(\"theme.source.detach_local\"); }"
    ));

    assert!(source.contains("callback ui_asset_action(instance_id: string, action_id: string);"));
    assert!(pane_block.contains(
        "property <UiAssetThemeSourceData> theme_source: root.pane.theme_source;"
    ));
}

#[test]
fn ui_asset_editor_theme_panel_declares_clone_imported_theme_control() {
    let panes = panes_source();

    assert!(panes.contains("label: \"Clone\";"));
    assert!(panes.contains(
        "clicked => { root.action(\"theme.source.clone_local\"); }"
    ));
    assert!(panes.contains(
        "enabled: root.theme_source.selected_source_kind == \"Imported\" && root.theme_source.selected_source_available;"
    ));
}

#[test]
fn ui_asset_editor_theme_panel_declares_cascade_inspection_controls() {
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(theme_struct_block.contains("cascade_layer_items: [string],"));
    assert!(theme_struct_block.contains("cascade_token_items: [string],"));
    assert!(theme_struct_block.contains("cascade_rule_items: [string],"));
    assert!(pane_block.contains(
        "property <UiAssetThemeSourceData> theme_source: root.pane.theme_source;"
    ));
    assert!(panes.contains("title: \"Cascade Layers\";"));
    assert!(panes.contains("items: root.theme_source.cascade_layer_items;"));
    assert!(panes.contains("title: \"Cascade Tokens\";"));
    assert!(panes.contains("items: root.theme_source.cascade_token_items;"));
    assert!(panes.contains("title: \"Cascade Rules\";"));
    assert!(panes.contains("items: root.theme_source.cascade_rule_items;"));
}

#[test]
fn ui_asset_editor_theme_panel_declares_local_merge_preview_controls() {
    let panes = panes_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(theme_struct_block.contains("merge_preview_items: [string],"));
    assert!(pane_block.contains(
        "property <UiAssetThemeSourceData> theme_source: root.pane.theme_source;"
    ));
    assert!(panes.contains("title: \"Merge Preview\";"));
    assert!(panes.contains("items: root.theme_source.merge_preview_items;"));
}
