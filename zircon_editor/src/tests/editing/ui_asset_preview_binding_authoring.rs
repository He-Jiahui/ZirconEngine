use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_binding"
version = 1
display_name = "Preview Binding"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", text_expr = "=status.text" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "MenuAction.SaveProject" }]
"##;

const PREVIEW_STATE_GRAPH_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_state_graph"
version = 1
display_name = "Preview State Graph"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { metadata = { title = "Ready", count = 1 }, items = ["Ready", "Dirty"] }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", text_expr = "=StatusLabel.metadata.title" }
"##;

const PREVIEW_BRACKET_EXPRESSION_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_bracket_expression"
version = 1
display_name = "Preview Bracket Expression"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { items = ["Ready", "Dirty"] }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", item_expr = "=StatusLabel.items[1]" }
"##;

const PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_deep_nested"
version = 1
display_name = "Preview Deep Nested"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { context = { dialog = { title = "Ready", steps = [{ label = "Plan" }, { label = "Dirty" }] } } }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", text_expr = "=StatusLabel.context.dialog.steps[1].label" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "Route.Form.ValueChanged" }]
"##;

const PREVIEW_FUNCTION_EXPRESSION_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.preview_function_expression"
version = 1
display_name = "Preview Function Expression"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
children = [{ child = "status" }, { child = "button" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready", subtitle = "", items = ["Ready", "Dirty"], metadata = { title = "Ready", severity = "Info" } }

[nodes.button]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save", summary_expr = "=concat(StatusLabel.text, \" / \", self.text)", fallback_expr = "=coalesce(StatusLabel.subtitle, StatusLabel.text, \"Unknown\")", item_count_expr = "=count(StatusLabel.items)", first_item_expr = "=first(StatusLabel.items)", last_item_expr = "=last(StatusLabel.items)", joined_items_expr = "=join(StatusLabel.items, \" | \")", status_matches_expr = "=eq(StatusLabel.text, \"Dirty\")", cta_expr = "=if(eq(StatusLabel.text, \"Dirty\"), \"Go\", \"Stop\")", metadata_title_expr = "=get(StatusLabel.metadata, \"title\")", review_item_expr = "=at(StatusLabel.items, 1)", has_title_expr = "=has(StatusLabel.metadata, \"title\")" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "MenuAction.SaveProject" }]
"##;

#[test]
fn ui_asset_editor_session_projects_preview_mock_subjects_and_expression_results() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    let initial = session.pane_presentation();
    assert_eq!(
        initial.preview_mock_subject_items,
        vec![
            "SaveButton • button".to_string(),
            "StatusLabel • status".to_string(),
        ]
    );
    assert_eq!(initial.preview_mock_subject_selected_index, 0);

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status preview subject"));
    assert!(session
        .select_preview_mock_property(0)
        .expect("select status text property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button preview subject"));
    let button_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text_expr"))
        .expect("expression property");
    assert!(session
        .select_preview_mock_property(button_index)
        .expect("select expression property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.preview_mock_kind, "Expression");
    assert_eq!(updated.preview_mock_expression_result, "Dirty");
}

#[test]
fn ui_asset_editor_session_projects_binding_target_suggestions_and_applies_them() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    session
        .select_hierarchy_index(2)
        .expect("select button node");

    let initial = session.pane_presentation();
    assert!(initial
        .inspector_binding_route_suggestion_items
        .iter()
        .any(|item| item.contains("MenuAction.SaveProject")));

    assert!(session
        .apply_selected_binding_route_suggestion(1)
        .expect("apply route suggestion"));
    let route_applied = session.pane_presentation();
    assert_ne!(
        route_applied.inspector_binding_route_target,
        "MenuAction.SaveProject"
    );

    assert!(session
        .select_binding_action_kind(2)
        .expect("switch to action binding kind"));
    let action_suggestions = session.pane_presentation();
    assert!(action_suggestions
        .inspector_binding_action_suggestion_items
        .iter()
        .any(|item| item.contains("EditorAction.SaveProject")));

    assert!(session
        .apply_selected_binding_action_suggestion(0)
        .expect("apply action suggestion"));
    let action_applied = session.pane_presentation();
    assert_eq!(
        action_applied.inspector_binding_action_target,
        "EditorAction.SaveProject"
    );
}

#[test]
fn ui_asset_editor_session_projects_expression_dependencies_into_preview_state_graph() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_state_graph.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_STATE_GRAPH_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview state graph session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let metadata_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("metadata [Object]"))
        .expect("metadata property");
    assert!(session
        .select_preview_mock_property(metadata_index)
        .expect("select metadata property"));
    assert!(session
        .select_preview_mock_nested_entry(1)
        .expect("select title nested entry"));
    assert!(session
        .set_selected_preview_mock_nested_value("Dirty")
        .expect("override metadata title"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));
    let expression_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text_expr"))
        .expect("text expression property");
    assert!(session
        .select_preview_mock_property(expression_index)
        .expect("select expression property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.preview_mock_expression_result, "Dirty");
    assert!(updated
        .preview_state_graph_items
        .contains(&"StatusLabel.metadata = { count = 1, title = \"Dirty\" }".to_string()));
    assert!(updated
        .preview_state_graph_items
        .contains(&"SaveButton.text_expr -> StatusLabel.metadata.title = \"Dirty\"".to_string()));
}

#[test]
fn ui_asset_editor_session_projects_preview_mock_schema_items_for_object_and_collection_values() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_state_graph.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_STATE_GRAPH_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview state graph session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));

    let metadata_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("metadata [Object]"))
        .expect("metadata property");
    assert!(session
        .select_preview_mock_property(metadata_index)
        .expect("select metadata property"));
    let object_schema = session.pane_presentation();
    assert_eq!(
        object_schema.preview_mock_schema_items,
        vec![
            "StatusLabel.metadata.count [Number]".to_string(),
            "StatusLabel.metadata.title [Text]".to_string(),
        ]
    );

    let items_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("items [Collection]"))
        .expect("items property");
    assert!(session
        .select_preview_mock_property(items_index)
        .expect("select items property"));
    let collection_schema = session.pane_presentation();
    assert_eq!(
        collection_schema.preview_mock_schema_items,
        vec![
            "StatusLabel.items[0] [Text]".to_string(),
            "StatusLabel.items[1] [Text]".to_string(),
            "StatusLabel.items[n] [Text]".to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_projects_binding_schema_items_for_route_and_action_targets() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    session
        .select_hierarchy_index(2)
        .expect("select button node");

    let route_schema = session.pane_presentation();
    assert_eq!(
        route_schema.inspector_binding_schema_items,
        vec![
            "event [UiEvent] = onClick".to_string(),
            "route.target [Route] = MenuAction.SaveProject".to_string(),
            "payload.confirm [Bool] default = true".to_string(),
            "payload.channel [Text] default = \"toolbar\"".to_string(),
            "payload.source [Text] default = \"ui.click\"".to_string(),
        ]
    );

    assert!(session
        .select_binding_action_kind(2)
        .expect("switch to action binding kind"));
    assert!(session
        .set_selected_binding_action_target("EditorAction.SaveProject")
        .expect("set action target"));
    let action_schema = session.pane_presentation();
    assert_eq!(
        action_schema.inspector_binding_schema_items,
        vec![
            "event [UiEvent] = onClick".to_string(),
            "action.target [EditorAction] = EditorAction.SaveProject".to_string(),
            "payload.confirm [Bool] default = true".to_string(),
            "payload.source [Text] default = \"ui.click\"".to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_evaluates_preview_mock_bracket_expression_paths() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_bracket_expression.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_BRACKET_EXPRESSION_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview bracket expression session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let items_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("items [Collection]"))
        .expect("items property");
    assert!(session
        .select_preview_mock_property(items_index)
        .expect("select items property"));
    assert!(session
        .select_preview_mock_nested_entry(1)
        .expect("select second collection entry"));
    assert!(session
        .set_selected_preview_mock_nested_value("Shipped")
        .expect("override second collection entry"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));
    let expression_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("item_expr"))
        .expect("item expression property");
    assert!(session
        .select_preview_mock_property(expression_index)
        .expect("select item expression property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.preview_mock_kind, "Expression");
    assert_eq!(updated.preview_mock_expression_result, "Shipped");
    assert!(updated
        .preview_state_graph_items
        .contains(&"StatusLabel.items = [\"Ready\", \"Shipped\"]".to_string()));
    assert!(updated
        .preview_state_graph_items
        .contains(&"SaveButton.item_expr -> StatusLabel.items[1] = \"Shipped\"".to_string()));
}

#[test]
fn ui_asset_editor_session_projects_target_aware_structured_binding_payload_schemas() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    session
        .select_hierarchy_index(2)
        .expect("select button node");

    assert!(session
        .select_binding_event_option(5)
        .expect("select change event"));
    assert!(session
        .apply_selected_binding_route_suggestion(0)
        .expect("apply selection changed route"));
    let route_payload = session.pane_presentation();
    assert_eq!(
        route_payload.inspector_binding_payload_suggestion_items,
        vec![
            "primary = \"SelectedNode\"".to_string(),
            "selection_ids = [\"SelectedNode\"]".to_string(),
            "context = { additive = false, source = \"hierarchy\" }".to_string(),
        ]
    );
    assert!(route_payload
        .inspector_binding_schema_items
        .contains(&"payload.selection_ids [Collection] default = [\"SelectedNode\"]".to_string()));
    assert!(route_payload
        .inspector_binding_schema_items
        .contains(&"payload.selection_ids[n] [Text] default = \"SelectedNode\"".to_string()));
    assert!(route_payload.inspector_binding_schema_items.contains(
        &"payload.context [Object] default = { additive = false, source = \"hierarchy\" }"
            .to_string()
    ));

    assert!(session
        .select_binding_event_option(7)
        .expect("select toggle event"));
    assert!(session
        .select_binding_action_kind(2)
        .expect("switch to action binding kind"));
    assert!(session
        .set_selected_binding_action_target("EditorAction.ToggleVisibility")
        .expect("set toggle visibility action"));
    let action_payload = session.pane_presentation();
    assert_eq!(
        action_payload.inspector_binding_payload_suggestion_items,
        vec![
            "checked = true".to_string(),
            "selection_ids = [\"SelectedNode\"]".to_string(),
            "context = { scope = \"selection\", source = \"ui.toggle\" }".to_string(),
        ]
    );
    assert!(action_payload
        .inspector_binding_schema_items
        .contains(&"payload.selection_ids [Collection] default = [\"SelectedNode\"]".to_string()));
    assert!(action_payload
        .inspector_binding_schema_items
        .contains(&"payload.selection_ids[n] [Text] default = \"SelectedNode\"".to_string()));
    assert!(action_payload.inspector_binding_schema_items.contains(
        &"payload.context [Object] default = { scope = \"selection\", source = \"ui.toggle\" }"
            .to_string()
    ));
}

#[test]
fn ui_asset_editor_session_projects_binding_expression_payload_previews_and_interaction_edges() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status preview subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text preview property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text preview property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload("status_text", "=StatusLabel.text")
        .expect("upsert binding expression payload"));
    assert!(session
        .upsert_selected_binding_payload(
            "context",
            "{ title = \"=StatusLabel.text\", dirty = true }",
        )
        .expect("upsert binding object payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.status_text [Expression] = \"=StatusLabel.text\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.status_text.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.preview [Object] = { dirty = true, title = \"Dirty\" }".to_string()
    ));
    assert!(pane
        .preview_state_graph_items
        .contains(&"SaveButton.onClick => MenuAction.SaveProject".to_string()));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.status_text -> StatusLabel.text = \"Dirty\"".to_string()
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.context.title -> StatusLabel.text = \"Dirty\"".to_string()
    ));
}

