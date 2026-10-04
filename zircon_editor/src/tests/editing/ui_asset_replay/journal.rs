use crate::ui::asset_editor::{
    UiAssetEditorCommandJournal, UiAssetEditorCommandJournalEntry,
    UiAssetEditorCommandJournalReplayError, UiAssetEditorExternalEffect,
    UiAssetEditorJournalCommand, UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession,
    UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind, UiAssetEditorUndoExternalEffects,
    UI_ASSET_EDITOR_BUG_REPORT_REPLAY_ARTIFACT_SCHEMA_VERSION,
    UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION,
};
use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

use super::fixtures::{EXISTING_EXTERNAL_STYLE_ASSET_TOML, LOCAL_THEME_LAYOUT_ASSET_TOML};

#[test]
fn ui_asset_editor_session_exports_sanitized_bug_report_replay_artifact() {
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
    .expect("replay artifact session");

    let promoted_style = session
        .promote_local_theme_to_external_style_asset(
            "res://ui/themes/replay_theme.zui",
            "ui.theme.replay_theme",
            "Replay Theme",
        )
        .expect("promote local theme")
        .expect("promoted style document");
    let promoted_style_source =
        crate::ui::asset_editor::serialize_authoring_document_as_v2(&promoted_style)
            .expect("serialize promoted style document as v2");

    let artifact = session.export_bug_report_replay_artifact();
    assert_eq!(
        artifact.schema_version,
        UI_ASSET_EDITOR_BUG_REPORT_REPLAY_ARTIFACT_SCHEMA_VERSION
    );
    assert_eq!(artifact.route.asset_id, "res://ui/tests/replay_theme.zui");
    assert!(artifact.initial_source.redacted);
    assert!(artifact.current_source.redacted);
    assert!(artifact.initial_source.byte_len > 0);
    assert!(artifact.current_source.byte_len > 0);
    assert_ne!(
        artifact.initial_source.stable_hash,
        artifact.current_source.stable_hash
    );
    assert_eq!(artifact.records.len(), 1);

    let record = &artifact.records[0];
    assert_eq!(record.label, "Promote Local Theme");
    assert!(record.redo_external_effects.iter().any(|effect| {
        effect.effect_id == "upsert_asset_source"
            && effect.asset_id == "res://ui/themes/replay_theme.zui"
            && effect
                .source
                .as_ref()
                .is_some_and(|source| source.byte_len > 0)
    }));
    assert!(record
        .undo_external_effects
        .iter()
        .any(|effect| effect.effect_id == "remove_asset_source"
            && effect.asset_id == "res://ui/themes/replay_theme.zui"
            && effect.source.is_none()));

    let serialized = serde_json::to_string(&artifact).expect("serialize replay artifact");
    assert!(
        !serialized.contains("[asset]"),
        "artifact leaked raw TOML source: {serialized}"
    );
    assert!(
        !serialized.contains("display_name ="),
        "artifact leaked raw source field syntax: {serialized}"
    );
    assert!(
        !serialized.contains(&promoted_style_source),
        "artifact leaked generated external source"
    );
}

#[test]
fn ui_asset_editor_command_journal_replays_source_edits_through_session_commands() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_journal.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("journal replay session");
    let next_source = LOCAL_THEME_LAYOUT_ASSET_TOML.replace(
        "display_name = \"Replay Theme\"",
        "display_name = \"Journal Replay Theme\"",
    );
    let journal =
        UiAssetEditorCommandJournal::new(vec![UiAssetEditorCommandJournalEntry::source_edit(
            1,
            "Journal Source Edit",
            next_source.clone(),
        )]);

    let report = session
        .apply_command_journal(&journal)
        .expect("apply command journal");

    assert_eq!(report.applied_entries, 1);
    assert_eq!(report.labels, vec!["Journal Source Edit".to_string()]);
    assert_eq!(session.source_buffer().text(), next_source);
    assert_eq!(
        session.next_undo_label(),
        Some("Journal Source Edit".to_string())
    );
}

