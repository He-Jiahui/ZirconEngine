use std::collections::BTreeMap;

use crate::ui::asset_editor::{
    apply_external_effects_to_asset_sources, UiAssetEditorDocumentReplayBundle,
    UiAssetEditorDocumentReplayCommand, UiAssetEditorExternalEffect, UiAssetEditorMode,
    UiAssetEditorReplayWorkspace, UiAssetEditorRoute, UiAssetEditorSession,
    UiAssetEditorSourceCursorSnapshot, UiAssetEditorUndoExternalEffects, UiAssetEditorUndoStack,
    UiDesignerSelectionModel,
};
use zircon_runtime::ui::template::{UiActionRef, UiBindingRef};
use zircon_runtime::ui::{
    binding::UiEventKind, layout::UiSize, template::UiAssetKind, template::UiNodeDefinitionKind,
    template::UiStyleDeclarationBlock, template::UiStyleRule, template::UiStyleSheet,
};

const LOCAL_THEME_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_theme"
version = 1
display_name = "Replay Theme"

[tokens]
accent = "#4488ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "RootLabel"
props = { text = "Replay Theme" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "#RootLabel"
set = { self = { text = "$accent" } }
"##;

const STYLE_RULE_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_style_rules"
version = 1
display_name = "Replay Style Rules"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
id = "primary"
selector = ".primary"
set = { self = { text = "Default" } }

[[stylesheets.rules]]
id = "primary_hover"
selector = ".primary:hover"
set = { self = { text = "Hover" } }

[[stylesheets.rules]]
id = "primary_disabled"
selector = ".primary:disabled"
set = { self = { text = "Disabled" } }
"##;

const STYLE_RULE_INSERT_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_style_rule_insert"
version = 1
display_name = "Replay Style Rule Insert"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
classes = ["primary"]
props = { text = "Save" }
"##;

const WIDGET_PROMOTE_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_widget_promote"
version = 1
display_name = "Replay Widget Promote"

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
"##;

const EXISTING_EXTERNAL_STYLE_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.replay_theme_existing"
version = 1
display_name = "Existing Replay Theme"

[tokens]
accent = "#223344"

[[stylesheets]]
id = "existing_theme"

[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "$accent" } }
"##;

const EXISTING_EXTERNAL_WIDGET_ASSET_TOML: &str = r##"
[asset]
kind = "widget"
id = "ui.widgets.save_button_existing"
version = 1
display_name = "Existing Save Button"

[root]
node = "button_root"

[components.SaveButton]
root = "button_root"

[nodes.button_root]
kind = "native"
type = "Button"
control_id = "ToolbarButton"
props = { text = "Existing Save" }
"##;

const THEME_RULE_VECTOR_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_theme_rule_vector"
version = 1
display_name = "Replay Theme Rule Vector"

[imports]
styles = ["res://ui/theme/shared_theme.ui.toml"]

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme" } }

[[stylesheets.rules]]
selector = "#SaveButton"
set = { self = { text = "Keep Local" } }
"##;

const THEME_RULE_VECTOR_IMPORTED_THEME_ASSET_TOML: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[[stylesheets]]
id = "shared_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "Imported Theme" } }
"##;

const BINDING_REPLAY_LAYOUT_ASSET_TOML: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_binding_payload"
version = 1
display_name = "Replay Binding Payload"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }
bindings = [{ id = "SaveButton/onClick", event = "Click", route = "MenuAction.SaveProject" }]
"##;

