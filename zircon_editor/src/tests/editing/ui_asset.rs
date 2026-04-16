use crate::editing::ui_asset::{UiAssetEditorCommand, UiAssetEditorSession};
use crate::{UiAssetEditorMode, UiAssetEditorRoute, UiDesignerSelectionModel, UiSize};
use zircon_ui::{UiAssetKind, UiAssetLoader};

const SIMPLE_LAYOUT_ASSET_TOML: &str = r#"
[asset]
kind = "layout"
id = "editor.test.layout"
version = 1
display_name = "Test Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "status" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }
layout = { width = { stretch = "Stretch" }, height = { min = 24.0, preferred = 24.0, max = 24.0, stretch = "Fixed" } }
"#;

const STYLED_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.styled_layout"
version = 1
display_name = "Styled Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
classes = ["shell"]
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }

[[stylesheets]]
id = "local"

[[stylesheets.rules]]
selector = ".primary"
set = { self.background = { color = "#224488" } }

[[stylesheets.rules]]
selector = "VerticalBox > Button.primary"
set = { self.text = { color = "#ffffff" } }
"##;

const STYLE_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.style_authoring"
version = 1
display_name = "Style Authoring Layout"

[tokens]
accent = "#4488ff"
panel_gap = 12

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }
style_overrides = { self = { text = { color = "#ffffff" } }, slot = { padding = 4 } }

[[stylesheets]]
id = "local"

[[stylesheets.rules]]
selector = ".primary"
set = { self.background = { color = "#224488" } }

[[stylesheets.rules]]
selector = ".primary:hover"
set = { self.text = { color = "#ffeeaa" } }

[[stylesheets.rules]]
selector = ".primary:disabled"
set = { self.background = { color = "#444444" } }
"##;

const SLOT_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.slot_authoring"
version = 1
display_name = "Slot Authoring Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button", mount = "actions", slot = { padding = 8, layout = { width = { preferred = 180 }, height = { preferred = 32 } } } }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
"##;

const LAYOUT_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.layout_authoring"
version = 1
display_name = "Layout Authoring Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
layout = { width = { preferred = 220 }, height = { preferred = 48 } }
"##;

const BINDING_AUTHORING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.test.binding_authoring"
version = 1
display_name = "Binding Authoring Layout"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "button" }]

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "MenuAction.SaveProject" }]
"##;

const IMPORTED_WIDGET_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.common.confirm_button"
version = 1
display_name = "Confirm Button"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "ConfirmButton"
props = { text = "Confirm" }
"##;

#[test]
fn ui_asset_editor_session_compiles_preview_surface_and_projects_reflection_state() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let session = UiAssetEditorSession::from_source(
        route.clone(),
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    assert_eq!(session.route(), &route);
    assert_eq!(session.preview_host().surface().tree.nodes.len(), 2);
    assert!(session.diagnostics().is_empty());
    assert_eq!(
        session.reflection_model().selection,
        UiDesignerSelectionModel::single("root")
    );
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Ready")
    );
}

#[test]
fn ui_asset_editor_session_preserves_last_good_preview_when_source_turns_invalid() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let edited = SIMPLE_LAYOUT_ASSET_TOML.replace("Ready", "Edited");

    session
        .apply_command(UiAssetEditorCommand::edit_source(edited.clone()))
        .expect("valid edit");
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );

    session
        .apply_command(UiAssetEditorCommand::edit_source("not valid toml"))
        .expect("source edit should still update buffer");

    assert_eq!(session.source_buffer().text(), "not valid toml");
    assert!(!session.diagnostics().is_empty());
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );
    assert!(session.reflection_model().preview_available);
    assert!(session.reflection_model().last_error.is_some());
}