#[test]
fn ui_asset_editor_session_projects_nested_binding_payload_schema_previews() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status preview subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text preview property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text preview property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload(
            "context",
            "{ title = \"=StatusLabel.text\", steps = [\"Idle\", \"=StatusLabel.text\"] }",
        )
        .expect("upsert nested binding payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(
            &"payload.context [Object] = { steps = [\"Idle\", \"=StatusLabel.text\"], title = \"=StatusLabel.text\" }"
                .to_string()
        ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.title [Expression] = \"=StatusLabel.text\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.steps [Collection] = [\"Idle\", \"=StatusLabel.text\"]".to_string()
    ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[0] [Text] = \"Idle\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[1] [Expression] = \"=StatusLabel.text\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[1].preview [Text] = \"Dirty\"".to_string()));
}

#[test]
fn ui_asset_editor_session_projects_recursive_preview_mock_paths_and_nested_expression_results() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_deep_nested.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview deep nested session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let context_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("context [Object]"))
        .expect("context property");
    assert!(session
        .select_preview_mock_property(context_index)
        .expect("select context property"));
    let initial = session.pane_presentation();
    assert!(initial
        .preview_mock_schema_items
        .contains(&"StatusLabel.context.dialog.title [Text]".to_string()));
    assert!(initial
        .preview_mock_schema_items
        .contains(&"StatusLabel.context.dialog.steps[1].label [Text]".to_string()));
    let nested_index = initial
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("dialog.steps[1].label"))
        .expect("deep nested preview entry");
    assert!(session
        .select_preview_mock_nested_entry(nested_index)
        .expect("select deep nested preview entry"));
    assert!(session
        .set_selected_preview_mock_nested_value("Reviewed")
        .expect("set deep nested preview entry"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));
    let expression_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text_expr"))
        .expect("text expression property");
    assert!(session
        .select_preview_mock_property(expression_index)
        .expect("select expression property"));

    let updated = session.pane_presentation();
    assert_eq!(updated.preview_mock_kind, "Expression");
    assert_eq!(updated.preview_mock_expression_result, "Reviewed");
    assert!(updated
        .preview_state_graph_items
        .contains(
            &"StatusLabel.context = { dialog = { steps = [{ label = \"Plan\" }, { label = \"Reviewed\" }], title = \"Ready\" } }"
                .to_string()
        ));
    assert!(updated.preview_state_graph_items.contains(
        &"SaveButton.text_expr -> StatusLabel.context.dialog.steps[1].label = \"Reviewed\""
            .to_string()
    ));
}

