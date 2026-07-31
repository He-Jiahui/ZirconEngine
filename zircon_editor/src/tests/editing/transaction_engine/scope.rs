use std::sync::atomic::Ordering;

use crate::core::editing::engine::{
    EditCommandError, EditorTransactionEngine, HistoryContextId, MergeMode,
};
use crate::core::editor_message::DocumentId;

use super::fixture::{finalized_counter, DeltaCommand, FixtureContext};

#[test]
fn cancel_and_drop_revert_applied_commands_in_reverse_order() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    {
        let mut scope = engine.begin("drop", HistoryContextId::Global).unwrap();
        scope
            .push(DeltaCommand::new("first", 1, 2, finalized.clone()))
            .unwrap();
        scope
            .push(DeltaCommand::new("second", 2, 3, finalized.clone()))
            .unwrap();
    }
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|ctx| ctx.value)
            .unwrap(),
        Some(0)
    );
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|ctx| ctx.trace.clone())
            .unwrap(),
        Some(vec!["first", "second", "revert", "revert"])
    );

    let mut explicit = engine.begin("cancel", HistoryContextId::Global).unwrap();
    explicit
        .push(DeltaCommand::new("third", 3, 7, finalized))
        .unwrap();
    explicit.cancel().unwrap();
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|ctx| ctx.value)
            .unwrap(),
        Some(0)
    );
}

#[test]
fn same_context_nesting_folds_and_cross_context_nesting_is_typed_error() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let document = DocumentId::new(9);
    let context = HistoryContextId::Document(document);
    let mut outer = engine.begin("outer", context).unwrap();
    outer
        .push(DeltaCommand::new("outer", 1, 1, finalized.clone()))
        .unwrap();
    {
        let mut nested = engine.begin("nested", context).unwrap();
        nested
            .push(DeltaCommand::new("nested", 2, 2, finalized.clone()))
            .unwrap();
        nested.commit().unwrap();
    }

    assert!(matches!(
        engine.begin("wrong", HistoryContextId::Global),
        Err(EditCommandError::CrossContextNested { active, requested })
            if active == context && requested == HistoryContextId::Global
    ));
    outer.commit().unwrap();

    let status = engine.history_status(context).unwrap();
    assert_eq!(status.len, 1);
    assert_eq!(
        engine.history_details(context, None, 1).unwrap().records()[0].command_count,
        2
    );
    engine.undo(context).unwrap();
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|ctx| ctx.value)
            .unwrap(),
        Some(0)
    );
}

#[test]
fn outer_scope_misuse_while_nested_does_not_abandon_the_outer_scope() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut outer = engine.begin("outer", HistoryContextId::Global).unwrap();
    let nested = engine.begin("nested", HistoryContextId::Global).unwrap();

    assert!(matches!(
        outer.push(DeltaCommand::new("too early", 1, 1, finalized.clone())),
        Err(EditCommandError::ScopeClosed)
    ));
    nested.cancel().unwrap();

    outer
        .push(DeltaCommand::new("after nested", 2, 3, finalized))
        .unwrap();
    outer.cancel().unwrap();
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|context| context.value)
            .unwrap(),
        Some(0)
    );

    let replacement = engine
        .begin("replacement", HistoryContextId::Global)
        .unwrap();
    replacement.cancel().unwrap();
}

#[test]
fn out_of_order_scope_consumption_cancels_descendants_without_residue() {
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let outer = engine.begin("outer", HistoryContextId::Global).unwrap();
    let middle = engine.begin("middle", HistoryContextId::Global).unwrap();
    let inner = engine.begin("inner", HistoryContextId::Global).unwrap();

    drop(middle);
    drop(inner);
    outer.cancel().unwrap();
    let replacement = engine
        .begin("replacement", HistoryContextId::Global)
        .unwrap();
    replacement.cancel().unwrap();

    let outer = engine
        .begin("outer commit", HistoryContextId::Global)
        .unwrap();
    let nested = engine.begin("nested", HistoryContextId::Global).unwrap();
    assert!(matches!(outer.commit(), Err(EditCommandError::ScopeClosed)));
    drop(nested);
    let final_scope = engine.begin("final", HistoryContextId::Global).unwrap();
    final_scope.cancel().unwrap();
}

