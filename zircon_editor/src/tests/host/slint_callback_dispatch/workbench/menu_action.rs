use super::super::support::*;

#[test]
fn menu_action_dispatches_through_runtime_and_sets_scene_dirty_effects() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_menu");
    let before = harness.runtime.editor_snapshot().scene_entries.len();

    let effects = dispatch_menu_action(&harness.runtime, "CreateNode.Cube").unwrap();

    assert_eq!(
        harness.runtime.editor_snapshot().scene_entries.len(),
        before + 1
    );
    assert!(effects.presentation_dirty);
    assert!(effects.render_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.sync_asset_workspace);
}