#[test]
fn ui_asset_editor_session_projects_preview_mock_suggestions_relative_to_selected_nested_container_and_applies_them(
) {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_deep_nested.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview deep nested session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let context_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("context [Object]"))
        .expect("context property");
    assert!(session
        .select_preview_mock_property(context_index)
        .expect("select context property"));

    let initial = session.pane_presentation();
    assert_eq!(
        initial.preview_mock_suggestion_items,
        vec![
            "[0] = { label = \"Plan\" }".to_string(),
            "[1] = { label = \"Dirty\" }".to_string(),
            "[n] = { label = \"Plan\" }".to_string(),
        ]
    );

    let dialog_index = initial
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("dialog [Object]"))
        .expect("dialog nested entry");
    assert!(session
        .select_preview_mock_nested_entry(dialog_index)
        .expect("select dialog nested entry"));
    let dialog_scope = session.pane_presentation();
    assert_eq!(
        dialog_scope.preview_mock_suggestion_items,
        vec![
            "steps = [{ label = \"Plan\" }, { label = \"Dirty\" }]".to_string(),
            "title = \"Ready\"".to_string(),
        ]
    );

    let steps_index = dialog_scope
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("dialog.steps [Collection]"))
        .expect("dialog.steps nested entry");
    assert!(session
        .select_preview_mock_nested_entry(steps_index)
        .expect("select dialog.steps nested entry"));
    let steps_scope = session.pane_presentation();
    assert_eq!(
        steps_scope.preview_mock_suggestion_items,
        vec![
            "[0] = { label = \"Plan\" }".to_string(),
            "[1] = { label = \"Dirty\" }".to_string(),
            "[n] = { label = \"Plan\" }".to_string(),
        ]
    );

    assert!(session
        .apply_selected_preview_mock_suggestion(2)
        .expect("apply append suggestion"));
    let updated = session.pane_presentation();
    assert!(updated
        .preview_mock_nested_items
        .iter()
        .any(|item| item.contains("dialog.steps[2] [Object] = { label = \"Plan\" }")));
    assert_eq!(updated.preview_mock_nested_key, "dialog.steps[2]");
    assert_eq!(updated.preview_mock_nested_value, "{ label = \"Plan\" }");
    assert_eq!(
        updated.preview_mock_suggestion_items,
        vec!["label = \"Plan\"".to_string()]
    );
    assert!(updated
        .preview_state_graph_items
        .contains(
            &"StatusLabel.context = { dialog = { steps = [{ label = \"Plan\" }, { label = \"Dirty\" }, { label = \"Plan\" }], title = \"Ready\" } }"
                .to_string()
        ));
}