#[test]
fn ui_asset_editor_session_undo_and_redo_replay_return_applied_external_effects() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay theme session");

    let promoted_style = session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/replay_theme.ui.toml",
            "ui.theme.replay_theme",
            "Replay Theme",
        )
        .expect("promote local theme")
        .expect("promoted style document");
    let promoted_style_source =
        toml::to_string_pretty(&promoted_style).expect("serialize promoted style document");

    let undone = session.undo_replay().expect("undo replay");
    assert!(undone.changed);
    assert_eq!(undone.label, "Promote Local Theme");
    assert_eq!(
        undone.external_effects,
        vec![UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: "res://ui/themes/replay_theme.ui.toml".to_string(),
        }]
    );

    let redone = session.redo_replay().expect("redo replay");
    assert!(redone.changed);
    assert_eq!(redone.label, "Promote Local Theme");
    assert_eq!(
        redone.external_effects,
        vec![UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/themes/replay_theme.ui.toml".to_string(),
            source: promoted_style_source,
        }]
    );
}

#[test]
fn ui_asset_editor_session_undo_and_redo_replay_style_rule_reorders() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_style_rules.ui.toml",
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
        "res://ui/tests/replay_style_rule_insert.ui.toml",
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
        "res://ui/tests/replay_style_rules.ui.toml",
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
            "res://ui/tests/replay_style_rules.ui.toml",
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
fn ui_asset_editor_session_theme_promotion_emits_executable_theme_replay_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme.ui.toml",
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
            "res://ui/themes/replay_theme.ui.toml",
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
                reference: "res://ui/themes/replay_theme.ui.toml".to_string(),
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
                reference: "res://ui/themes/replay_theme.ui.toml".to_string(),
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
        "res://ui/tests/replay_widget_promote.ui.toml",
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
            "res://ui/widgets/save_button.ui.toml",
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
                    && reference == "res://ui/widgets/save_button.ui.toml#SaveButton"
        )
    }));

    let redo_commands = session.next_redo_document_replay_commands();
    assert!(redo_commands.iter().any(|command| {
        matches!(
            command,
            UiAssetEditorDocumentReplayCommand::InsertWidgetImport { index, reference }
                if *index == 0
                    && reference == "res://ui/widgets/save_button.ui.toml#SaveButton"
        )
    }));
}

#[test]
fn ui_asset_editor_external_effects_apply_to_asset_source_maps_in_order() {
    let mut asset_sources: BTreeMap<String, String> = [
        (
            "res://ui/theme/editor_base.ui.toml".to_string(),
            "[asset]\nid = \"ui.theme.editor_base\"\n".to_string(),
        ),
        (
            "res://ui/theme/editor_local.ui.toml".to_string(),
            "[asset]\nid = \"ui.theme.editor_local\"\n".to_string(),
        ),
    ]
    .into_iter()
    .collect();

    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[
            UiAssetEditorExternalEffect::RemoveAssetSource {
                asset_id: "res://ui/theme/editor_base.ui.toml".to_string(),
            },
            UiAssetEditorExternalEffect::UpsertAssetSource {
                asset_id: "res://ui/theme/editor_theme_clone.ui.toml".to_string(),
                source: "[asset]\nid = \"ui.theme.editor_theme_clone\"\n".to_string(),
            },
        ],
    ));
    assert!(!asset_sources.contains_key("res://ui/theme/editor_base.ui.toml"));
    assert_eq!(
        asset_sources.get("res://ui/theme/editor_theme_clone.ui.toml"),
        Some(&"[asset]\nid = \"ui.theme.editor_theme_clone\"\n".to_string())
    );

    assert!(!apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/theme/editor_theme_clone.ui.toml".to_string(),
            source: "[asset]\nid = \"ui.theme.editor_theme_clone\"\n".to_string(),
        }],
    ));
}

#[test]
fn ui_asset_editor_external_effects_restore_previous_asset_source_when_replaying_overwrite() {
    let asset_id = "res://ui/theme/editor_base.ui.toml".to_string();
    let previous_source = "[asset]\nid = \"ui.theme.editor_base\"\n".to_string();
    let overwritten_source = "[asset]\nid = \"ui.theme.editor_base.updated\"\n".to_string();
    let mut asset_sources: BTreeMap<String, String> = [(asset_id.clone(), previous_source.clone())]
        .into_iter()
        .collect();

    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: asset_id.clone(),
            source: overwritten_source.clone(),
        }],
    ));
    assert_eq!(asset_sources.get(&asset_id), Some(&overwritten_source));

    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: asset_id.clone(),
            source: previous_source.clone(),
        }],
    ));
    assert_eq!(asset_sources.get(&asset_id), Some(&previous_source));

    assert!(!apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &[UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id,
            source: previous_source,
        }],
    ));
}

