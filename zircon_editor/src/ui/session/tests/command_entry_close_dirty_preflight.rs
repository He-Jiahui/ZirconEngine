use crate::ui::asset_editor::{
    UiAssetEditorCommand, UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
};
use zircon_runtime::ui::v2::UiV2AssetLoader;
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

const VIEW: &str = r#"
[asset]
kind = "view"
id = "editor.test.close_dirty_preflight"
version = 2
display_name = "Before"

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
"#;

const COMPONENT: &str = r#"
[asset]
kind = "component"
id = "editor.test.close_dirty_preflight.component"
version = 2
display_name = "Imported Component"

[components.ImportedWidget]
root = "imported_root"

[nodes.imported_root]
component = "Text"
props = { text = "Imported" }
"#;

#[test]
fn failed_imported_widget_projection_keeps_command_source_unchanged() {
    let route = UiAssetEditorRoute::new(
        "editor.test.close_dirty_preflight",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_v2_source(route, VIEW, UiSize::new(640.0, 360.0))
        .expect("valid V2 view");
    let mut imported = UiV2AssetLoader::load_toml_str(COMPONENT).expect("valid V2 component");
    imported.components.get_mut("ImportedWidget").unwrap().root = "missing_root".into();
    session
        .v2_compiler_imports
        .widgets
        .insert("res://ui/widgets/imported.zui".to_string(), imported);
    let source_before = session.source_buffer().text().to_string();
    let revision_before = session.source_revision();

    assert!(session
        .apply_command(UiAssetEditorCommand::edit_source(
            VIEW.replace("Before", "After")
        ))
        .is_err());
    assert_eq!(session.source_buffer().text(), source_before);
    assert_eq!(session.source_revision(), revision_before);
}

fn replay_session(source: &str) -> UiAssetEditorSession {
    UiAssetEditorSession::from_v2_source(
        UiAssetEditorRoute::new(
            "editor.test.close_dirty_preflight",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        ),
        source,
        UiSize::new(640.0, 360.0),
    )
    .expect("valid V2 replay fixture")
}

fn assert_failed_replay_keeps_checkpoint(session: &mut UiAssetEditorSession, redo: bool) {
    let buffer = session.source_buffer.clone();
    let document = session.last_valid_document.clone();
    let v2_document = serde_json::to_value(&session.last_valid_v2_document).unwrap();
    let history = session.undo_stack.clone();
    let navigation = (
        session.selection.clone(),
        session.source_cursor_snapshot(),
        session.selected_theme_source_key.clone(),
        session.selected_style_rule_id.clone(),
    );
    let derived = (
        session.last_valid_source_text.clone(),
        session.last_valid_source_generation,
        session.palette_catalog_build_count,
        session.palette_catalog.entries().to_vec(),
        session.diagnostics.clone(),
    );
    assert!(if redo {
        session.redo_replay()
    } else {
        session.undo_replay()
    }
    .is_err());
    assert_eq!(
        session.source_buffer, buffer,
        "text, revision and saved text"
    );
    assert_eq!(session.last_valid_document, document);
    assert_eq!(
        serde_json::to_value(&session.last_valid_v2_document).unwrap(),
        v2_document
    );
    assert_eq!(session.undo_stack, history, "both history sides");
    assert_eq!(
        (
            session.selection.clone(),
            session.source_cursor_snapshot(),
            session.selected_theme_source_key.clone(),
            session.selected_style_rule_id.clone(),
        ),
        navigation,
    );
    assert_eq!(
        (
            session.last_valid_source_text.clone(),
            session.last_valid_source_generation,
            session.palette_catalog_build_count,
            session.palette_catalog.entries().to_vec(),
            session.diagnostics.clone(),
        ),
        derived,
    );
}

#[test]
fn failed_partial_document_replay_keeps_source_history_and_savepoint() {
    use crate::ui::asset_editor::{
        UiAssetEditorDocumentReplayBundle, UiAssetEditorDocumentReplayCommand,
        UiAssetEditorTreeEditKind,
    };
    use zircon_runtime_interface::ui::template::UiStyleRule;
    for redo in [false, true] {
        let mut session = replay_session(VIEW);
        let invalid = vec![
            UiAssetEditorDocumentReplayCommand::UpsertStyleToken {
                token_name: "temporary".into(),
                value: toml::Value::Integer(42),
            },
            UiAssetEditorDocumentReplayCommand::InsertStyleRule {
                stylesheet_index: 0,
                index: 0,
                selector: "Button#".into(),
                rule: Some(UiStyleRule {
                    id: Some("invalid".into()),
                    selector: "Button#".into(),
                    set: Default::default(),
                }),
            },
        ];
        let replay = if redo {
            UiAssetEditorDocumentReplayBundle {
                undo: Vec::new(),
                redo: invalid,
            }
        } else {
            UiAssetEditorDocumentReplayBundle {
                undo: invalid,
                redo: Vec::new(),
            }
        };
        session
            .apply_command(
                UiAssetEditorCommand::tree_edit(
                    UiAssetEditorTreeEditKind::DocumentEdit,
                    "Replay partial document",
                    VIEW.replace("Before", "After"),
                )
                .with_document_replay(replay),
            )
            .unwrap();
        if redo {
            assert!(session.undo().unwrap());
        }
        assert_failed_replay_keeps_checkpoint(&mut session, redo);
        assert!(!session.last_valid_document.tokens.contains_key("temporary"));
    }
}

#[test]
fn failed_late_palette_replay_keeps_checkpoint_and_can_retry_both_directions() {
    for redo in [false, true] {
        let mut session = replay_session(VIEW);
        session
            .apply_command(UiAssetEditorCommand::edit_source(
                VIEW.replace("Before", "After"),
            ))
            .unwrap();
        if redo {
            assert!(session.undo().unwrap());
        }
        let mut imported = UiV2AssetLoader::load_toml_str(COMPONENT).unwrap();
        imported.components.get_mut("ImportedWidget").unwrap().root = "missing_root".into();
        session
            .v2_compiler_imports
            .widgets
            .insert("res://ui/widgets/imported.zui".into(), imported);
        assert_failed_replay_keeps_checkpoint(&mut session, redo);
        session.v2_compiler_imports.widgets.clear();
        assert!(if redo {
            session.redo().unwrap()
        } else {
            session.undo().unwrap()
        });
        assert_eq!(
            session.asset_display_name(),
            if redo { "After" } else { "Before" }
        );
        assert!(if redo {
            session.undo().unwrap()
        } else {
            session.redo().unwrap()
        });
        assert_eq!(
            session.asset_display_name(),
            if redo { "Before" } else { "After" }
        );
    }
}

#[test]
fn invalid_source_history_remains_replayable_with_last_good_preview() {
    let mut session = replay_session(VIEW);
    let document = session.last_valid_document.clone();
    session
        .apply_command(UiAssetEditorCommand::edit_source("not valid toml"))
        .unwrap();
    assert!(!session.diagnostics().is_empty());
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), VIEW);
    assert!(!session.source_buffer().is_dirty());
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), "not valid toml");
    assert_eq!(session.last_valid_document, document);
    assert!(!session.diagnostics().is_empty());
}