#[test]
fn nested_commit_uses_parent_merge_mode_and_finalizes_absorbed_commands() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());

    let mut ends = engine.begin("ends", HistoryContextId::Global).unwrap();
    ends.set_merge_mode(MergeMode::Ends);
    ends.push(DeltaCommand::new("parent", 1, 1, finalized.clone()))
        .unwrap();
    let mut nested = engine.begin("nested", HistoryContextId::Global).unwrap();
    nested
        .push(DeltaCommand::new("child", 1, 2, finalized.clone()))
        .unwrap();
    nested.commit().unwrap();
    ends.commit().unwrap();
    assert_eq!(
        engine
            .history_details(HistoryContextId::Global, None, 1)
            .unwrap()
            .records()[0]
            .command_count,
        1
    );

    let document = HistoryContextId::Document(DocumentId::new(44));
    let mut all = engine.begin("all", document).unwrap();
    all.set_merge_mode(MergeMode::All);
    all.push(DeltaCommand::new("first", 1, 1, finalized.clone()))
        .unwrap();
    all.push(DeltaCommand::new("second", 2, 1, finalized.clone()))
        .unwrap();
    let mut nested = engine.begin("nested", document).unwrap();
    nested
        .push(DeltaCommand::new("child", 1, 2, finalized.clone()))
        .unwrap();
    nested.commit().unwrap();
    all.commit().unwrap();
    assert_eq!(
        engine.history_details(document, None, 1).unwrap().records()[0].command_count,
        2
    );
    assert_eq!(finalized.load(Ordering::SeqCst), 2);
}

#[test]
fn merge_modes_disable_ends_and_all_have_distinct_search_scope() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());

    let mut disabled = engine.begin("disabled", HistoryContextId::Global).unwrap();
    disabled.set_merge_mode(MergeMode::Disable);
    for _ in 0..2 {
        disabled
            .push(DeltaCommand::new("a", 1, 1, finalized.clone()))
            .unwrap();
    }
    disabled.commit().unwrap();
    assert_eq!(
        engine
            .history_details(HistoryContextId::Global, None, 1)
            .unwrap()
            .records()[0]
            .command_count,
        2
    );

    let document = HistoryContextId::Document(DocumentId::new(2));
    let mut ends = engine.begin("ends", document).unwrap();
    ends.set_merge_mode(MergeMode::Ends);
    for _ in 0..2 {
        ends.push(DeltaCommand::new("a", 1, 1, finalized.clone()))
            .unwrap();
    }
    ends.commit().unwrap();
    assert_eq!(
        engine.history_details(document, None, 1).unwrap().records()[0].command_count,
        1
    );

    let all_context = HistoryContextId::Document(DocumentId::new(3));
    let mut all = engine.begin("all", all_context).unwrap();
    all.set_merge_mode(MergeMode::All);
    for key in [1, 2, 1] {
        all.push(DeltaCommand::new("mixed", key, 1, finalized.clone()))
            .unwrap();
    }
    all.commit().unwrap();
    assert_eq!(
        engine
            .history_details(all_context, None, 1)
            .unwrap()
            .records()[0]
            .command_count,
        2
    );
    assert_eq!(finalized.load(Ordering::SeqCst), 2);
}

#[test]
fn failed_push_cancels_prior_applied_commands() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext::default());
    let mut scope = engine.begin("failure", HistoryContextId::Global).unwrap();
    scope
        .push(DeltaCommand::new("good", 1, 4, finalized.clone()))
        .unwrap();
    assert!(matches!(
        scope.push(DeltaCommand::new("bad", 2, 8, finalized).failing()),
        Err(EditCommandError::TargetMissing { .. })
    ));
    assert_eq!(
        engine
            .with_context::<FixtureContext, _>(|ctx| ctx.value)
            .unwrap(),
        Some(0)
    );
    assert_eq!(
        engine.history_status(HistoryContextId::Global).unwrap().len,
        0
    );
}

#[test]
fn record_keeps_participants_selection_significance_and_frame() {
    let finalized = finalized_counter();
    let engine = EditorTransactionEngine::new(FixtureContext {
        selection: 5,
        ..FixtureContext::default()
    });
    engine.set_frame(41).unwrap();
    let document = DocumentId::new(7);
    let context = HistoryContextId::Document(document);
    let mut scope = engine.begin("metadata", context).unwrap();
    scope.add_participant(DocumentId::new(8));
    scope
        .push(
            DeltaCommand::new("selection", 1, 0, finalized)
                .selecting(9)
                .insignificant(),
        )
        .unwrap();
    scope.commit().unwrap();

    let page = engine.history_details(context, None, 1).unwrap();
    let record = &page.records()[0];
    assert_eq!(record.timestamp_frame, 41);
    assert_eq!(record.participants.len(), 2);
    assert_eq!(record.selection_before.fixture_value_ref().unwrap(), 5);
    assert_eq!(record.selection_after.fixture_value_ref().unwrap(), 9);
    assert!(!record.significant);
}