#[test]
fn ui_asset_editor_session_selects_preview_mock_schema_items_as_nested_authoring_targets() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_deep_nested.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview deep nested session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let context_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("context [Object]"))
        .expect("context property");
    assert!(session
        .select_preview_mock_property(context_index)
        .expect("select context property"));

    let schema_index = session
        .pane_presentation()
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("StatusLabel.context.dialog.steps[1].label"))
        .expect("deep nested preview entry");
    assert!(session
        .select_preview_mock_nested_entry(schema_index)
        .expect("select preview nested entry"));

    let selected = session.pane_presentation();
    assert_eq!(
        selected.preview_mock_nested_key,
        "dialog.steps[1].label".to_string()
    );
    assert_eq!(selected.preview_mock_nested_value, "Dirty".to_string());
}

#[test]
fn ui_asset_editor_session_selects_binding_schema_items_as_payload_authoring_targets() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status preview subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text preview property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text preview property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload(
            "context",
            "{ title = \"=StatusLabel.text\", steps = [\"Idle\", \"=StatusLabel.text\"] }",
        )
        .expect("upsert nested binding payload"));

    let schema_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context.steps[1] = \"=StatusLabel.text\""))
        .expect("binding payload expression item");
    assert!(session
        .select_binding_payload(schema_index)
        .expect("select binding payload item"));

    let selected = session.pane_presentation();
    assert_eq!(
        selected.inspector_binding_payload_key,
        "context.steps[1]".to_string()
    );
    assert_eq!(
        selected.inspector_binding_payload_value,
        "\"=StatusLabel.text\"".to_string()
    );
}