fn authored_source(base: &str, name: &str) -> String {
    format!(
        "# authored UTF-8 注释🙂 {}\n{}\n# trailing {}\n",
        "x".repeat(1024),
        base.replace("Before", name),
        "界".repeat(1024),
    )
}

#[test]
fn canonical_save_reconnects_both_authored_source_neighbors() {
    let initial = authored_source(VIEW, "初始🙂");
    let middle = authored_source(VIEW, "保存🙂");
    let next = authored_source(VIEW, "后续🙂");
    let mut session = replay_session(&initial);
    session
        .apply_command(UiAssetEditorCommand::edit_source(middle.clone()))
        .unwrap();
    session
        .apply_command(UiAssetEditorCommand::edit_source(next.clone()))
        .unwrap();
    assert!(session.undo().unwrap());
    let records = session.undo_stack.replay_records();
    let canonical = session.canonical_source().unwrap();
    assert!(canonical.len() < middle.len());
    assert!(session
        .mark_canonical_source_persisted(session.source_revision(), canonical.clone())
        .unwrap());
    assert_eq!(session.undo_stack.replay_records(), records);
    assert_eq!(session.source_buffer().text(), canonical);
    assert!(!session.source_buffer().is_dirty());
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), initial);
    assert!(session.source_buffer().is_dirty());
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), canonical);
    assert!(!session.source_buffer().is_dirty());
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), next);
    assert!(session.source_buffer().is_dirty());
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), canonical);
    assert!(!session.source_buffer().is_dirty());
}

