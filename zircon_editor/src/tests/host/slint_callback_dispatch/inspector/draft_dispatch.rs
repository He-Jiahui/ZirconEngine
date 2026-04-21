use super::super::support::*;

#[test]
fn inspector_draft_field_dispatch_updates_live_snapshot_without_scene_side_effects() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_inspector_draft");

    let effects =
        dispatch_inspector_draft_field(&harness.runtime, "entity://selected", "name", "Draft Cube")
            .unwrap();

    assert_eq!(
        harness
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Draft Cube")
    );
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.sync_asset_workspace);
}