#[test]
fn ui_asset_editor_session_undo_and_redo_source_edits_restore_preview_state() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Source,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SIMPLE_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let edited = SIMPLE_LAYOUT_ASSET_TOML.replace("Ready", "Edited");

    session
        .apply_command(UiAssetEditorCommand::edit_source(edited.clone()))
        .expect("valid edit");
    session
        .apply_command(UiAssetEditorCommand::edit_source("not valid toml"))
        .expect("invalid edit still tracked");

    assert!(session.can_undo());
    assert!(!session.can_redo());

    assert!(session.undo().expect("undo invalid edit"));
    assert_eq!(session.source_buffer().text(), edited);
    assert!(session.diagnostics().is_empty());
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );

    assert!(session.undo().expect("undo valid edit"));
    assert_eq!(session.source_buffer().text(), SIMPLE_LAYOUT_ASSET_TOML);
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Ready")
    );
    assert!(session.can_redo());

    assert!(session.redo().expect("redo valid edit"));
    assert_eq!(session.source_buffer().text(), edited);
    assert_eq!(
        selected_text(session.preview_host().surface(), "StatusLabel"),
        Some("Edited")
    );
}

#[test]
fn ui_asset_editor_session_switches_modes_and_updates_selection_from_hierarchy() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/styled-layout.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLED_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .set_mode(UiAssetEditorMode::Preview)
        .expect("preview mode");
    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let reflection = session.reflection_model();
    assert_eq!(reflection.route.mode, UiAssetEditorMode::Preview);
    assert_eq!(
        reflection.selection.primary_node_id.as_deref(),
        Some("button")
    );
    assert_eq!(
        reflection.style_inspector.selected_node_id.as_deref(),
        Some("button")
    );
    assert_eq!(
        reflection.style_inspector.classes,
        vec!["primary".to_string()]
    );
    assert_eq!(reflection.style_inspector.matched_rules.len(), 2);
    assert_eq!(
        reflection.style_inspector.matched_rules[0].selector,
        ".primary".to_string()
    );
    assert_eq!(
        reflection.style_inspector.matched_rules[1].selector,
        "VerticalBox > Button.primary".to_string()
    );
}

#[test]
fn ui_asset_editor_session_creates_stylesheet_rule_from_selected_node() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session.create_rule_from_selection().expect("create rule"));

    let reflection = session.reflection_model();
    assert!(reflection.source_dirty);
    assert!(reflection
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == "#SaveButton"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let created = document
        .stylesheets
        .first()
        .and_then(|sheet| sheet.rules.last())
        .expect("created rule");
    assert_eq!(created.selector, "#SaveButton");
    assert!(created.set.self_values.is_empty());
    assert!(created.set.slot.is_empty());
}

#[test]
fn ui_asset_editor_session_extracts_inline_overrides_into_stylesheet_rule() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .extract_inline_overrides_to_rule()
        .expect("extract inline overrides"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert!(button.style_overrides.self_values.is_empty());
    assert!(button.style_overrides.slot.is_empty());

    let extracted = document
        .stylesheets
        .first()
        .and_then(|sheet| sheet.rules.last())
        .expect("extracted rule");
    assert_eq!(extracted.selector, "#SaveButton");
    assert_eq!(
        extracted
            .set
            .self_values
            .get("text")
            .and_then(|value| value.get("color"))
            .and_then(toml::Value::as_str),
        Some("#ffffff")
    );
    assert_eq!(
        extracted
            .set
            .slot
            .get("padding")
            .and_then(toml::Value::as_integer),
        Some(4)
    );
}

#[test]
fn ui_asset_editor_session_adds_and_removes_selection_classes() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .add_class_to_selection("toolbar")
        .expect("add toolbar class"));
    assert_eq!(
        session.reflection_model().style_inspector.classes,
        vec!["primary".to_string(), "toolbar".to_string()]
    );
    assert!(session.reflection_model().source_dirty);
    assert!(!session
        .add_class_to_selection("toolbar")
        .expect("duplicate add should no-op"));

    assert!(session
        .remove_class_from_selection("primary")
        .expect("remove primary class"));
    assert_eq!(
        session.reflection_model().style_inspector.classes,
        vec!["toolbar".to_string()]
    );
    assert!(!session
        .remove_class_from_selection("missing")
        .expect("missing remove should no-op"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.classes, vec!["toolbar".to_string()]);
}