#[test]
fn ui_asset_editor_session_supports_recursive_binding_payload_paths_and_previews() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status preview subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text preview property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text preview property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload("context.dialog.title", "=StatusLabel.text")
        .expect("upsert nested dialog title payload"));
    assert!(session
        .upsert_selected_binding_payload("context.steps[0].label", "\"Queued\"")
        .expect("upsert queued step payload"));
    assert!(session
        .upsert_selected_binding_payload("context.steps[1].label", "=StatusLabel.text")
        .expect("upsert nested step payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.dialog.title = \"=StatusLabel.text\"")));
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.steps[1].label = \"=StatusLabel.text\"")));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.dialog.title [Expression] = \"=StatusLabel.text\"".to_string()
    ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.dialog.title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[1].label.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.context.dialog.title -> StatusLabel.text = \"Dirty\""
            .to_string()
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.context.steps[1].label -> StatusLabel.text = \"Dirty\""
            .to_string()
    ));

    let delete_index = pane
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context.steps[1].label = \"=StatusLabel.text\""))
        .expect("nested payload item");
    session
        .select_binding_payload(delete_index)
        .expect("select nested payload item");
    assert!(session
        .delete_selected_binding_payload()
        .expect("delete nested payload item"));

    let updated = session.pane_presentation();
    assert!(updated
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.dialog.title = \"=StatusLabel.text\"")));
    assert!(!updated
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.steps[1].label = \"=StatusLabel.text\"")));
}

#[test]
fn ui_asset_editor_session_evaluates_function_preview_expressions_and_binding_payload_previews() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_function_expression.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_FUNCTION_EXPRESSION_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview function expression session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));
    let summary_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("summary_expr"))
        .expect("summary expression property");
    assert!(session
        .select_preview_mock_property(summary_index)
        .expect("select summary expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Dirty / Save"
    );

    let fallback_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("fallback_expr"))
        .expect("fallback expression property");
    assert!(session
        .select_preview_mock_property(fallback_index)
        .expect("select fallback expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Dirty"
    );

    let count_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("item_count_expr"))
        .expect("count expression property");
    assert!(session
        .select_preview_mock_property(count_index)
        .expect("select count expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "2"
    );

    session
        .select_hierarchy_index(2)
        .expect("select button node from hierarchy");
    assert!(session
        .upsert_selected_binding_payload(
            "summary",
            r#"'=concat(StatusLabel.text, " / ", self.text)'"#,
        )
        .expect("upsert concat binding payload"));
    assert!(session
        .upsert_selected_binding_payload(
            "fallback",
            r#"'=coalesce(StatusLabel.subtitle, StatusLabel.text, "Unknown")'"#,
        )
        .expect("upsert coalesce binding payload"));
    assert!(session
        .upsert_selected_binding_payload("item_count", r#"'=count(StatusLabel.items)'"#)
        .expect("upsert count binding payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .preview_state_graph_items
        .contains(&"SaveButton.summary_expr -> StatusLabel.text = \"Dirty\"".to_string()));
    assert!(pane
        .preview_state_graph_items
        .contains(&"SaveButton.summary_expr -> SaveButton.text = \"Save\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.summary.preview [Text] = \"Dirty / Save\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.fallback.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.item_count.preview [Number] = 2".to_string()));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.summary -> StatusLabel.text = \"Dirty\"".to_string()
    ));
    assert!(pane
        .preview_state_graph_items
        .contains(&"SaveButton.onClick.payload.summary -> SaveButton.text = \"Save\"".to_string()));
}

