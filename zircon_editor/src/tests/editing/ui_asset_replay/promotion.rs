use std::collections::BTreeMap;

use crate::ui::asset_editor::{
    apply_external_effects_to_asset_sources, UiAssetEditorDocumentReplayCommand,
    UiAssetEditorExternalEffect, UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
};
use zircon_runtime_interface::ui::{
    layout::UiSize,
    template::{UiAssetKind, UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet},
};

use super::fixtures::{
    EXISTING_EXTERNAL_STYLE_ASSET_TOML, EXISTING_EXTERNAL_WIDGET_ASSET_TOML,
    LOCAL_THEME_LAYOUT_ASSET_TOML, WIDGET_PROMOTE_REPLAY_LAYOUT_ASSET_TOML,
};

#[test]
fn ui_asset_editor_session_theme_promotion_emits_executable_theme_replay_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay theme session");

    assert!(session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/replay_theme.zui",
            "ui.theme.replay_theme",
            "Replay Theme",
        )
        .expect("promote local theme")
        .is_some());

    assert_eq!(
        session.next_undo_document_replay_commands(),
        vec![
            UiAssetEditorDocumentReplayCommand::RemoveStyleImport {
                index: 0,
                reference: "res://ui/themes/replay_theme.zui".to_string(),
            },
            UiAssetEditorDocumentReplayCommand::UpsertStyleToken {
                token_name: "accent".to_string(),
                value: toml::Value::String("#4488ff".to_string()),
            },
            UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
                index: 0,
                stylesheet_id: "local_theme".to_string(),
                stylesheet: Some(UiStyleSheet {
                    id: "local_theme".to_string(),
                    rules: vec![UiStyleRule {
                        id: None,
                        selector: "#RootLabel".to_string(),
                        set: UiStyleDeclarationBlock {
                            self_values: [(
                                "text".to_string(),
                                toml::Value::String("$accent".to_string()),
                            )]
                            .into_iter()
                            .collect(),
                            slot: Default::default(),
                        },
                    }],
                }),
            },
        ]
    );

    assert!(session.undo().expect("undo theme promotion"));
    assert_eq!(
        session.next_redo_document_replay_commands(),
        vec![
            UiAssetEditorDocumentReplayCommand::InsertStyleImport {
                index: 0,
                reference: "res://ui/themes/replay_theme.zui".to_string(),
            },
            UiAssetEditorDocumentReplayCommand::RemoveStyleToken {
                token_name: "accent".to_string(),
            },
            UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
                index: 0,
                stylesheet_id: "local_theme".to_string(),
            },
        ]
    );
}

#[test]
fn ui_asset_editor_session_widget_promotion_emits_executable_widget_import_replay_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_widget_promote.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        WIDGET_PROMOTE_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay widget promote session");

    session
        .select_hierarchy_index(1)
        .expect("select button node");
    assert!(session
        .extract_selected_node_to_component()
        .expect("extract button into component"));
    assert!(session
        .promote_selected_component_to_external_widget(
            "res://ui/widgets/save_button.zui",
            "SaveButton",
            "ui.widgets.save_button",
        )
        .expect("promote selected component")
        .is_some());

    let undo_commands = session.next_undo_document_replay_commands();
    assert!(undo_commands.iter().any(|command| {
        matches!(
            command,
            UiAssetEditorDocumentReplayCommand::RemoveWidgetImport { index, reference }
                if *index == 0
                    && reference == "res://ui/widgets/save_button.zui#SaveButton"
        )
    }));

    let redo_commands = session.next_redo_document_replay_commands();
    assert!(redo_commands.iter().any(|command| {
        matches!(
            command,
            UiAssetEditorDocumentReplayCommand::InsertWidgetImport { index, reference }
                if *index == 0
                    && reference == "res://ui/widgets/save_button.zui#SaveButton"
        )
    }));
}

