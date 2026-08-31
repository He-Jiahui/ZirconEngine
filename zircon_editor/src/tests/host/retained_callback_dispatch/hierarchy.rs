use super::support::*;

#[test]
fn hierarchy_selection_dispatches_through_runtime_and_updates_selected_node() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_callback_hierarchy");
    let initial_snapshot = harness.runtime.editor_snapshot();
    let target = initial_snapshot
        .scene_entries
        .iter()
        .find(|entry| !initial_snapshot.scene_entries.is_selected(entry.entity))
        .map(|entry| entry.entity)
        .expect("default scene should contain an unselected node");

    let effects = dispatch_hierarchy_selection(&harness.runtime, target).unwrap();
    let snapshot = harness.runtime.editor_snapshot();

    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert_eq!(
        snapshot
            .scene_entries
            .iter()
            .find(|entry| entry.entity == target)
            .map(|entry| snapshot.scene_entries.is_selected(entry.entity)),
        Some(true)
    );
}

#[test]
fn repeated_hierarchy_selection_is_an_invalidation_noop() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_callback_hierarchy_noop");
    let initial_snapshot = harness.runtime.editor_snapshot();
    let target = initial_snapshot
        .scene_entries
        .iter()
        .find(|entry| !initial_snapshot.scene_entries.is_selected(entry.entity))
        .map(|entry| entry.entity)
        .expect("default scene should contain an unselected node");

    let first = dispatch_hierarchy_selection(&harness.runtime, target).unwrap();
    let repeated = dispatch_hierarchy_selection(&harness.runtime, target).unwrap();

    assert!(first.presentation_dirty);
    assert!(!repeated.presentation_dirty);
    assert!(!repeated.layout_dirty);
    assert!(!repeated.render_dirty);
}