#[test]
fn ui_asset_editor_session_evaluates_collection_and_branch_preview_expressions() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_function_expression.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_FUNCTION_EXPRESSION_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview function expression session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    let items_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("items [Collection]"))
        .expect("items property");
    assert!(session
        .select_preview_mock_property(items_index)
        .expect("select items property"));
    assert!(session
        .set_selected_preview_mock_nested_value("Queued")
        .expect("override first item"));
    assert!(session
        .select_preview_mock_nested_entry(1)
        .expect("select second item"));
    assert!(session
        .set_selected_preview_mock_nested_value("Reviewed")
        .expect("override second item"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));

    let first_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("first_item_expr"))
        .expect("first item expression property");
    assert!(session
        .select_preview_mock_property(first_index)
        .expect("select first item expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Queued"
    );

    let last_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("last_item_expr"))
        .expect("last item expression property");
    assert!(session
        .select_preview_mock_property(last_index)
        .expect("select last item expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Reviewed"
    );

    let joined_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("joined_items_expr"))
        .expect("joined items expression property");
    assert!(session
        .select_preview_mock_property(joined_index)
        .expect("select joined items expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Queued | Reviewed"
    );

    let eq_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("status_matches_expr"))
        .expect("status matches expression property");
    assert!(session
        .select_preview_mock_property(eq_index)
        .expect("select status matches expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "true"
    );

    let if_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("cta_expr"))
        .expect("cta expression property");
    assert!(session
        .select_preview_mock_property(if_index)
        .expect("select cta expression property"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Go"
    );

    session
        .select_hierarchy_index(2)
        .expect("select button node from hierarchy");
    assert!(session
        .upsert_selected_binding_payload("first_item", r#"'=first(StatusLabel.items)'"#)
        .expect("upsert first item binding payload"));
    assert!(session
        .upsert_selected_binding_payload("joined_items", r#"'=join(StatusLabel.items, " | ")'"#,)
        .expect("upsert joined items binding payload"));
    assert!(session
        .upsert_selected_binding_payload("is_dirty", r#"'=eq(StatusLabel.text, "Dirty")'"#)
        .expect("upsert eq binding payload"));
    assert!(session
        .upsert_selected_binding_payload(
            "cta",
            r#"'=if(eq(StatusLabel.text, "Dirty"), "Go", "Stop")'"#,
        )
        .expect("upsert if binding payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.first_item.preview [Text] = \"Queued\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.joined_items.preview [Text] = \"Queued | Reviewed\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.is_dirty.preview [Bool] = true".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.cta.preview [Text] = \"Go\"".to_string()));
}

#[test]
fn ui_asset_editor_session_evaluates_accessor_preview_expressions_and_binding_payload_previews() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_function_expression.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_FUNCTION_EXPRESSION_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview function expression session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status subject"));
    let metadata_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("metadata [Object]"))
        .expect("metadata property");
    assert!(session
        .select_preview_mock_property(metadata_index)
        .expect("select metadata property"));
    let title_index = session
        .pane_presentation()
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("title [Text]"))
        .expect("metadata title");
    assert!(session
        .select_preview_mock_nested_entry(title_index)
        .expect("select metadata title"));
    assert!(session
        .set_selected_preview_mock_nested_value("Dirty")
        .expect("override metadata title"));

    let items_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("items [Collection]"))
        .expect("items property");
    assert!(session
        .select_preview_mock_property(items_index)
        .expect("select items property"));
    let review_index = session
        .pane_presentation()
        .preview_mock_nested_items
        .iter()
        .position(|item| item.contains("[1] [Text]"))
        .expect("second item");
    assert!(session
        .select_preview_mock_nested_entry(review_index)
        .expect("select second item"));
    assert!(session
        .set_selected_preview_mock_nested_value("Reviewed")
        .expect("override second item"));

    assert!(session
        .select_preview_mock_subject_node("button")
        .expect("select button subject"));

    let metadata_title_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("metadata_title_expr"))
        .expect("metadata title expression");
    assert!(session
        .select_preview_mock_property(metadata_title_index)
        .expect("select metadata title expression"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Dirty"
    );

    let review_item_expr_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("review_item_expr"))
        .expect("review item expression");
    assert!(session
        .select_preview_mock_property(review_item_expr_index)
        .expect("select review item expression"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "Reviewed"
    );

    let has_title_expr_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("has_title_expr"))
        .expect("has title expression");
    assert!(session
        .select_preview_mock_property(has_title_expr_index)
        .expect("select has title expression"));
    assert_eq!(
        session.pane_presentation().preview_mock_expression_result,
        "true"
    );

    session
        .select_hierarchy_index(2)
        .expect("select button node from hierarchy");
    assert!(session
        .upsert_selected_binding_payload(
            "metadata_title",
            r#"'=get(StatusLabel.metadata, "title")'"#,
        )
        .expect("upsert metadata title binding payload"));
    assert!(session
        .upsert_selected_binding_payload("review_item", r#"'=at(StatusLabel.items, 1)'"#)
        .expect("upsert review item binding payload"));
    assert!(session
        .upsert_selected_binding_payload("has_title", r#"'=has(StatusLabel.metadata, "title")'"#)
        .expect("upsert has title binding payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.metadata_title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.review_item.preview [Text] = \"Reviewed\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.has_title.preview [Bool] = true".to_string()));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.metadata_title_expr -> StatusLabel.metadata.title = \"Dirty\"".to_string(),
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.review_item_expr -> StatusLabel.items[1] = \"Reviewed\"".to_string(),
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.metadata_title -> StatusLabel.metadata.title = \"Dirty\""
            .to_string(),
    ));
    assert!(pane.preview_state_graph_items.contains(
        &"SaveButton.onClick.payload.review_item -> StatusLabel.items[1] = \"Reviewed\""
            .to_string(),
    ));
}

