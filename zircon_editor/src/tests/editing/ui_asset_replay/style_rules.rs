use crate::ui::asset_editor::{
    UiAssetEditorDocumentReplayCommand, UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
};
use zircon_runtime_interface::ui::{
    layout::UiSize,
    template::{UiAssetKind, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet},
};

use super::fixtures::{
    STYLE_RULE_INSERT_REPLAY_LAYOUT_ASSET_TOML, STYLE_RULE_REPLAY_LAYOUT_ASSET_TOML,
    THEME_RULE_VECTOR_IMPORTED_THEME_ASSET_TOML, THEME_RULE_VECTOR_REPLAY_LAYOUT_ASSET_TOML,
};

#[test]
fn ui_asset_editor_session_undo_and_redo_replay_style_rule_reorders() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_style_rules.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        STYLE_RULE_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay style rule session");

    session
        .select_stylesheet_rule(2)
        .expect("select disabled rule");
    assert!(session
        .move_selected_stylesheet_rule_up()
        .expect("move selected stylesheet rule up"));

    let reordered = session
        .canonical_source()
        .expect("canonical reordered source");
    let reordered_document = crate::tests::support::load_test_ui_asset(&reordered)
        .expect("parse reordered stylesheet source");
    assert_eq!(
        reordered_document.stylesheets[0]
            .rules
            .iter()
            .map(|rule| rule.selector.clone())
            .collect::<Vec<_>>(),
        vec![
            ".primary".to_string(),
            ".primary:disabled".to_string(),
            ".primary:hover".to_string(),
        ]
    );

    let undone = session.undo_replay().expect("undo replay");
    assert!(undone.changed);
    let undone_document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("parse undone stylesheet source");
    assert_eq!(
        undone_document.stylesheets[0]
            .rules
            .iter()
            .map(|rule| rule.selector.clone())
            .collect::<Vec<_>>(),
        vec![
            ".primary".to_string(),
            ".primary:hover".to_string(),
            ".primary:disabled".to_string(),
        ]
    );

    let redone = session.redo_replay().expect("redo replay");
    assert!(redone.changed);
    let redone_document = crate::tests::support::load_test_ui_asset(session.source_buffer().text())
        .expect("parse redone stylesheet source");
    assert_eq!(
        redone_document.stylesheets[0]
            .rules
            .iter()
            .map(|rule| rule.selector.clone())
            .collect::<Vec<_>>(),
        vec![
            ".primary".to_string(),
            ".primary:disabled".to_string(),
            ".primary:hover".to_string(),
        ]
    );
}

#[test]
fn ui_asset_editor_session_tracks_executable_replay_commands_for_style_rule_insert_delete_and_reorder(
) {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_style_rule_insert.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut insert_session = UiAssetEditorSession::from_source(
        route,
        STYLE_RULE_INSERT_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay style rule insert session");

    insert_session
        .select_hierarchy_index(0)
        .expect("select button node");
    assert!(insert_session
        .create_rule_from_selection()
        .expect("create stylesheet rule from selection"));
    assert_eq!(
        insert_session.next_undo_document_replay_commands(),
        vec![UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
            index: 0,
            stylesheet_id: "local_editor_rules".to_string(),
        }]
    );
    assert!(insert_session.undo().expect("undo created stylesheet rule"));
    assert_eq!(
        insert_session.next_redo_document_replay_commands(),
        vec![UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
            index: 0,
            stylesheet_id: "local_editor_rules".to_string(),
            stylesheet: Some(UiStyleSheet {
                id: "local_editor_rules".to_string(),
                rules: vec![UiStyleRule {
                    id: Some("save_button".to_string()),
                    selector: "#SaveButton".to_string(),
                    set: UiStyleDeclarationBlock::default(),
                }],
            }),
        }]
    );

    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_style_rules.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut rule_session = UiAssetEditorSession::from_source(
        route,
        STYLE_RULE_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay style rule command session");

    rule_session
        .select_stylesheet_rule(1)
        .expect("select hover rule");
    assert!(rule_session
        .delete_selected_stylesheet_rule()
        .expect("delete selected stylesheet rule"));
    assert_eq!(
        rule_session.next_undo_document_replay_commands(),
        vec![UiAssetEditorDocumentReplayCommand::InsertStyleRule {
            stylesheet_index: 0,
            index: 1,
            selector: ".primary:hover".to_string(),
            rule: Some(UiStyleRule {
                selector: ".primary:hover".to_string(),
                id: Some("primary_hover".to_string()),
                set: UiStyleDeclarationBlock {
                    self_values: [("text".to_string(), toml::Value::String("Hover".to_string()),)]
                        .into_iter()
                        .collect(),
                    slot: Default::default(),
                },
            }),
        }]
    );

    let mut reorder_session = UiAssetEditorSession::from_source(
        UiAssetEditorRoute::new(
            "res://ui/tests/replay_style_rules.zui",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        STYLE_RULE_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay style rule reorder session");
    reorder_session
        .select_stylesheet_rule(2)
        .expect("select disabled rule");
    assert!(reorder_session
        .move_selected_stylesheet_rule_up()
        .expect("move stylesheet rule up"));
    assert_eq!(
        reorder_session.next_undo_document_replay_commands(),
        vec![UiAssetEditorDocumentReplayCommand::MoveStyleRule {
            stylesheet_index: 0,
            from_index: 1,
            to_index: 2,
        }]
    );
}

#[test]
fn ui_asset_editor_session_theme_refactor_uses_style_rule_vector_replay_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme_rule_vector.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let imported_theme =
        crate::tests::support::load_test_ui_asset(THEME_RULE_VECTOR_IMPORTED_THEME_ASSET_TOML)
            .expect("imported theme");
    let mut session = UiAssetEditorSession::from_source(
        route,
        THEME_RULE_VECTOR_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("theme rule vector session");

    session
        .register_style_import("res://ui/theme/shared_theme.zui", imported_theme)
        .expect("register imported theme");

    let refactor_index = session
        .pane_presentation()
        .theme_refactor_items
        .iter()
        .position(|item| item == "duplicate local rule • local_theme • Button")
        .expect("duplicate local rule refactor");
    assert!(session
        .apply_theme_refactor_item(refactor_index)
        .expect("apply duplicate local rule refactor"));

    assert_eq!(
        session.next_undo_document_replay_commands(),
        vec![UiAssetEditorDocumentReplayCommand::InsertStyleRule {
            stylesheet_index: 0,
            index: 0,
            selector: "Button".to_string(),
            rule: Some(UiStyleRule {
                id: None,
                selector: "Button".to_string(),
                set: UiStyleDeclarationBlock {
                    self_values: [(
                        "text".to_string(),
                        toml::Value::String("Imported Theme".to_string()),
                    )]
                    .into_iter()
                    .collect(),
                    slot: Default::default(),
                },
            }),
        }]
    );

    assert!(session.undo().expect("undo duplicate local rule refactor"));
    assert_eq!(
        session.next_redo_document_replay_commands(),
        vec![UiAssetEditorDocumentReplayCommand::RemoveStyleRule {
            stylesheet_index: 0,
            index: 0,
            selector: "Button".to_string(),
        }]
    );
}