fn replay_view_with_label() -> String {
    format!(
        "{}\n[nodes.label]\ncomponent = \"Text\"\nprops = {{ text = \"文字🙂\" }}\n",
        VIEW.replace(
            "component = \"VerticalGroup\"",
            "component = \"VerticalGroup\"\nchildren = [{ node = \"label\" }]"
        ),
    )
}

#[test]
fn canonical_save_keeps_selection_only_runs_clean_on_both_history_sides() {
    use crate::ui::asset_editor::{
        UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind, UiDesignerSelectionModel,
    };
    let base = replay_view_with_label();
    let initial = authored_source(&base, "初始🙂");
    let middle = authored_source(&base, "保存🙂");
    let next = authored_source(&base, "后续🙂");
    let mut session = replay_session(&initial);
    session
        .apply_command(UiAssetEditorCommand::edit_source(middle.clone()))
        .unwrap();
    for index in 0..4 {
        let selected = if index % 2 == 0 { "label" } else { "root" };
        session
            .apply_command(UiAssetEditorCommand::tree_edit_structured_with_selection(
                UiAssetEditorTreeEdit::generic(UiAssetEditorTreeEditKind::DocumentEdit),
                format!("Select {selected} {index}"),
                middle.clone(),
                UiDesignerSelectionModel::single(selected),
            ))
            .unwrap();
        assert_eq!(session.selection.primary_node_id.as_deref(), Some(selected));
    }
    session
        .apply_command(UiAssetEditorCommand::edit_source(next.clone()))
        .unwrap();
    assert!(session.undo().unwrap());
    assert!(session.undo().unwrap());
    assert!(session.undo().unwrap());
    let records = session.undo_stack.replay_records();
    let canonical = session.canonical_source().unwrap();
    assert!(session
        .mark_canonical_source_persisted(session.source_revision(), canonical.clone())
        .unwrap());
    assert_eq!(session.undo_stack.replay_records(), records);
    for selected in ["label", "root"] {
        assert!(
            session.undo().unwrap(),
            "selection replay must synchronize its host"
        );
        assert_eq!(session.selection.primary_node_id.as_deref(), Some(selected));
        assert_eq!(session.source_buffer().text(), canonical);
        assert!(!session.source_buffer().is_dirty());
        assert!(canonical.is_char_boundary(session.source_cursor_byte_offset));
    }
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), initial);
    assert!(session.source_buffer().is_dirty());
    assert!(session.redo().unwrap());
    for selected in ["label", "root", "label", "root"] {
        assert!(session.redo().unwrap());
        assert_eq!(session.selection.primary_node_id.as_deref(), Some(selected));
        assert_eq!(session.source_buffer().text(), canonical);
        assert!(!session.source_buffer().is_dirty());
        assert!(canonical.is_char_boundary(session.source_cursor_byte_offset));
    }
    assert!(session.redo().unwrap());
    assert_eq!(session.source_buffer().text(), next);
    assert!(session.source_buffer().is_dirty());
    assert!(session.undo().unwrap());
    assert_eq!(session.source_buffer().text(), canonical);
    assert!(!session.source_buffer().is_dirty());
}

