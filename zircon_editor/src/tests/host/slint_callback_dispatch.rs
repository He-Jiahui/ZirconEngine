use crate::host::slint_host::callback_dispatch::{
    dispatch_asset_search, dispatch_hierarchy_selection, dispatch_menu_action,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};

#[test]
fn menu_action_dispatches_through_runtime_and_sets_scene_dirty_effects() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_menu");
    let before = harness.runtime.editor_snapshot().scene_entries.len();

    let effects = dispatch_menu_action(&harness.runtime, "CreateNode.Cube").unwrap();

    assert_eq!(harness.runtime.editor_snapshot().scene_entries.len(), before + 1);
    assert!(effects.presentation_dirty);
    assert!(effects.render_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.sync_asset_workspace);
}

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

#[test]
fn asset_search_dispatches_through_runtime_and_requests_asset_sync_effects() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_asset_search");

    let effects = dispatch_asset_search(&harness.runtime, "cube").unwrap();
    let snapshot = harness.runtime.editor_snapshot();

    assert_eq!(snapshot.asset_activity.search_query, "cube");
    assert_eq!(snapshot.asset_browser.search_query, "cube");
    assert!(effects.presentation_dirty);
    assert!(effects.sync_asset_workspace);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
}
