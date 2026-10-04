use std::collections::BTreeMap;

use crate::ui::asset_editor::{
    UiAssetEditorDocumentReplayBundle, UiAssetEditorDocumentReplayCommand,
    UiAssetEditorExternalEffect, UiAssetEditorReplayWorkspace, UiAssetEditorSourceCursorSnapshot,
    UiAssetEditorUndoExternalEffects, UiAssetEditorUndoStack, UiDesignerSelectionModel,
};
use zircon_runtime_interface::ui::template::{UiStyleDeclarationBlock, UiStyleRule, UiStyleSheet};

use super::fixtures::{
    STYLE_RULE_INSERT_REPLAY_LAYOUT_ASSET_TOML, STYLE_RULE_REPLAY_LAYOUT_ASSET_TOML,
};

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
    let generated_asset_id = "res://ui/theme/generated_insert.zui".to_string();
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
styles = ["res://ui/theme/base.zui", "res://ui/theme/local.zui"]

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
        "res://ui/theme/local.zui".to_string(),
        "res://ui/theme/accent.zui".to_string(),
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
                    reference: "res://ui/theme/accent.zui".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::RemoveStyleImport {
                    index: 0,
                    reference: "res://ui/theme/local.zui".to_string(),
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
                    reference: "res://ui/theme/local.zui".to_string(),
                },
                UiAssetEditorDocumentReplayCommand::InsertStyleImport {
                    index: 1,
                    reference: "res://ui/theme/accent.zui".to_string(),
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