#[test]
fn ui_asset_editor_session_theme_promotion_restore_effects_reinstate_existing_external_source() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme_restore.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay theme restore session");

    let existing_document =
        crate::tests::support::load_test_ui_asset(EXISTING_EXTERNAL_STYLE_ASSET_TOML)
            .expect("existing external style");
    let existing_source =
        crate::ui::asset_editor::serialize_authoring_document_as_v2(&existing_document)
            .expect("serialize existing external style as v2");
    session
        .register_style_import("res://ui/themes/replay_theme.zui", existing_document)
        .expect("register existing external style import");

    let promoted_style = session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/replay_theme.zui",
            "ui.theme.replay_theme",
            "Replay Theme",
        )
        .expect("promote local theme over existing style")
        .expect("promoted style document");
    let promoted_style_source =
        crate::ui::asset_editor::serialize_authoring_document_as_v2(&promoted_style)
            .expect("serialize promoted style document as v2");

    assert_eq!(
        session.next_undo_external_effects(),
        vec![UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: "res://ui/themes/replay_theme.zui".to_string(),
            source: existing_source.clone(),
        }]
    );

    let undone = session.undo_replay().expect("undo replay");
    assert_eq!(
        undone.external_effects,
        vec![UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: "res://ui/themes/replay_theme.zui".to_string(),
            source: existing_source.clone(),
        }]
    );

    let mut asset_sources: BTreeMap<String, String> = [(
        "res://ui/themes/replay_theme.zui".to_string(),
        promoted_style_source.clone(),
    )]
    .into_iter()
    .collect();
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &undone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/themes/replay_theme.zui"),
        Some(&existing_source)
    );

    let redone = session.redo_replay().expect("redo replay");
    assert_eq!(
        redone.external_effects,
        vec![UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/themes/replay_theme.zui".to_string(),
            source: promoted_style_source.clone(),
        }]
    );
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &redone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/themes/replay_theme.zui"),
        Some(&promoted_style_source)
    );
}

#[test]
fn ui_asset_editor_session_undo_and_redo_replay_return_widget_promotion_external_effects() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_widget_promote.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        WIDGET_PROMOTE_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay widget promote session");

    session
        .select_hierarchy_index(1)
        .expect("select button node");
    assert!(session
        .extract_selected_node_to_component()
        .expect("extract button into component"));
    let promoted_widget = session
        .promote_selected_component_to_external_widget(
            "res://ui/widgets/save_button.zui",
            "SaveButton",
            "ui.widgets.save_button",
        )
        .expect("promote selected component")
        .expect("promoted widget");
    let promoted_widget_source =
        crate::ui::asset_editor::serialize_authoring_document_as_v2(&promoted_widget)
            .expect("serialize promoted widget as v2");

    let undone = session.undo_replay().expect("undo replay");
    assert!(undone.changed);
    assert_eq!(
        undone.external_effects,
        vec![UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: "res://ui/widgets/save_button.zui".to_string(),
        }]
    );

    let redone = session.redo_replay().expect("redo replay");
    assert!(redone.changed);
    assert_eq!(
        redone.external_effects,
        vec![UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/widgets/save_button.zui".to_string(),
            source: promoted_widget_source,
        }]
    );
}

#[test]
fn ui_asset_editor_session_widget_promotion_restore_effects_reinstate_existing_external_source() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_widget_promote_restore.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        WIDGET_PROMOTE_REPLAY_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay widget restore session");

    session
        .select_hierarchy_index(1)
        .expect("select button node");
    assert!(session
        .extract_selected_node_to_component()
        .expect("extract button into component"));

    let existing_document =
        crate::tests::support::load_test_ui_asset(EXISTING_EXTERNAL_WIDGET_ASSET_TOML)
            .expect("existing external widget");
    let existing_source =
        crate::ui::asset_editor::serialize_authoring_document_as_v2(&existing_document)
            .expect("serialize existing external widget as v2");
    session
        .register_widget_import(
            "res://ui/widgets/save_button.zui#SaveButton",
            existing_document,
        )
        .expect("register existing external widget import");

    let promoted_widget = session
        .promote_selected_component_to_external_widget(
            "res://ui/widgets/save_button.zui",
            "SaveButton",
            "ui.widgets.save_button",
        )
        .expect("promote selected component over existing widget")
        .expect("promoted widget");
    let promoted_widget_source =
        crate::ui::asset_editor::serialize_authoring_document_as_v2(&promoted_widget)
            .expect("serialize promoted widget as v2");

    assert_eq!(
        session.next_undo_external_effects(),
        vec![UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: "res://ui/widgets/save_button.zui".to_string(),
            source: existing_source.clone(),
        }]
    );

    let undone = session.undo_replay().expect("undo replay");
    assert_eq!(
        undone.external_effects,
        vec![UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: "res://ui/widgets/save_button.zui".to_string(),
            source: existing_source.clone(),
        }]
    );

    let mut asset_sources: BTreeMap<String, String> = [(
        "res://ui/widgets/save_button.zui".to_string(),
        promoted_widget_source.clone(),
    )]
    .into_iter()
    .collect();
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &undone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/widgets/save_button.zui"),
        Some(&existing_source)
    );

    let redone = session.redo_replay().expect("redo replay");
    assert_eq!(
        redone.external_effects,
        vec![UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/widgets/save_button.zui".to_string(),
            source: promoted_widget_source.clone(),
        }]
    );
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &redone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/widgets/save_button.zui"),
        Some(&promoted_widget_source)
    );
}
