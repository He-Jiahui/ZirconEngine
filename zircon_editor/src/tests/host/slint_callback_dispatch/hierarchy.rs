use super::support::*;

#[test]
fn hierarchy_selection_dispatches_through_runtime_and_updates_selected_node() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_hierarchy");
    let target = harness
        .runtime
        .editor_snapshot()
        .scene_entries
        .iter()
        .find(|entry| !entry.selected)
        .map(|entry| entry.id)
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
            .find(|entry| entry.id == target)
            .map(|entry| entry.selected),
        Some(true)
    );
}
