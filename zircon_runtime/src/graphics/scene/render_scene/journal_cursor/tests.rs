use super::{
    RenderSceneJournalCommit, RenderSceneJournalCursor, RenderSceneJournalCursorError,
    RenderSceneJournalPreflight, RenderSceneJournalPreflightKind,
};
use crate::core::framework::render::RenderWorldSnapshotHandle;
use crate::graphics::scene::render_scene::{
    RenderSceneApplyStats, RenderSceneChangeJournal, RenderSceneGeneration,
};

#[test]
fn render_scene_journal_cursor_advances_only_after_explicit_commit() {
    let mut cursor = test_cursor(0);
    let journal = test_journal(0, 1);

    let preflight = cursor.preflight(&journal).expect("contiguous journal");

    assert!(preflight.requires_apply());
    assert_eq!(cursor.world(), test_world(1));
    assert_eq!(
        preflight.journal_from_generation(),
        RenderSceneGeneration::INITIAL
    );
    assert_eq!(
        preflight.journal_to_generation(),
        RenderSceneGeneration::new(1)
    );
    assert_eq!(cursor.applied_generation(), RenderSceneGeneration::INITIAL);
    assert_eq!(
        cursor.commit(preflight).expect("commit staged journal"),
        RenderSceneJournalCommit::Applied
    );
    assert_eq!(cursor.applied_generation(), RenderSceneGeneration::new(1));
}

#[test]
fn render_scene_journal_cursor_treats_exact_generation_replay_as_no_op() {
    let mut cursor = RenderSceneJournalCursor::at(test_world(1), RenderSceneGeneration::new(2));
    let replay = test_journal(1, 2);

    let preflight = cursor.preflight(&replay).expect("exact replay");

    assert!(!preflight.requires_apply());
    assert_eq!(
        cursor.commit(preflight).expect("commit replay"),
        RenderSceneJournalCommit::Replayed
    );
    assert_eq!(cursor.applied_generation(), RenderSceneGeneration::new(2));
}

#[test]
fn render_scene_journal_cursor_rejects_skipped_and_stale_generations() {
    let cursor = RenderSceneJournalCursor::at(test_world(1), RenderSceneGeneration::new(2));

    assert_eq!(
        cursor
            .preflight(&test_journal(3, 4))
            .expect_err("generation three was skipped"),
        RenderSceneJournalCursorError::GenerationGap {
            applied_generation: RenderSceneGeneration::new(2),
            journal_from_generation: RenderSceneGeneration::new(3),
            journal_to_generation: RenderSceneGeneration::new(4),
        }
    );
    assert_eq!(
        cursor
            .preflight(&test_journal(0, 1))
            .expect_err("journal is older than applied state"),
        RenderSceneJournalCursorError::StaleJournal {
            applied_generation: RenderSceneGeneration::new(2),
            journal_to_generation: RenderSceneGeneration::new(1),
        }
    );
}

#[test]
fn render_scene_journal_cursor_rejects_forward_journal_that_skips_a_generation() {
    let cursor = test_cursor(2);

    assert_eq!(
        cursor
            .preflight(&test_journal(2, 4))
            .expect_err("one sealed journal cannot span multiple generations"),
        RenderSceneJournalCursorError::NonAdjacentJournalRange {
            journal_from_generation: RenderSceneGeneration::new(2),
            journal_to_generation: RenderSceneGeneration::new(4),
        }
    );
}

#[test]
fn render_scene_journal_cursor_rejects_wide_overlap_as_non_exact_replay() {
    let cursor = RenderSceneJournalCursor::at(test_world(1), RenderSceneGeneration::new(4));

    assert_eq!(
        cursor
            .preflight(&test_journal(1, 4))
            .expect_err("a wide overlap is not an exact journal replay"),
        RenderSceneJournalCursorError::NonAdjacentJournalRange {
            journal_from_generation: RenderSceneGeneration::new(1),
            journal_to_generation: RenderSceneGeneration::new(4),
        }
    );
}