#[test]
fn ui_asset_editor_session_selects_renames_and_deletes_local_stylesheet_rules() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_rule_items,
        vec![
            ".primary".to_string(),
            ".primary:hover".to_string(),
            ".primary:disabled".to_string()
        ]
    );
    assert_eq!(initial_pane.style_rule_selected_index, -1);
    assert_eq!(initial_pane.style_selected_rule_selector, "");

    assert!(session
        .select_stylesheet_rule(1)
        .expect("select local stylesheet rule"));
    let selected = session.pane_presentation();
    assert_eq!(selected.style_rule_selected_index, 1);
    assert_eq!(selected.style_selected_rule_selector, ".primary:hover");

    assert!(session
        .rename_selected_stylesheet_rule("Button.toolbar:hover")
        .expect("rename selected stylesheet rule"));
    let renamed_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert_eq!(
        renamed_document.stylesheets[0].rules[1].selector,
        "Button.toolbar:hover"
    );
    let renamed = session.pane_presentation();
    assert_eq!(renamed.style_rule_selected_index, 1);
    assert_eq!(
        renamed.style_selected_rule_selector,
        "Button.toolbar:hover".to_string()
    );

    assert!(session
        .delete_selected_stylesheet_rule()
        .expect("delete selected stylesheet rule"));
    let after_delete =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let selectors = after_delete.stylesheets[0]
        .rules
        .iter()
        .map(|rule| rule.selector.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        selectors,
        vec![".primary".to_string(), ".primary:disabled".to_string()]
    );
    let deleted = session.pane_presentation();
    assert_eq!(
        deleted.style_rule_items,
        vec![".primary".to_string(), ".primary:disabled".to_string()]
    );
    assert_eq!(deleted.style_rule_selected_index, 1);
    assert_eq!(
        deleted.style_selected_rule_selector,
        ".primary:disabled".to_string()
    );
}

#[test]
fn ui_asset_editor_session_selects_upserts_and_deletes_stylesheet_rule_declarations() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    session
        .select_stylesheet_rule(0)
        .expect("select local stylesheet rule");

    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_rule_declaration_items,
        vec!["self.background.color = \"#224488\"".to_string()]
    );
    assert_eq!(initial_pane.style_rule_declaration_selected_index, -1);
    assert_eq!(initial_pane.style_selected_rule_declaration_path, "");
    assert_eq!(initial_pane.style_selected_rule_declaration_value, "");

    assert!(session
        .select_stylesheet_rule_declaration(0)
        .expect("select style declaration"));
    let selected = session.pane_presentation();
    assert_eq!(selected.style_rule_declaration_selected_index, 0);
    assert_eq!(
        selected.style_selected_rule_declaration_path,
        "self.background.color"
    );
    assert_eq!(
        selected.style_selected_rule_declaration_value,
        "\"#224488\""
    );

    assert!(session
        .upsert_selected_stylesheet_rule_declaration("slot.padding", "6")
        .expect("rename style declaration"));
    let updated_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let updated_rule = &updated_document.stylesheets[0].rules[0];
    assert!(updated_rule.set.self_values.is_empty());
    assert_eq!(
        updated_rule
            .set
            .slot
            .get("padding")
            .and_then(toml::Value::as_integer),
        Some(6)
    );
    let updated = session.pane_presentation();
    assert_eq!(
        updated.style_rule_declaration_items,
        vec!["slot.padding = 6".to_string()]
    );
    assert_eq!(updated.style_rule_declaration_selected_index, 0);
    assert_eq!(updated.style_selected_rule_declaration_path, "slot.padding");
    assert_eq!(updated.style_selected_rule_declaration_value, "6");

    assert!(session
        .delete_selected_stylesheet_rule_declaration()
        .expect("delete style declaration"));
    let deleted_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let deleted_rule = &deleted_document.stylesheets[0].rules[0];
    assert!(deleted_rule.set.self_values.is_empty());
    assert!(deleted_rule.set.slot.is_empty());
    let deleted = session.pane_presentation();
    assert!(deleted.style_rule_declaration_items.is_empty());
    assert_eq!(deleted.style_rule_declaration_selected_index, -1);
    assert_eq!(deleted.style_selected_rule_declaration_path, "");
    assert_eq!(deleted.style_selected_rule_declaration_value, "");
}

