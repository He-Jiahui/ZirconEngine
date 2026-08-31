use crate::core::asset::AssetWriteAccess;
use crate::core::commands::{CommandEvalCtx, DocumentKind, PlayModePredicate, WhenClause};
use crate::core::editor_message::{PlayStateKind, SceneModeId};

#[test]
fn when_clause_evaluates_boolean_composition_deterministically() {
    let context = CommandEvalCtx::interactive()
        .with_project_open(true)
        .with_undo_available(true)
        .with_selection_count(2)
        .with_capabilities(["editor.scene.authoring"]);

    assert!(WhenClause::All(vec![
        WhenClause::ProjectOpen,
        WhenClause::SelectionNonEmpty,
        WhenClause::Capability("editor.scene.authoring".to_string()),
    ])
    .eval(&context));
    assert!(WhenClause::UndoAvailable.eval(&context));
    assert!(!WhenClause::RedoAvailable.eval(&context));
    assert!(WhenClause::Any(vec![
        WhenClause::UndoAvailable,
        WhenClause::SelectionNonEmpty,
    ])
    .eval(&context));
    assert!(WhenClause::Not(Box::new(WhenClause::RedoAvailable)).eval(&context));
}

#[test]
fn focused_document_scene_mode_and_selection_use_typed_snapshot_values() {
    let scene_document = DocumentKind::parse("scene").unwrap();
    assert!(DocumentKind::parse("Scene Document").is_err());
    let select_mode = SceneModeId::new("select");
    let context = CommandEvalCtx::interactive()
        .with_focused_document_kind(scene_document.clone())
        .with_scene_mode(select_mode.clone())
        .with_selection_count(1);

    assert!(WhenClause::FocusedDocumentKind(scene_document).eval(&context));
    assert!(WhenClause::SceneModeActive(select_mode).eval(&context));
    assert!(WhenClause::SelectionNonEmpty.eval(&context));
    assert!(!WhenClause::SceneModeActive(SceneModeId::new("paint")).eval(&context));
    assert!(
        !WhenClause::SelectionNonEmpty.eval(&CommandEvalCtx::interactive().with_selection_count(0))
    );
}

#[test]
fn play_mode_predicates_distinguish_edit_building_playing_and_cleanup() {
    for (state, expected) in [
        (PlayStateKind::Edit, PlayModePredicate::Edit),
        (PlayStateKind::Building, PlayModePredicate::Building),
        (PlayStateKind::Playing, PlayModePredicate::Playing),
        (
            PlayStateKind::CleanupFailed,
            PlayModePredicate::CleanupFailed,
        ),
    ] {
        let context = CommandEvalCtx::interactive().with_play_state(state);
        for predicate in [
            PlayModePredicate::Edit,
            PlayModePredicate::Building,
            PlayModePredicate::Playing,
            PlayModePredicate::CleanupFailed,
        ] {
            assert_eq!(
                WhenClause::PlayMode(predicate).eval(&context),
                predicate == expected
            );
        }
    }
}

#[test]
fn headless_context_only_satisfies_always_and_capability_predicates() {
    let context = CommandEvalCtx::headless(["editor.remote.safe"]);

    assert!(WhenClause::Always.eval(&context));
    assert!(WhenClause::Capability("editor.remote.safe".to_string()).eval(&context));
    assert!(!WhenClause::Capability("editor.remote.missing".to_string()).eval(&context));
    for contextual in [
        WhenClause::ProjectOpen,
        WhenClause::UndoAvailable,
        WhenClause::RedoAvailable,
        WhenClause::FocusedDocumentKind(DocumentKind::parse("scene").unwrap()),
        WhenClause::SceneModeActive(SceneModeId::new("select")),
        WhenClause::SelectionNonEmpty,
        WhenClause::PlayMode(PlayModePredicate::Edit),
    ] {
        assert!(!contextual.eval(&context));
        assert!(!WhenClause::Not(Box::new(contextual)).eval(&context));
    }
    assert!(!WhenClause::Not(Box::new(WhenClause::All(vec![
        WhenClause::ProjectOpen,
        WhenClause::Capability("editor.remote.missing".to_string()),
    ])))
    .eval(&context));
}

#[test]
fn interactive_overlap_is_exact_for_exclusive_and_composed_when_domains() {
    let scene = WhenClause::FocusedDocumentKind(DocumentKind::scene());
    let material = WhenClause::FocusedDocumentKind(DocumentKind::material());
    assert!(!scene.can_overlap_in_interactive_context(&material));

    let select = WhenClause::SceneModeActive(SceneModeId::new("select"));
    let paint = WhenClause::SceneModeActive(SceneModeId::new("paint"));
    assert!(!select.can_overlap_in_interactive_context(&paint));

    let selected = WhenClause::SelectionNonEmpty;
    let unselected = WhenClause::Not(Box::new(WhenClause::SelectionNonEmpty));
    assert!(!selected.can_overlap_in_interactive_context(&unselected));

    let writable = WhenClause::AssetWritable;
    let read_only = WhenClause::Not(Box::new(WhenClause::AssetWritable));
    assert!(!writable.can_overlap_in_interactive_context(&read_only));
    assert!(writable
        .eval(&CommandEvalCtx::interactive().with_asset_write_access(AssetWriteAccess::Writable)));

    let edit = WhenClause::PlayMode(PlayModePredicate::Edit);
    let playing = WhenClause::PlayMode(PlayModePredicate::Playing);
    assert!(!edit.can_overlap_in_interactive_context(&playing));

    let project_without_selection = WhenClause::All(vec![
        WhenClause::ProjectOpen,
        WhenClause::Not(Box::new(WhenClause::SelectionNonEmpty)),
    ]);
    assert!(!project_without_selection.can_overlap_in_interactive_context(&selected));
    assert!(project_without_selection.can_overlap_in_interactive_context(&WhenClause::ProjectOpen));

    let authoring = WhenClause::Capability("editor.scene.authoring".to_string());
    let diagnostics = WhenClause::Capability("editor.diagnostics".to_string());
    assert!(authoring.can_overlap_in_interactive_context(&diagnostics));
}