#[test]
fn render_scene_journal_cursor_rejects_commit_token_after_another_commit_advances_it() {
    let mut cursor = test_cursor(0);
    let journal = test_journal(0, 1);
    let first = cursor.preflight(&journal).expect("first token");
    let stale = cursor
        .preflight(&journal)
        .expect("parallel preflight token");
    cursor.commit(first).expect("first commit");

    assert_eq!(
        cursor
            .commit(stale)
            .expect_err("stale token must not advance twice"),
        RenderSceneJournalCursorError::CursorAdvanced {
            expected_generation: RenderSceneGeneration::INITIAL,
            applied_generation: RenderSceneGeneration::new(1),
        }
    );
}

#[test]
fn render_scene_journal_cursor_commit_rejects_forged_non_adjacent_token() {
    let mut cursor = test_cursor(2);
    let forged = RenderSceneJournalPreflight {
        kind: RenderSceneJournalPreflightKind::Apply,
        world: test_world(1),
        journal_from_generation: RenderSceneGeneration::new(2),
        journal_to_generation: RenderSceneGeneration::new(4),
    };

    assert_eq!(
        cursor
            .commit(forged)
            .expect_err("commit must independently reject a generation jump"),
        RenderSceneJournalCursorError::NonAdjacentJournalRange {
            journal_from_generation: RenderSceneGeneration::new(2),
            journal_to_generation: RenderSceneGeneration::new(4),
        }
    );
    assert_eq!(cursor.applied_generation(), RenderSceneGeneration::new(2));
}

#[test]
fn render_scene_journal_cursor_rejects_inverted_journal_range() {
    let cursor = test_cursor(0);

    assert_eq!(
        cursor
            .preflight(&test_journal(2, 1))
            .expect_err("journal range cannot run backwards"),
        RenderSceneJournalCursorError::InvalidJournalRange {
            journal_from_generation: RenderSceneGeneration::new(2),
            journal_to_generation: RenderSceneGeneration::new(1),
        }
    );
}

#[test]
fn render_scene_journal_cursor_rejects_matching_generation_from_another_world() {
    let cursor = RenderSceneJournalCursor::at(test_world(1), RenderSceneGeneration::new(2));
    let foreign = test_journal_for_world(test_world(2), 1, 2);

    assert_eq!(
        cursor
            .preflight(&foreign)
            .expect_err("another world cannot replay by generation alone"),
        RenderSceneJournalCursorError::WorldChanged {
            expected_world: test_world(1),
            journal_world: test_world(2),
        }
    );

    let source_cursor = test_cursor(0);
    let token = source_cursor
        .preflight(&test_journal(0, 1))
        .expect("source-world token");
    let mut other_cursor =
        RenderSceneJournalCursor::at(test_world(2), RenderSceneGeneration::INITIAL);
    assert_eq!(
        other_cursor
            .commit(token)
            .expect_err("preflight token is bound to its source world"),
        RenderSceneJournalCursorError::WorldChanged {
            expected_world: test_world(2),
            journal_world: test_world(1),
        }
    );
}

fn test_journal(from: u64, to: u64) -> RenderSceneChangeJournal {
    test_journal_for_world(test_world(1), from, to)
}

fn test_journal_for_world(
    world: RenderWorldSnapshotHandle,
    from: u64,
    to: u64,
) -> RenderSceneChangeJournal {
    RenderSceneChangeJournal::new(
        world,
        RenderSceneGeneration::new(from),
        RenderSceneGeneration::new(to),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        RenderSceneApplyStats::default(),
    )
}

fn test_cursor(generation: u64) -> RenderSceneJournalCursor {
    RenderSceneJournalCursor::at(test_world(1), RenderSceneGeneration::new(generation))
}

const fn test_world(raw: u64) -> RenderWorldSnapshotHandle {
    RenderWorldSnapshotHandle::new(raw)
}