#[test]
fn ui_asset_editor_session_upserts_binding_payload_entries_relative_to_selected_container() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status preview subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text preview property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text preview property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload("context", "{ title = \"=StatusLabel.text\" }")
        .expect("upsert root context payload"));
    assert!(session
        .upsert_selected_binding_payload("subtitle", "\"Preview\"")
        .expect("upsert context subtitle payload"));
    let context_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| {
            item.contains("context = { subtitle = \"Preview\", title = \"=StatusLabel.text\" }")
        })
        .expect("context payload");
    session
        .select_binding_payload(context_index)
        .expect("reselect context payload");
    assert!(session
        .upsert_selected_binding_payload("steps", "[\"Plan\"]")
        .expect("upsert context steps payload"));

    let context_steps_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context.steps = [\"Plan\"]"))
        .expect("context steps payload");
    session
        .select_binding_payload(context_steps_index)
        .expect("select context steps payload");
    assert!(session
        .upsert_selected_binding_payload("", "\"Review\"")
        .expect("append selected collection payload entry"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context = { steps = [\"Plan\", \"Review\"], subtitle = \"Preview\", title = \"=StatusLabel.text\" }")));
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.subtitle = \"Preview\"")));
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.steps[1] = \"Review\"")));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.subtitle [Text] = \"Preview\"".to_string()));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[1] [Text] = \"Review\"".to_string()));
}

#[test]
fn ui_asset_editor_session_upserts_binding_payload_nested_relative_paths_from_selected_container() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_binding.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_AND_BINDING_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview binding session");

    assert!(session
        .select_preview_mock_subject_node("status")
        .expect("select status preview subject"));
    let status_text_index = session
        .pane_presentation()
        .preview_mock_items
        .iter()
        .position(|item| item.contains("text [Text]"))
        .expect("status text preview property");
    assert!(session
        .select_preview_mock_property(status_text_index)
        .expect("select status text preview property"));
    assert!(session
        .set_selected_preview_mock_value("Dirty")
        .expect("override status text preview"));

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload("context", "{}")
        .expect("upsert root context payload"));

    let context_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context = {}"))
        .expect("context payload");
    session
        .select_binding_payload(context_index)
        .expect("select context payload");
    assert!(session
        .upsert_selected_binding_payload("dialog.title", "=StatusLabel.text")
        .expect("upsert nested relative object payload"));

    let context_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context = { dialog = { title = \"=StatusLabel.text\" } }"))
        .expect("updated context payload");
    session
        .select_binding_payload(context_index)
        .expect("reselect updated context payload");
    assert!(session
        .upsert_selected_binding_payload("steps", "[]")
        .expect("upsert steps collection payload"));

    let steps_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context.steps = []"))
        .expect("steps payload");
    session
        .select_binding_payload(steps_index)
        .expect("select steps payload");
    assert!(session
        .upsert_selected_binding_payload("[0].label", "=StatusLabel.text")
        .expect("upsert indexed relative collection payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.dialog.title = \"=StatusLabel.text\"")));
    assert!(pane
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.steps[0].label = \"=StatusLabel.text\"")));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.dialog.title [Expression] = \"=StatusLabel.text\"".to_string()
    ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.dialog.title.preview [Text] = \"Dirty\"".to_string()));
    assert!(pane.inspector_binding_schema_items.contains(
        &"payload.context.steps[0].label [Expression] = \"=StatusLabel.text\"".to_string()
    ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[0].label.preview [Text] = \"Dirty\"".to_string()));
}

