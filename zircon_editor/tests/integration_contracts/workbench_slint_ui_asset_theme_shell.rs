use std::fs;
use std::path::PathBuf;

fn ui_asset_editor_source() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    [
        "ui/workbench/pane_fields.slint",
        "ui/workbench/ui_asset_editor_data.slint",
        "ui/workbench/ui_asset_editor_components.slint",
        "ui/workbench/ui_asset_editor_center_column.slint",
        "ui/workbench/ui_asset_editor_inspector_panel.slint",
        "ui/workbench/ui_asset_editor_stylesheet_panel.slint",
        "ui/workbench/ui_asset_editor_pane.slint",
    ]
    .into_iter()
    .map(|relative| {
        fs::read_to_string(manifest_dir.join(relative))
            .unwrap_or_else(|_| panic!("{relative} should be readable"))
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn pane_surface_host_context_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("ui/workbench/pane_surface_host_context.slint");
    fs::read_to_string(path).expect("pane_surface_host_context.slint should be readable")
}

fn block_after<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker `{marker}`"));
    &source[start..]
}

#[test]
fn ui_asset_editor_theme_panel_declares_open_promote_and_local_detail_selection_controls() {
    let panes = ui_asset_editor_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(panes.contains("root.action(\"theme.source.open\");"));
    assert!(panes.contains("root.action(\"theme.local.promote\");"));
    assert!(panes.contains(
        "enabled: root.style_panel.theme_source.selected_source_kind == \"Imported\" && root.style_panel.theme_source.selected_source_available;"
    ));
    assert!(panes.contains(
        "enabled: root.style_panel.theme_source.selected_source_kind == \"Local\" && root.style_panel.theme_source.can_promote_local;"
    ));
    assert!(panes.contains("item_activated(item_index) => {"));
    assert!(panes.contains("root.action(\"theme.source.select.\" + item_index);"));
    assert!(panes.contains("selected_index: root.style_panel.theme_source.selected_source_kind == \"Local\" ? root.style_panel.token.selected_index : -1;"));
    assert!(panes.contains(
        "root.detail_event(\"style_token\", \"style.token.select\", item_index, \"\", \"\");"
    ));
    assert!(panes.contains("selected_index: root.style_panel.theme_source.selected_source_kind == \"Local\" ? root.style_panel.rule.selected_index : -1;"));
    assert!(panes.contains(
        "root.detail_event(\"style_rule\", \"style.rule.select\", item_index, \"\", \"\");"
    ));

    assert!(theme_struct_block.contains("can_promote_local: bool,"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
}

#[test]
fn ui_asset_editor_theme_panel_declares_promote_draft_controls() {
    let panes = ui_asset_editor_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(panes.contains("text: root.style_panel.theme_source.promote_asset_id;"));
    assert!(panes.contains("text: root.style_panel.theme_source.promote_document_id;"));
    assert!(panes.contains("text: root.style_panel.theme_source.promote_display_name;"));
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
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
}

#[test]
fn ui_asset_editor_theme_panel_declares_detach_imported_theme_control() {
    let source = pane_surface_host_context_source();
    let panes = ui_asset_editor_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );

    assert!(panes.contains("label: \"Detach\";"));
    assert!(panes.contains(
        "enabled: root.style_panel.theme_source.selected_source_kind == \"Imported\" && root.style_panel.theme_source.selected_source_available;"
    ));
    assert!(panes.contains("clicked => { root.action(\"theme.source.detach_local\"); }"));

    assert!(source.contains("callback ui_asset_action(instance_id: string, action_id: string);"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
}

#[test]
fn ui_asset_editor_theme_panel_declares_clone_imported_theme_control() {
    let panes = ui_asset_editor_source();

    assert!(panes.contains("label: \"Clone\";"));
    assert!(panes.contains("clicked => { root.action(\"theme.source.clone_local\"); }"));
    assert!(panes.contains(
        "enabled: root.style_panel.theme_source.selected_source_kind == \"Imported\" && root.style_panel.theme_source.selected_source_available;"
    ));
}

#[test]
fn ui_asset_editor_theme_panel_declares_cascade_inspection_controls() {
    let panes = ui_asset_editor_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(theme_struct_block.contains("cascade_layer_items: [string],"));
    assert!(theme_struct_block.contains("cascade_token_items: [string],"));
    assert!(theme_struct_block.contains("cascade_rule_items: [string],"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(panes.contains("title: \"Cascade Layers\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.cascade_layer_items;"));
    assert!(panes.contains("title: \"Cascade Tokens\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.cascade_token_items;"));
    assert!(panes.contains("title: \"Cascade Rules\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.cascade_rule_items;"));
}

#[test]
fn ui_asset_editor_theme_panel_declares_local_merge_preview_controls() {
    let panes = ui_asset_editor_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(theme_struct_block.contains("merge_preview_items: [string],"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(panes.contains("title: \"Merge Preview\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.merge_preview_items;"));
}

#[test]
fn ui_asset_editor_theme_panel_declares_compare_inspection_controls() {
    let panes = ui_asset_editor_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(theme_struct_block.contains("compare_items: [string],"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(panes.contains("title: \"Theme Compare\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.compare_items;"));
}

#[test]
fn ui_asset_editor_theme_panel_declares_rule_helper_and_refactor_controls() {
    let panes = ui_asset_editor_source();
    let pane_block = block_after(
        &panes,
        "export component UiAssetEditorPane inherits Rectangle {",
    );
    let theme_struct_block = block_after(&panes, "export struct UiAssetThemeSourceData {");

    assert!(theme_struct_block.contains("rule_helper_items: [string],"));
    assert!(theme_struct_block.contains("refactor_items: [string],"));
    assert!(theme_struct_block.contains("can_prune_duplicate_local_overrides: bool,"));
    assert!(pane_block.contains("property <UiAssetStylePanelData> style_panel: root.pane.style;"));
    assert!(panes.contains("title: \"Rule Helpers\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.rule_helper_items;"));
    assert!(panes.contains(
        "root.detail_event(\"theme_source\", \"theme.rule_helper.apply\", item_index, \"\", \"\");"
    ));
    assert!(panes.contains("title: \"Batch Refactor\";"));
    assert!(panes.contains("items: root.style_panel.theme_source.refactor_items;"));
    assert!(panes.contains(
        "root.detail_event(\"theme_source\", \"theme.refactor.apply\", item_index, \"\", \"\");"
    ));
    assert!(panes.contains("label: \"Prune Duplicates\";"));
    assert!(panes.contains("clicked => { root.action(\"theme.local.prune_duplicates\"); }"));
}