#[test]
fn ui_asset_editor_session_upserts_and_deletes_local_tokens() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel_gap = 12".to_string()
        ]
    );
    assert_eq!(initial_pane.style_token_selected_index, -1);
    assert_eq!(initial_pane.style_selected_token_name, "");
    assert_eq!(initial_pane.style_selected_token_value, "");

    assert!(session
        .upsert_style_token("surface_fill", "#223344")
        .expect("add token"));
    let added = session.pane_presentation();
    assert_eq!(
        added.style_token_items,
        vec![
            "accent = \"#4488ff\"".to_string(),
            "panel_gap = 12".to_string(),
            "surface_fill = \"#223344\"".to_string()
        ]
    );
    assert_eq!(added.style_token_selected_index, 2);
    assert_eq!(added.style_selected_token_name, "surface_fill");
    assert_eq!(added.style_selected_token_value, "\"#223344\"");

    assert!(session.select_style_token(0).expect("select accent token"));
    assert!(session
        .upsert_style_token("accent_primary", "#99bbff")
        .expect("rename selected token"));
    let renamed_document =
        UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert!(!renamed_document.tokens.contains_key("accent"));
    assert_eq!(
        renamed_document
            .tokens
            .get("accent_primary")
            .and_then(toml::Value::as_str),
        Some("#99bbff")
    );

    assert!(session
        .delete_selected_style_token()
        .expect("delete selected token"));
    let deleted = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert!(!deleted.tokens.contains_key("accent_primary"));
    assert_eq!(
        session.pane_presentation().style_token_items,
        vec![
            "panel_gap = 12".to_string(),
            "surface_fill = \"#223344\"".to_string()
        ]
    );
}

#[test]
fn ui_asset_editor_session_toggles_pseudo_state_preview_matches() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(!session.reflection_model().source_dirty);
    assert_eq!(
        session
            .reflection_model()
            .style_inspector
            .active_pseudo_states,
        Vec::<String>::new()
    );
    assert!(!session
        .reflection_model()
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));

    assert!(session
        .toggle_pseudo_state_preview("hover")
        .expect("enable hover preview"));
    let hover = session.reflection_model();
    assert_eq!(hover.style_inspector.active_pseudo_states, vec!["hover"]);
    assert!(hover
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));

    assert!(session
        .toggle_pseudo_state_preview("disabled")
        .expect("enable disabled preview"));
    let disabled = session.reflection_model();
    assert_eq!(
        disabled.style_inspector.active_pseudo_states,
        vec!["hover", "disabled"]
    );
    assert!(disabled
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:disabled"));

    assert!(session
        .toggle_pseudo_state_preview("hover")
        .expect("disable hover preview"));
    let toggled = session.reflection_model();
    assert_eq!(
        toggled.style_inspector.active_pseudo_states,
        vec!["disabled"]
    );
    assert!(!toggled
        .style_inspector
        .matched_rules
        .iter()
        .any(|rule| rule.selector == ".primary:hover"));
    assert!(!toggled.source_dirty);
}

#[test]
fn ui_asset_editor_session_projects_matched_style_rules_into_stylesheet_summary_items() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    session
        .toggle_pseudo_state_preview("hover")
        .expect("enable hover preview");

    let pane = session.pane_presentation();
    assert!(pane
        .stylesheet_items
        .contains(&"selection selector: #SaveButton".to_string()));
    assert!(pane.stylesheet_items.contains(&"states: hover".to_string()));
    assert!(pane
        .stylesheet_items
        .contains(&".primary (spec 10, order 0)".to_string()));
    assert!(pane
        .stylesheet_items
        .contains(&".primary:hover (spec 20, order 1)".to_string()));
}

