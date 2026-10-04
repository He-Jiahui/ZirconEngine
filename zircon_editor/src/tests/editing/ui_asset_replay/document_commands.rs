use crate::ui::asset_editor::{
    UiAssetEditorDocumentReplayCommand, UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
};
use zircon_runtime::ui::template::UiAssetDocumentRuntimeExt;
use zircon_runtime_interface::ui::{
    binding::UiEventKind,
    layout::UiSize,
    template::{UiActionRef, UiAssetKind, UiBindingRef, UiNodeDefinitionKind},
};

use super::fixtures::{BINDING_REPLAY_LAYOUT_ASSET_TOML, WIDGET_PROMOTE_REPLAY_LAYOUT_ASSET_TOML};

#[test]
fn ui_asset_editor_session_binding_payload_authoring_uses_executable_binding_replay_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_binding_payload.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        BINDING_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("binding replay session");

    session
        .select_hierarchy_index(0)
        .expect("select button node");
    assert!(session
        .upsert_selected_binding_payload("status_text", "\"Dirty\"")
        .expect("upsert binding payload"));

    assert_eq!(
        session.next_undo_document_replay_commands(),
        vec![UiAssetEditorDocumentReplayCommand::SetNodeBindings {
            node_id: "root".to_string(),
            bindings: vec![UiBindingRef {
                component_event: None,
                id: "SaveButton/onClick".to_string(),
                event: UiEventKind::Click,
                mode: Default::default(),
                route: Some("menu_action.workbench.project.save".to_string()),
                action: None,
                targets: Vec::new(),
            }],
        }]
    );

    assert!(session.undo().expect("undo binding payload upsert"));
    assert_eq!(
        session.next_redo_document_replay_commands(),
        vec![UiAssetEditorDocumentReplayCommand::SetNodeBindings {
            node_id: "root".to_string(),
            bindings: vec![UiBindingRef {
                component_event: None,
                id: "SaveButton/onClick".to_string(),
                event: UiEventKind::Click,
                mode: Default::default(),
                route: Some("menu_action.workbench.project.save".to_string()),
                action: Some(UiActionRef {
                    route: Some("menu_action.workbench.project.save".to_string()),
                    action: None,
                    payload: [(
                        "status_text".to_string(),
                        toml::Value::String("Dirty".to_string()),
                    )]
                    .into_iter()
                    .collect(),
                    payload_missing_policy: Default::default(),
                }),
                targets: Vec::new(),
            }],
        }]
    );
}

#[test]
fn ui_asset_editor_session_tree_edits_use_executable_node_and_component_replay_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_tree.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );

    let mut wrap_session = UiAssetEditorSession::from_source(
        route.clone(),
        WIDGET_PROMOTE_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("wrap replay session");
    wrap_session
        .select_hierarchy_index(1)
        .expect("select button node");
    assert!(wrap_session
        .wrap_selected_node_with("VerticalBox")
        .expect("wrap selected node"));

    let wrap_undo_commands = wrap_session.next_undo_document_replay_commands();
    assert!(wrap_undo_commands.iter().any(|command| {
        matches!(
            command,
            UiAssetEditorDocumentReplayCommand::UpsertNode { node_id, node }
                if node_id == "root"
                    && node.children.len() == 1
                    && node.children[0].node.node_id == "button"
        )
    }));
    assert!(wrap_undo_commands.iter().any(|command| {
        matches!(
            command,
            UiAssetEditorDocumentReplayCommand::RemoveNode { node_id }
                if node_id.starts_with("verticalbox")
        )
    }));

    assert!(wrap_session.undo().expect("undo wrapped node"));
    let wrapped_undo_document =
        crate::tests::support::load_test_ui_asset(wrap_session.source_buffer().text())
            .expect("wrapped undo");
    assert_eq!(
        wrapped_undo_document
            .node("root")
            .expect("root node")
            .children[0]
            .node
            .node_id,
        "button"
    );

    assert!(wrap_session.redo().expect("redo wrapped node"));
    let wrapped_redo_document =
        crate::tests::support::load_test_ui_asset(wrap_session.source_buffer().text())
            .expect("wrapped redo");
    assert_ne!(
        wrapped_redo_document
            .node("root")
            .expect("root node")
            .children[0]
            .node
            .node_id,
        "button"
    );

    let mut extract_session = UiAssetEditorSession::from_source(
        route,
        WIDGET_PROMOTE_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("extract replay session");
    extract_session
        .select_hierarchy_index(1)
        .expect("select button node");
    assert!(extract_session
        .extract_selected_node_to_component()
        .expect("extract selected node"));

    let extract_undo_commands = extract_session.next_undo_document_replay_commands();
    assert!(extract_undo_commands.iter().any(|command| {
        matches!(
            command,
            UiAssetEditorDocumentReplayCommand::UpsertNode { node_id, node }
                if node_id == "button" && node.kind == UiNodeDefinitionKind::Native
        )
    }));
    assert!(extract_undo_commands.iter().any(|command| {
        matches!(
            command,
            UiAssetEditorDocumentReplayCommand::RemoveComponent { component_name }
                if component_name == "SaveButton"
        )
    }));
    assert!(extract_undo_commands.iter().any(|command| {
        matches!(
            command,
            UiAssetEditorDocumentReplayCommand::RemoveNode { node_id }
                if node_id == "savebutton_root"
        )
    }));

    assert!(extract_session.undo().expect("undo extracted component"));
    let extracted_undo_document =
        crate::tests::support::load_test_ui_asset(extract_session.source_buffer().text())
            .expect("extracted undo");
    assert_eq!(
        extracted_undo_document
            .node("button")
            .expect("button node")
            .kind,
        UiNodeDefinitionKind::Native
    );
    assert!(!extracted_undo_document
        .components
        .contains_key("SaveButton"));

    assert!(extract_session.redo().expect("redo extracted component"));
    let extracted_redo_document =
        crate::tests::support::load_test_ui_asset(extract_session.source_buffer().text())
            .expect("extracted redo");
    assert_eq!(
        extracted_redo_document
            .node("button")
            .expect("button node")
            .kind,
        UiNodeDefinitionKind::Component
    );
    assert!(extracted_redo_document
        .components
        .contains_key("SaveButton"));
}