#[test]
fn ui_asset_editor_session_replay_effects_can_rebuild_cross_file_asset_sources() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme.ui.toml",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("replay theme session");

    let promoted_style = session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/replay_theme.ui.toml",
            "ui.theme.replay_theme",
            "Replay Theme",
        )
        .expect("promote local theme")
        .expect("promoted style document");
    let promoted_style_source =
        toml::to_string_pretty(&promoted_style).expect("serialize promoted style document");
    let mut asset_sources: BTreeMap<String, String> = [(
        "res://ui/themes/replay_theme.ui.toml".to_string(),
        promoted_style_source.clone(),
    )]
    .into_iter()
    .collect();

    let undone = session.undo_replay().expect("undo replay");
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &undone.external_effects,
    ));
    assert!(!asset_sources.contains_key("res://ui/themes/replay_theme.ui.toml"));

    let redone = session.redo_replay().expect("redo replay");
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &redone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/themes/replay_theme.ui.toml"),
        Some(&promoted_style_source)
    );
}

#[test]
fn ui_asset_editor_session_theme_promotion_restore_effects_reinstate_existing_external_source() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme_restore.ui.toml",
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
        toml::to_string_pretty(&existing_document).expect("serialize existing external style");
    session
        .register_style_import("res://ui/themes/replay_theme.ui.toml", existing_document)
        .expect("register existing external style import");

    let promoted_style = session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/replay_theme.ui.toml",
            "ui.theme.replay_theme",
            "Replay Theme",
        )
        .expect("promote local theme over existing style")
        .expect("promoted style document");
    let promoted_style_source =
        toml::to_string_pretty(&promoted_style).expect("serialize promoted style document");

    assert_eq!(
        session.next_undo_external_effects(),
        vec![UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: "res://ui/themes/replay_theme.ui.toml".to_string(),
            source: existing_source.clone(),
        }]
    );

    let undone = session.undo_replay().expect("undo replay");
    assert_eq!(
        undone.external_effects,
        vec![UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: "res://ui/themes/replay_theme.ui.toml".to_string(),
            source: existing_source.clone(),
        }]
    );

    let mut asset_sources: BTreeMap<String, String> = [(
        "res://ui/themes/replay_theme.ui.toml".to_string(),
        promoted_style_source.clone(),
    )]
    .into_iter()
    .collect();
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &undone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/themes/replay_theme.ui.toml"),
        Some(&existing_source)
    );

    let redone = session.redo_replay().expect("redo replay");
    assert_eq!(
        redone.external_effects,
        vec![UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/themes/replay_theme.ui.toml".to_string(),
            source: promoted_style_source.clone(),
        }]
    );
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &redone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/themes/replay_theme.ui.toml"),
        Some(&promoted_style_source)
    );
}