#[test]
fn ui_asset_editor_session_selects_matched_style_rules_and_projects_details() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Preview,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    session
        .toggle_pseudo_state_preview("hover")
        .expect("enable hover preview");

    let initial_pane = session.pane_presentation();
    assert_eq!(
        initial_pane.style_matched_rule_items,
        vec![
            ".primary [editor.test.style_authoring::local]".to_string(),
            ".primary:hover [editor.test.style_authoring::local]".to_string()
        ]
    );
    assert_eq!(initial_pane.style_matched_rule_selected_index, -1);
    assert_eq!(initial_pane.style_selected_matched_rule_origin, "");
    assert_eq!(initial_pane.style_selected_matched_rule_selector, "");
    assert_eq!(initial_pane.style_selected_matched_rule_specificity, -1);
    assert_eq!(initial_pane.style_selected_matched_rule_source_order, -1);
    assert!(initial_pane
        .style_selected_matched_rule_declaration_items
        .is_empty());

    assert!(session
        .select_matched_style_rule(1)
        .expect("select matched style rule"));
    let selected_pane = session.pane_presentation();
    assert_eq!(selected_pane.style_matched_rule_selected_index, 1);
    assert_eq!(
        selected_pane.style_selected_matched_rule_origin,
        "editor.test.style_authoring::local"
    );
    assert_eq!(
        selected_pane.style_selected_matched_rule_selector,
        ".primary:hover"
    );
    assert_eq!(selected_pane.style_selected_matched_rule_specificity, 20);
    assert_eq!(selected_pane.style_selected_matched_rule_source_order, 1);
    assert_eq!(
        selected_pane.style_selected_matched_rule_declaration_items,
        vec!["self.text.color = \"#ffeeaa\"".to_string()]
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_widget_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_selected_node_id, "button");
    assert_eq!(pane.inspector_parent_node_id, "root");
    assert_eq!(pane.inspector_mount, "");
    assert_eq!(pane.inspector_widget_kind, "Native");
    assert_eq!(pane.inspector_widget_label, "Button");
    assert_eq!(pane.inspector_control_id, "SaveButton");
    assert_eq!(pane.inspector_text_prop, "Save");
    assert!(pane.inspector_can_edit_control_id);
    assert!(pane.inspector_can_edit_text_prop);
}

#[test]
fn ui_asset_editor_session_updates_selected_widget_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .set_selected_widget_control_id("ConfirmButton")
        .expect("set selected control id"));
    assert!(session
        .set_selected_widget_text_property("Confirm")
        .expect("set selected text property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_control_id, "ConfirmButton");
    assert_eq!(updated.inspector_text_prop, "Confirm");

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.control_id.as_deref(), Some("ConfirmButton"));
    assert_eq!(
        button.props.get("text").and_then(toml::Value::as_str),
        Some("Confirm")
    );
    assert!(preview_has_control_id(
        session.preview_host().surface(),
        "ConfirmButton"
    ));
}

#[test]
fn ui_asset_editor_session_projects_structured_slot_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/slot-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        SLOT_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_mount, "actions");
    assert_eq!(pane.inspector_slot_padding, "8");
    assert_eq!(pane.inspector_slot_width_preferred, "180");
    assert_eq!(pane.inspector_slot_height_preferred, "32");
}