#[test]
fn ui_asset_editor_command_journal_serializes_tree_edits_and_external_effects() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_journal_tree.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("journal tree replay session");
    let next_source = LOCAL_THEME_LAYOUT_ASSET_TOML.replace(
        "display_name = \"Replay Theme\"",
        "display_name = \"Journal Tree Replay Theme\"",
    );
    let journal =
        UiAssetEditorCommandJournal::new(vec![UiAssetEditorCommandJournalEntry::tree_edit(
            1,
            UiAssetEditorTreeEdit::generic(UiAssetEditorTreeEditKind::DocumentEdit),
            "Journal Tree Edit",
            next_source.clone(),
        )
        .with_external_effects(UiAssetEditorUndoExternalEffects {
            undo: vec![UiAssetEditorExternalEffect::RemoveAssetSource {
                asset_id: "res://ui/themes/journal_theme.zui".to_string(),
            }],
            redo: vec![UiAssetEditorExternalEffect::UpsertAssetSource {
                asset_id: "res://ui/themes/journal_theme.zui".to_string(),
                source: EXISTING_EXTERNAL_STYLE_ASSET_TOML.to_string(),
            }],
        })]);
    let serialized = serde_json::to_string(&journal).expect("serialize command journal");
    let deserialized: UiAssetEditorCommandJournal =
        serde_json::from_str(&serialized).expect("deserialize command journal");

    let report = session
        .apply_command_journal(&deserialized)
        .expect("apply deserialized command journal");

    assert_eq!(report.labels, vec!["Journal Tree Edit".to_string()]);
    assert_eq!(session.source_buffer().text(), next_source);
    assert_eq!(
        session.next_undo_external_effects(),
        vec![UiAssetEditorExternalEffect::RemoveAssetSource {
            asset_id: "res://ui/themes/journal_theme.zui".to_string(),
        }]
    );
    assert_eq!(
        session.next_redo_external_effects(),
        vec![UiAssetEditorExternalEffect::UpsertAssetSource {
            asset_id: "res://ui/themes/journal_theme.zui".to_string(),
            source: EXISTING_EXTERNAL_STYLE_ASSET_TOML.to_string(),
        }]
    );
    assert!(matches!(
        &deserialized.entries[0].command,
        UiAssetEditorJournalCommand::TreeEdit { .. }
    ));
}

#[test]
fn ui_asset_editor_command_journal_rejects_unsupported_schema_and_sequence_regressions() {
    let route = UiAssetEditorRoute::new(
        "res://ui/tests/replay_journal_errors.zui",
        UiAssetKind::Layout,
        UiAssetEditorMode::Design,
    );
    let mut session = UiAssetEditorSession::from_source(
        route,
        LOCAL_THEME_LAYOUT_ASSET_TOML,
        UiSize::new(960.0, 540.0),
    )
    .expect("journal error session");
    let mut unsupported = UiAssetEditorCommandJournal::new(Vec::new());
    unsupported.schema_version = UI_ASSET_EDITOR_COMMAND_JOURNAL_SCHEMA_VERSION + 1;

    let error = session
        .apply_command_journal(&unsupported)
        .expect_err("unsupported schema rejected");
    assert!(matches!(
        error,
        UiAssetEditorCommandJournalReplayError::UnsupportedSchemaVersion { .. }
    ));

    let journal = UiAssetEditorCommandJournal::new(vec![
        UiAssetEditorCommandJournalEntry::source_edit(
            2,
            "First Journal Source Edit",
            LOCAL_THEME_LAYOUT_ASSET_TOML,
        ),
        UiAssetEditorCommandJournalEntry::source_edit(
            2,
            "Duplicate Journal Source Edit",
            LOCAL_THEME_LAYOUT_ASSET_TOML,
        ),
    ]);
    let error = session
        .apply_command_journal(&journal)
        .expect_err("non-monotonic sequence rejected");
    assert!(matches!(
        error,
        UiAssetEditorCommandJournalReplayError::NonMonotonicSequence {
            previous: 2,
            current: 2
        }
    ));
    assert_eq!(
        session.source_buffer().text(),
        LOCAL_THEME_LAYOUT_ASSET_TOML
    );
    assert!(!session.can_undo());
}