#[test]
fn ui_asset_editor_session_projects_binding_payload_suggestions_relative_to_selected_container() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_deep_nested.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview deep nested session");

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    let initial = session.pane_presentation();
    assert_eq!(
        initial.inspector_binding_payload_suggestion_items,
        vec![
            "value = \"preview\"".to_string(),
            "committed = true".to_string(),
            "fields = [\"title\"]".to_string(),
            "context = { source = \"ui.click\", subject = \"field\" }".to_string(),
        ]
    );

    assert!(session
        .upsert_selected_binding_payload("context", "{}")
        .expect("upsert context payload"));
    let context_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("context = {}"))
        .expect("context payload");
    session
        .select_binding_payload(context_index)
        .expect("select context payload");

    let context_pane = session.pane_presentation();
    assert_eq!(
        context_pane.inspector_binding_payload_suggestion_items,
        vec![
            "source = \"ui.click\"".to_string(),
            "subject = \"field\"".to_string(),
        ]
    );

    assert!(session
        .apply_selected_binding_payload_suggestion(1)
        .expect("apply contextual object suggestion"));
    let after_object = session.pane_presentation();
    assert!(after_object
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("context.subject = \"field\"")));
    assert_eq!(
        after_object.inspector_binding_payload_key,
        "context.subject".to_string()
    );

    assert!(session
        .upsert_selected_binding_payload("fields", "[]")
        .expect("upsert fields payload"));
    let fields_index = session
        .pane_presentation()
        .inspector_binding_payload_items
        .iter()
        .position(|item| item.contains("fields = []"))
        .expect("fields payload");
    session
        .select_binding_payload(fields_index)
        .expect("select fields payload");

    let fields_pane = session.pane_presentation();
    assert_eq!(
        fields_pane.inspector_binding_payload_suggestion_items,
        vec!["[0] = \"title\"".to_string(), "[1] = \"title\"".to_string()]
    );

    assert!(session
        .apply_selected_binding_payload_suggestion(0)
        .expect("apply contextual collection suggestion"));
    let after_collection = session.pane_presentation();
    assert!(after_collection
        .inspector_binding_payload_items
        .iter()
        .any(|item| item.contains("fields[0] = \"title\"")));
    assert_eq!(
        after_collection.inspector_binding_payload_key,
        "fields[0]".to_string()
    );
}

#[test]
fn ui_asset_editor_session_projects_collection_template_schema_paths_for_nested_binding_payloads() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/preview_deep_nested.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        PREVIEW_DEEP_NESTED_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("preview deep nested session");

    session
        .select_hierarchy_index(2)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload(
            "context",
            "{ steps = [{ label = \"=StatusLabel.context.dialog.steps[0].label\" }] }"
        )
        .expect("upsert nested collection payload"));

    let pane = session.pane_presentation();
    assert!(pane
        .inspector_binding_schema_items
        .contains(
            &"payload.context.steps [Collection] = [{ label = \"=StatusLabel.context.dialog.steps[0].label\" }]".to_string()
        ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(
            &"payload.context.steps[n] [Object] = { label = \"=StatusLabel.context.dialog.steps[0].label\" }".to_string()
        ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(
            &"payload.context.steps[n].label [Expression] = \"=StatusLabel.context.dialog.steps[0].label\"".to_string()
        ));
    assert!(pane
        .inspector_binding_schema_items
        .contains(&"payload.context.steps[n].label.preview [Text] = \"Plan\"".to_string()));
}