#[test]
fn ui_asset_editor_session_updates_selected_slot_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .set_selected_slot_mount("footer")
        .expect("set selected mount"));
    assert!(session
        .set_selected_slot_padding("12")
        .expect("set selected slot padding"));
    assert!(session
        .set_selected_slot_width_preferred("240")
        .expect("set selected slot width preferred"));
    assert!(session
        .set_selected_slot_height_preferred("44")
        .expect("set selected slot height preferred"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_mount, "footer");
    assert_eq!(updated.inspector_slot_padding, "12");
    assert_eq!(updated.inspector_slot_width_preferred, "240");
    assert_eq!(updated.inspector_slot_height_preferred, "44");

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let child_mount = document.nodes["root"]
        .children
        .iter()
        .find(|child_mount| child_mount.child == "button")
        .expect("button child mount");
    assert_eq!(child_mount.mount.as_deref(), Some("footer"));
    assert_eq!(
        slot_value(&child_mount.slot, &["padding"]).and_then(toml::Value::as_integer),
        Some(12)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "width", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(240)
    );
    assert_eq!(
        slot_value(&child_mount.slot, &["layout", "height", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(44)
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_layout_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/layout-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LAYOUT_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(pane.inspector_layout_width_preferred, "220");
    assert_eq!(pane.inspector_layout_height_preferred, "48");
}

#[test]
fn ui_asset_editor_session_updates_selected_layout_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .set_selected_layout_width_preferred("220")
        .expect("set selected layout width preferred"));
    assert!(session
        .set_selected_layout_height_preferred("48")
        .expect("set selected layout height preferred"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_layout_width_preferred, "220");
    assert_eq!(updated.inspector_layout_height_preferred, "48");

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(
        layout_value(button.layout.as_ref(), &["width", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(220)
    );
    assert_eq!(
        layout_value(button.layout.as_ref(), &["height", "preferred"])
            .and_then(toml::Value::as_integer),
        Some(48)
    );
}

#[test]
fn ui_asset_editor_session_projects_structured_binding_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/binding-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        BINDING_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(
        pane.inspector_binding_items,
        vec!["onClick | SaveButton/onClick -> MenuAction.SaveProject".to_string()]
    );
    assert_eq!(pane.inspector_binding_selected_index, 0);
    assert_eq!(pane.inspector_binding_id, "SaveButton/onClick");
    assert_eq!(pane.inspector_binding_event, "onClick");
    assert_eq!(pane.inspector_binding_route, "MenuAction.SaveProject");
}

#[test]
fn ui_asset_editor_session_updates_selected_binding_inspector_fields() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session.add_binding().expect("add binding"));
    assert!(session
        .set_selected_binding_id("SaveButton/onHover")
        .expect("set selected binding id"));
    assert!(session
        .set_selected_binding_event("onHover")
        .expect("set selected binding event"));
    assert!(session
        .set_selected_binding_route("MenuAction.HighlightSave")
        .expect("set selected binding route"));

    let updated = session.pane_presentation();
    assert_eq!(updated.inspector_binding_selected_index, 0);
    assert_eq!(updated.inspector_binding_id, "SaveButton/onHover");
    assert_eq!(updated.inspector_binding_event, "onHover");
    assert_eq!(updated.inspector_binding_route, "MenuAction.HighlightSave");

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let button = document.nodes.get("button").expect("button node");
    assert_eq!(button.bindings.len(), 1);
    assert_eq!(button.bindings[0].id, "SaveButton/onHover");
    assert_eq!(button.bindings[0].event.to_string(), "onHover");
    assert_eq!(
        button.bindings[0].route.as_deref(),
        Some("MenuAction.HighlightSave")
    );
}

#[test]
fn ui_asset_editor_session_projects_selection_indices_and_source_block_summary() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Split,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");

    let pane = session.pane_presentation();
    assert_eq!(pane.hierarchy_selected_index, 1);
    assert!(pane.preview_selected_index >= 0);
    assert_eq!(pane.source_selected_block_label, "[nodes.button]");
    assert!(pane.source_selected_line > 0);
    assert!(pane.source_selected_excerpt.contains("[nodes.button]"));
    assert!(pane.source_roundtrip_status.contains("line"));
}

#[test]
fn ui_asset_editor_session_inserts_palette_items_and_tracks_tree_edits_in_undo_stack() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let original_source = session.source_buffer().text().to_string();
    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Native / Button")
        .expect("button palette item");

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(session
        .select_palette_index(palette_index)
        .expect("select palette item"));
    assert!(session
        .insert_selected_palette_item_as_child()
        .expect("insert button as child"));

    let inserted = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert!(inserted.nodes.contains_key("button_2"));
    assert_eq!(
        inserted
            .nodes
            .get("button_2")
            .and_then(|node| node.widget_type.as_deref()),
        Some("Button")
    );
    assert_eq!(
        inserted
            .nodes
            .get("button_2")
            .and_then(|node| node.props.get("text"))
            .and_then(toml::Value::as_str),
        Some("Button")
    );
    assert!(session.can_undo());

    assert!(session.undo().expect("undo tree edit"));
    assert_eq!(session.source_buffer().text(), original_source);
    assert!(session.can_redo());

    assert!(session.redo().expect("redo tree edit"));
    let redone = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert!(redone.nodes.contains_key("button_2"));
}

