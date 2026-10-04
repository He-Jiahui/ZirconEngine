use super::*;

fn context(
    window_generation: u64,
    focus_generation: u64,
    composition_generation: u64,
) -> ZrRuntimeImeCompositionContextV2 {
    ZrRuntimeImeCompositionContextV2 {
        window_generation,
        focus_generation,
        composition_generation,
    }
}

#[test]
fn cancel_tombstone_rejects_late_text_and_allows_retry_in_new_generation() {
    let mut gate = RuntimeImeCompositionGenerationGate::new(11);
    let first = context(11, 3, 7);
    assert!(gate.begin_or_update(first));
    assert!(gate.begin_or_update(first));

    // Focus loss advances the focus epoch. A delayed callback from the old native context
    // must not resurrect the preedit after the cancellation has been applied.
    let cancel = context(11, 4, 7);
    assert!(gate.cancel(cancel));
    assert!(!gate.begin_or_update(first));
    assert!(!gate.cancel(first));

    // Re-entering text input opens a new composition generation; updates can retry normally.
    let retry = context(11, 5, 8);
    assert!(gate.begin_or_update(retry));
    assert!(gate.commit(retry));
    assert!(!gate.begin_or_update(retry));
}

#[test]
fn window_generation_and_focus_generation_keep_independent_sessions_isolated() {
    let mut first_window = RuntimeImeCompositionGenerationGate::new(21);
    let mut second_window = RuntimeImeCompositionGenerationGate::new(22);
    let first = context(21, 1, 1);
    let second = context(22, 1, 1);

    assert!(first_window.begin_or_update(first));
    assert!(second_window.begin_or_update(second));
    assert!(!first_window.begin_or_update(second));
    assert!(!second_window.commit(first));
    assert!(first_window.cancel(context(21, 2, 1)));
    assert!(second_window.begin_or_update(second));
}

#[test]
fn duplicate_commit_and_cancel_are_idempotent_without_reopening_the_session() {
    let mut gate = RuntimeImeCompositionGenerationGate::new(31);
    let active = context(31, 9, 10);
    assert!(gate.begin_or_update(active));
    assert!(gate.commit(active));
    assert!(!gate.commit(active));
    assert!(!gate.begin_or_update(active));
    assert!(gate.cancel(context(31, 10, 10)));
    assert!(!gate.cancel(context(31, 10, 10)));
}