#[test]
fn ui_asset_editor_replay_workspace_applies_stylesheet_insert_and_cross_file_effects() {
    let before_source = STYLE_RULE_INSERT_REPLAY_LAYOUT_ASSET_TOML.to_string();
    let before_document = crate::tests::support::load_test_ui_asset(&before_source)
        .expect("parse replay insert layout");
    let inserted_stylesheet = UiStyleSheet {
        id: "local_editor_rules".to_string(),
        rules: vec![UiStyleRule {
            id: None,
            selector: "#SaveButton".to_string(),
            set: UiStyleDeclarationBlock::default(),
        }],
    };
    let mut after_document = before_document.clone();
    after_document.stylesheets.push(inserted_stylesheet.clone());
    let after_source =
        toml::to_string_pretty(&after_document).expect("serialize replay insert layout");

    let before_selection = UiDesignerSelectionModel::default();
    let after_selection = UiDesignerSelectionModel::single("root");
    let before_cursor = UiAssetEditorSourceCursorSnapshot::default();
    let after_cursor = UiAssetEditorSourceCursorSnapshot {
        byte_offset: after_source
            .find("[[stylesheets]]")
            .unwrap_or(after_source.len()),
        anchor_node_id: Some("root".to_string()),
        line_offset: 1,
    };
    let generated_asset_id = "res://ui/theme/generated_insert.ui.toml".to_string();
    let generated_asset_source = "[asset]\nkind = \"style\"\nid = \"ui.theme.generated_insert\"\nversion = 1\ndisplay_name = \"Generated Insert\"\n".to_string();

    let mut stack = UiAssetEditorUndoStack::default();
    stack.push_edit(
        "Replay Workspace Insert",
        None,
        Some(UiAssetEditorDocumentReplayBundle {
            undo: vec![UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
                index: 0,
                stylesheet_id: inserted_stylesheet.id.clone(),
            }],
            redo: vec![UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
                index: 0,
                stylesheet_id: inserted_stylesheet.id.clone(),
                stylesheet: Some(inserted_stylesheet.clone()),
            }],
        }),
        before_source.clone(),
        before_selection.clone(),
        before_cursor.clone(),
        None,
        Some(before_document.clone()),
        after_source.clone(),
        after_selection.clone(),
        after_cursor.clone(),
        Some("local".to_string()),
        Some(after_document.clone()),
        UiAssetEditorUndoExternalEffects {
            undo: vec![UiAssetEditorExternalEffect::RemoveAssetSource {
                asset_id: generated_asset_id.clone(),
            }],
            redo: vec![UiAssetEditorExternalEffect::UpsertAssetSource {
                asset_id: generated_asset_id.clone(),
                source: generated_asset_source.clone(),
            }],
        },
    );

    let mut workspace = UiAssetEditorReplayWorkspace {
        source: after_source.clone(),
        document: after_document.clone(),
        selection: after_selection.clone(),
        source_cursor: after_cursor.clone(),
        selected_theme_source_key: Some("local".to_string()),
        selected_style_rule_id: None,
        asset_sources: BTreeMap::from([(
            generated_asset_id.clone(),
            generated_asset_source.clone(),
        )]),
    };

    let undo = stack.undo_record().expect("undo replay record");
    let undo_result = undo
        .transition
        .apply_to_workspace(&mut workspace)
        .expect("apply undo transition to workspace");
    assert_eq!(workspace.source, before_source);
    assert_eq!(workspace.document, before_document);
    assert_eq!(workspace.selection, before_selection);
    assert_eq!(workspace.source_cursor, before_cursor);
    assert_eq!(workspace.selected_theme_source_key, None);
    assert!(workspace.asset_sources.is_empty());
    assert!(undo_result.source_changed);
    assert!(undo_result.document_changed);
    assert!(undo_result.selection_changed);
    assert!(undo_result.source_cursor_changed);
    assert!(undo_result.theme_source_changed);
    assert!(undo_result.asset_sources_changed);

    let redo = stack.redo_record().expect("redo replay record");
    let redo_result = redo
        .transition
        .apply_to_workspace(&mut workspace)
        .expect("apply redo transition to workspace");
    assert_eq!(workspace.source, after_source);
    assert_eq!(workspace.document, after_document);
    assert_eq!(workspace.selection, after_selection);
    assert_eq!(workspace.source_cursor, after_cursor);
    assert_eq!(
        workspace.selected_theme_source_key,
        Some("local".to_string())
    );
    assert_eq!(
        workspace.asset_sources.get(&generated_asset_id),
        Some(&generated_asset_source)
    );
    assert!(redo_result.source_changed);
    assert!(redo_result.document_changed);
    assert!(redo_result.selection_changed);
    assert!(redo_result.source_cursor_changed);
    assert!(redo_result.theme_source_changed);
    assert!(redo_result.asset_sources_changed);
}