#[test]
fn canonical_save_keeps_a_wholly_source_neutral_history_clean() {
    use crate::ui::asset_editor::{UiAssetEditorTreeEditKind, UiDesignerSelectionModel};
    let source = authored_source(&replay_view_with_label(), "保存🙂");
    let mut session = replay_session(&source);
    for selected in ["label", "root", "label", "root"] {
        session
            .apply_command(UiAssetEditorCommand::tree_edit_with_selection(
                UiAssetEditorTreeEditKind::DocumentEdit,
                format!("Select {selected}"),
                source.clone(),
                UiDesignerSelectionModel::single(selected),
            ))
            .unwrap();
        assert_eq!(session.selection.primary_node_id.as_deref(), Some(selected));
    }
    assert!(session.undo().unwrap());
    assert!(session.undo().unwrap());
    let records = session.undo_stack.replay_records();
    let canonical = session.canonical_source().unwrap();
    session
        .mark_canonical_source_persisted(session.source_revision(), canonical.clone())
        .unwrap();
    assert_eq!(session.undo_stack.replay_records(), records);
    for (redo, selected) in [
        (false, "label"),
        (false, "root"),
        (true, "label"),
        (true, "root"),
        (true, "label"),
        (true, "root"),
    ] {
        assert!(if redo {
            session.redo().unwrap()
        } else {
            session.undo().unwrap()
        });
        assert_eq!(session.selection.primary_node_id.as_deref(), Some(selected));
        assert_eq!(session.source_buffer().text(), canonical);
        assert!(!session.source_buffer().is_dirty());
        assert!(canonical.is_char_boundary(session.source_cursor_byte_offset));
    }
    assert!(!session.undo_stack.can_redo());
}

#[test]
fn canonical_save_refreshes_empty_history_source_ranges_and_utf8_cursor() {
    let authored = authored_source(VIEW, "保存🙂");
    let mut session = replay_session(&authored);
    session.source_cursor_byte_offset = authored.find('🙂').unwrap() + 1;
    let canonical = session.canonical_source().unwrap();
    assert!(session
        .mark_canonical_source_persisted(session.source_revision(), canonical.clone())
        .unwrap());
    let fresh = replay_session(&canonical);
    assert_eq!(session.last_valid_source_text, canonical);
    assert!(canonical.is_char_boundary(session.source_cursor_byte_offset));
    assert!(session
        .source_outline_cache
        .borrow()
        .is_current(session.source_revision()));
    assert!(session
        .last_valid_source_outline_cache
        .borrow()
        .is_current(session.last_valid_source_generation));
    assert_eq!(
        session.source_outline_index().entry_for_node("root"),
        fresh.source_outline_index().entry_for_node("root")
    );
    assert!(!session.undo_stack.can_undo());
    assert!(!session.undo_stack.can_redo());
    assert!(!session.source_buffer().is_dirty());
}

#[test]
fn stale_canonical_save_does_not_replace_newer_source_or_history() {
    let mut session = replay_session(VIEW);
    let revision = session.source_revision();
    let canonical = session.canonical_source().unwrap();
    session
        .apply_command(UiAssetEditorCommand::edit_source(
            VIEW.replace("Before", "Newer"),
        ))
        .unwrap();
    let buffer = session.source_buffer.clone();
    let history = session.undo_stack.clone();
    assert!(!session
        .mark_canonical_source_persisted(revision, canonical)
        .unwrap());
    assert_eq!(session.source_buffer, buffer);
    assert_eq!(session.undo_stack, history);
    assert!(session.source_buffer().is_dirty());
}

#[test]
fn invalid_checkpoint_edge_leaves_buffer_history_and_dirty_unchanged() {
    let mut session = replay_session(VIEW);
    session
        .apply_command(UiAssetEditorCommand::edit_source(authored_source(
            VIEW,
            "保存🙂",
        )))
        .unwrap();
    let canonical = session.canonical_source().unwrap();
    session.source_buffer.replace("界🙂");
    let buffer = session.source_buffer.clone();
    let history = session.undo_stack.clone();
    let valid_source = session.last_valid_source_text.clone();
    let generation = session.last_valid_source_generation;
    assert!(session
        .mark_canonical_source_persisted(session.source_revision(), canonical)
        .is_err());
    assert_eq!(session.source_buffer, buffer);
    assert_eq!(session.undo_stack, history);
    assert_eq!(session.last_valid_source_text, valid_source);
    assert_eq!(session.last_valid_source_generation, generation);
    assert!(session.source_buffer().is_dirty());
}