#[test]
fn ui_asset_editor_session_creates_reference_nodes_from_imported_widget_palette_entries() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");
    let imported_widget =
        UiAssetLoader::load_toml_str(IMPORTED_WIDGET_ASSET_TOML).expect("imported widget");
    let reference = "asset://ui/common/confirm_button.ui#ConfirmButton";
    session
        .register_widget_import(reference, imported_widget)
        .expect("register widget import");
    let palette_index = session
        .pane_presentation()
        .palette_items
        .iter()
        .position(|item| item == "Reference / ConfirmButton")
        .expect("reference palette item");

    session
        .select_hierarchy_index(0)
        .expect("select root from hierarchy");
    assert!(session
        .select_palette_index(palette_index)
        .expect("select reference palette item"));
    assert!(session
        .insert_selected_palette_item_as_child()
        .expect("insert reference node"));

    let document = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let reference_node = document
        .nodes
        .values()
        .find(|node| node.kind == zircon_ui::UiNodeDefinitionKind::Reference)
        .expect("reference node");
    assert_eq!(reference_node.component_ref.as_deref(), Some(reference));
}

#[test]
fn ui_asset_editor_session_wraps_and_unwraps_selected_node() {
    let route = UiAssetEditorRoute::new(
        "asset://ui/tests/style-authoring.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_AUTHORING_LAYOUT_ASSET_TOML,
        UiSize::new(640.0, 360.0),
    )
    .expect("session");

    session
        .select_hierarchy_index(1)
        .expect("select button from hierarchy");
    assert!(session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap selected node"));

    let wrapped = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    let wrapper_id = wrapped
        .nodes
        .get("root")
        .and_then(|node| node.children.first())
        .map(|child| child.child.clone())
        .expect("wrapper child");
    assert_ne!(wrapper_id, "button");
    assert_eq!(
        wrapped
            .nodes
            .get(&wrapper_id)
            .and_then(|node| node.widget_type.as_deref()),
        Some("VerticalBox")
    );
    assert_eq!(
        wrapped
            .nodes
            .get(&wrapper_id)
            .map(|node| node.children.iter().map(|child| child.child.clone()).collect::<Vec<_>>()),
        Some(vec!["button".to_string()])
    );

    assert!(session.unwrap_selected_node().expect("unwrap wrapper"));
    let unwrapped = UiAssetLoader::load_toml_str(session.source_buffer().text()).expect("document");
    assert_eq!(
        unwrapped
            .nodes
            .get("root")
            .map(|node| node.children.iter().map(|child| child.child.clone()).collect::<Vec<_>>()),
        Some(vec!["button".to_string()])
    );
}

fn selected_text<'a>(surface: &'a zircon_ui::UiSurface, control_id: &str) -> Option<&'a str> {
    surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some(control_id)
        })
        .and_then(|node| node.template_metadata.as_ref())
        .and_then(|metadata| metadata.attributes.get("text"))
        .and_then(|value| value.as_str())
}

fn preview_has_control_id(surface: &zircon_ui::UiSurface, control_id: &str) -> bool {
    surface.tree.nodes.values().any(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            == Some(control_id)
    })
}

fn slot_value<'a>(
    slot: &'a std::collections::BTreeMap<String, toml::Value>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let value = slot.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let toml::Value::Table(table) = value else {
        return None;
    };
    slot_table_value(table, rest)
}

fn layout_value<'a>(
    layout: Option<&'a std::collections::BTreeMap<String, toml::Value>>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let layout = layout?;
    slot_value(layout, path)
}

fn slot_table_value<'a>(
    table: &'a toml::map::Map<String, toml::Value>,
    path: &[&str],
) -> Option<&'a toml::Value> {
    let (first, rest) = path.split_first()?;
    let value = table.get(*first)?;
    if rest.is_empty() {
        return Some(value);
    }
    let toml::Value::Table(child) = value else {
        return None;
    };
    slot_table_value(child, rest)
}