#[test]
fn ui_asset_editor_replay_workspace_applies_style_rule_reorders_from_document_commands() {
    let before_source = STYLE_RULE_REPLAY_LAYOUT_ASSET_TOML.to_string();
    let before_document = crate::tests::support::load_test_ui_asset(&before_source)
        .expect("parse replay rule reorder layout");
    let mut after_document = before_document.clone();
    let moved_rule = after_document.stylesheets[0].rules.remove(2);
    after_document.stylesheets[0].rules.insert(1, moved_rule);
    let after_source =
        toml::to_string_pretty(&after_document).expect("serialize replay reorder layout");

    let before_selection = UiDesignerSelectionModel::single("root");
    let after_selection = UiDesignerSelectionModel::single("root").with_mount("styles");
    let before_cursor = UiAssetEditorSourceCursorSnapshot {
        byte_offset: before_source.find(".primary:hover").unwrap_or_default(),
        anchor_node_id: Some("root".to_string()),
        line_offset: 0,
    };
    let after_cursor = UiAssetEditorSourceCursorSnapshot {
        byte_offset: after_source.find(".primary:disabled").unwrap_or_default(),
        anchor_node_id: Some("root".to_string()),
        line_offset: 2,
    };

    let mut stack = UiAssetEditorUndoStack::default();
    stack.push_edit_with_style_rule_selection(
        "Replay Workspace Reorder",
        None,
        Some(UiAssetEditorDocumentReplayBundle {
            undo: vec![UiAssetEditorDocumentReplayCommand::MoveStyleRule {
                stylesheet_index: 0,
                from_index: 1,
                to_index: 2,
            }],
            redo: vec![UiAssetEditorDocumentReplayCommand::MoveStyleRule {
                stylesheet_index: 0,
                from_index: 2,
                to_index: 1,
            }],
        }),
        before_source.clone(),
        before_selection.clone(),
        before_cursor.clone(),
        None,
        Some("primary_hover".to_string()),
        Some(before_document.clone()),
        after_source.clone(),
        after_selection.clone(),
        after_cursor.clone(),
        None,
        Some("primary_disabled".to_string()),
        Some(after_document.clone()),
        UiAssetEditorUndoExternalEffects::default(),
    );

    let mut workspace = UiAssetEditorReplayWorkspace {
        source: after_source.clone(),
        document: after_document.clone(),
        selection: after_selection.clone(),
        source_cursor: after_cursor.clone(),
        selected_theme_source_key: None,
        selected_style_rule_id: Some("primary_disabled".to_string()),
        asset_sources: BTreeMap::new(),
    };

    let undo = stack.undo_record().expect("undo replay record");
    let undo_result = undo
        .transition
        .apply_to_workspace(&mut workspace)
        .expect("apply undo transition to reorder workspace");
    assert_eq!(workspace.source, before_source);
    assert_eq!(
        workspace.document.stylesheets[0]
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
    assert_eq!(workspace.selection, before_selection);
    assert_eq!(workspace.source_cursor, before_cursor);
    assert_eq!(
        workspace.selected_style_rule_id,
        Some("primary_hover".to_string())
    );
    assert!(undo_result.source_changed);
    assert!(undo_result.document_changed);
    assert!(undo_result.selection_changed);
    assert!(undo_result.source_cursor_changed);
    assert!(!undo_result.theme_source_changed);
    assert!(undo_result.style_rule_selection_changed);
    assert!(!undo_result.asset_sources_changed);

    let redo = stack.redo_record().expect("redo replay record");
    let redo_result = redo
        .transition
        .apply_to_workspace(&mut workspace)
        .expect("apply redo transition to reorder workspace");
    assert_eq!(workspace.source, after_source);
    assert_eq!(
        workspace.document.stylesheets[0]
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
    assert_eq!(workspace.selection, after_selection);
    assert_eq!(workspace.source_cursor, after_cursor);
    assert_eq!(
        workspace.selected_style_rule_id,
        Some("primary_disabled".to_string())
    );
    assert!(redo_result.source_changed);
    assert!(redo_result.document_changed);
    assert!(redo_result.selection_changed);
    assert!(redo_result.source_cursor_changed);
    assert!(!redo_result.theme_source_changed);
    assert!(redo_result.style_rule_selection_changed);
    assert!(!redo_result.asset_sources_changed);
}

#[test]
fn ui_asset_editor_replay_workspace_applies_stylesheet_vector_replay_commands() {
    let before_source = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.replay_stylesheet_vector"
version = 1
display_name = "Replay Stylesheet Vector"

[imports]
styles = ["res://ui/theme/base.ui.toml", "res://ui/theme/local.ui.toml"]

[tokens]
accent = "#4488ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "RootLabel"
classes = ["footer"]
props = { text = "Replay" }

[[stylesheets]]
id = "base"

[[stylesheets.rules]]
selector = "Label"
set = { self = { text = "Base" } }

[[stylesheets]]
id = "detail"

[[stylesheets.rules]]
selector = "#RootLabel"
set = { self = { text = "$accent" } }

[[stylesheets]]
id = "footer"

[[stylesheets.rules]]
selector = ".footer"
set = { self = { text = "Footer" } }
"##
    .trim_start()
    .to_string();
    let before_document = crate::tests::support::load_test_ui_asset(&before_source)
        .expect("parse replay stylesheet vector");
    let mut after_document = before_document.clone();
    after_document.imports.styles = vec![
        "res://ui/theme/local.ui.toml".to_string(),
        "res://ui/theme/accent.ui.toml".to_string(),
    ];
    after_document.tokens = BTreeMap::from([
        (
            "accent".to_string(),
            toml::Value::String("#5599ff".to_string()),
        ),
        (
            "panel".to_string(),
            toml::Value::String("$accent".to_string()),
        ),
    ]);
    let _ = after_document.stylesheets.remove(0);
    let moved_footer = after_document.stylesheets.remove(1);
    after_document.stylesheets.insert(0, moved_footer);
    after_document.stylesheets.push(UiStyleSheet {
        id: "accent".to_string(),
        rules: vec![UiStyleRule {
            id: None,
            selector: ".accent".to_string(),
            set: UiStyleDeclarationBlock {
                self_values: [(
                    "text".to_string(),
                    toml::Value::String("$panel".to_string()),
                )]
                .into_iter()
                .collect(),
                slot: Default::default(),
            },
        }],
    });
    let after_source =
        toml::to_string_pretty(&after_document).expect("serialize replay stylesheet vector");

    let before_selection = UiDesignerSelectionModel::single("root");
    let after_selection = UiDesignerSelectionModel::single("root").with_mount("styles");
    let before_cursor = UiAssetEditorSourceCursorSnapshot {
        byte_offset: before_source.find("detail").unwrap_or_default(),
        anchor_node_id: Some("root".to_string()),
        line_offset: 0,
    };
    let after_cursor = UiAssetEditorSourceCursorSnapshot {
        byte_offset: after_source.find("accent").unwrap_or_default(),
        anchor_node_id: Some("root".to_string()),
        line_offset: 2,
    };

    let mut stack = UiAssetEditorUndoStack::default();
    stack.push_edit(
        "Replay Workspace Stylesheet Vector",
        None,
        Some(UiAssetEditorDocumentReplayBundle {
            undo: vec![
                UiAssetEditorDocumentReplayCommand::RemoveStyleImport {
                    index: 1,
                    reference: "res://ui/theme/accent.ui.toml".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::RemoveStyleImport {
                    index: 0,
                    reference: "res://ui/theme/local.ui.toml".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::RemoveStyleToken {
                    token_name: "panel".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::UpsertStyleToken {
                    token_name: "accent".to_string(),
                    value: toml::Value::String("#4488ff".to_string()),
                },
                UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
                    index: 2,
                    stylesheet_id: "accent".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::MoveStyleSheet {
                    from_index: 0,
                    to_index: 1,
                    stylesheet_id: "footer".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
                    index: 0,
                    stylesheet_id: "base".to_string(),
                    stylesheet: Some(before_document.stylesheets[0].clone()),
                },
            ],
            redo: vec![
                UiAssetEditorDocumentReplayCommand::InsertStyleImport {
                    index: 0,
                    reference: "res://ui/theme/local.ui.toml".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::InsertStyleImport {
                    index: 1,
                    reference: "res://ui/theme/accent.ui.toml".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::UpsertStyleToken {
                    token_name: "accent".to_string(),
                    value: toml::Value::String("#5599ff".to_string()),
                },
                UiAssetEditorDocumentReplayCommand::UpsertStyleToken {
                    token_name: "panel".to_string(),
                    value: toml::Value::String("$accent".to_string()),
                },
                UiAssetEditorDocumentReplayCommand::RemoveStyleSheet {
                    index: 0,
                    stylesheet_id: "base".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::MoveStyleSheet {
                    from_index: 1,
                    to_index: 0,
                    stylesheet_id: "footer".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::InsertStyleSheet {
                    index: 2,
                    stylesheet_id: "accent".to_string(),
                    stylesheet: after_document.stylesheets.get(2).cloned(),
                },
            ],
        }),
        before_source.clone(),
        before_selection.clone(),
        before_cursor.clone(),
        None,
        Some(before_document.clone()),
        after_source.clone(),
        after_selection.clone(),
        after_cursor.clone(),
        Some("local".to_string()),
        Some(after_document.clone()),
        UiAssetEditorUndoExternalEffects::default(),
    );

    let mut workspace = UiAssetEditorReplayWorkspace {
        source: after_source.clone(),
        document: after_document.clone(),
        selection: after_selection.clone(),
        source_cursor: after_cursor.clone(),
        selected_theme_source_key: Some("local".to_string()),
        selected_style_rule_id: None,
        asset_sources: BTreeMap::new(),
    };

    let undo = stack.undo_record().expect("undo replay record");
    let undo_result = undo
        .transition
        .apply_to_workspace(&mut workspace)
        .expect("apply undo transition to stylesheet vector workspace");
    assert_eq!(workspace.source, before_source);
    assert_eq!(workspace.document, before_document);
    assert_eq!(workspace.selection, before_selection);
    assert_eq!(workspace.source_cursor, before_cursor);
    assert_eq!(workspace.selected_theme_source_key, None);
    assert!(undo_result.source_changed);
    assert!(undo_result.document_changed);
    assert!(undo_result.selection_changed);
    assert!(undo_result.source_cursor_changed);
    assert!(undo_result.theme_source_changed);

    let redo = stack.redo_record().expect("redo replay record");
    let redo_result = redo
        .transition
        .apply_to_workspace(&mut workspace)
        .expect("apply redo transition to stylesheet vector workspace");
    assert_eq!(workspace.source, after_source);
    assert_eq!(workspace.document, after_document);
    assert_eq!(workspace.selection, after_selection);
    assert_eq!(workspace.source_cursor, after_cursor);
    assert_eq!(
        workspace.selected_theme_source_key,
        Some("local".to_string())
    );
    assert!(redo_result.source_changed);
    assert!(redo_result.document_changed);
    assert!(redo_result.selection_changed);
    assert!(redo_result.source_cursor_changed);
    assert!(redo_result.theme_source_changed);
}

#[test]
fn ui_asset_editor_session_undo_and_redo_replay_return_widget_promotion_external_effects() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_widget_promote.ui.toml",
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
            "res://ui/widgets/save_button.ui.toml",
            "SaveButton",
            "ui.widgets.save_button",
        )
        .expect("promote selected component")
        .expect("promoted widget");
    let promoted_widget_source =
        toml::to_string_pretty(&promoted_widget).expect("serialize promoted widget");

    let undone = session.undo_replay().expect("undo replay");
    assert!(undone.changed);
    assert_eq!(
        undone.external_effects,
        vec![UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
        }]
    );

    let redone = session.redo_replay().expect("redo replay");
    assert!(redone.changed);
    assert_eq!(
        redone.external_effects,
        vec![UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
            source: promoted_widget_source,
        }]
    );
}

#[test]
fn ui_asset_editor_session_widget_promotion_restore_effects_reinstate_existing_external_source() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_widget_promote_restore.ui.toml",
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
        toml::to_string_pretty(&existing_document).expect("serialize existing external widget");
    session
        .register_widget_import(
            "res://ui/widgets/save_button.ui.toml#SaveButton",
            existing_document,
        )
        .expect("register existing external widget import");

    let promoted_widget = session
        .promote_selected_component_to_external_widget(
            "res://ui/widgets/save_button.ui.toml",
            "SaveButton",
            "ui.widgets.save_button",
        )
        .expect("promote selected component over existing widget")
        .expect("promoted widget");
    let promoted_widget_source =
        toml::to_string_pretty(&promoted_widget).expect("serialize promoted widget");

    assert_eq!(
        session.next_undo_external_effects(),
        vec![UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
            source: existing_source.clone(),
        }]
    );

    let undone = session.undo_replay().expect("undo replay");
    assert_eq!(
        undone.external_effects,
        vec![UiAssetEditorExternalEffect::RestoreAssetSource {
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
            source: existing_source.clone(),
        }]
    );

    let mut asset_sources: BTreeMap<String, String> = [(
        "res://ui/widgets/save_button.ui.toml".to_string(),
        promoted_widget_source.clone(),
    )]
    .into_iter()
    .collect();
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &undone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/widgets/save_button.ui.toml"),
        Some(&existing_source)
    );

    let redone = session.redo_replay().expect("redo replay");
    assert_eq!(
        redone.external_effects,
        vec![UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/widgets/save_button.ui.toml".to_string(),
            source: promoted_widget_source.clone(),
        }]
    );
    assert!(apply_external_effects_to_asset_sources(
        &mut asset_sources,
        &redone.external_effects,
    ));
    assert_eq!(
        asset_sources.get("res://ui/widgets/save_button.ui.toml"),
        Some(&promoted_widget_source)
    );
}

#[test]
fn ui_asset_editor_session_theme_refactor_uses_style_rule_vector_replay_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_theme_rule_vector.ui.toml",
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
        .register_style_import("res://ui/theme/shared_theme.ui.toml", imported_theme)
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

#[test]
fn ui_asset_editor_session_binding_payload_authoring_uses_executable_binding_replay_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_binding_payload.ui.toml",
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
                id: "SaveButton/onClick".to_string(),
                event: UiEventKind::Click,
                route: Some("MenuAction.SaveProject".to_string()),
                action: None,
            }],
        }]
    );

    assert!(session.undo().expect("undo binding payload upsert"));
    assert_eq!(
        session.next_redo_document_replay_commands(),
        vec![UiAssetEditorDocumentReplayCommand::SetNodeBindings {
            node_id: "root".to_string(),
            bindings: vec![UiBindingRef {
                id: "SaveButton/onClick".to_string(),
                event: UiEventKind::Click,
                route: Some("MenuAction.SaveProject".to_string()),
                action: Some(UiActionRef {
                    route: Some("MenuAction.SaveProject".to_string()),
                    action: None,
                    payload: [(
                        "status_text".to_string(),
                        toml::Value::String("Dirty".to_string()),
                    )]
                    .into_iter()
                    .collect(),
                }),
            }],
        }]
    );
}

#[test]
fn ui_asset_editor_session_tree_edits_use_executable_node_and_component_replay_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_tree.ui.toml",
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
