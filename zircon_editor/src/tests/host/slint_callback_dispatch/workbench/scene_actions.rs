use super::super::support::*;

#[test]
fn scene_menu_actions_dispatch_placeholder_status_through_callback_runtime_path() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_scene_menu_actions");
    let open_effects = dispatch_menu_action(&harness.runtime, "OpenScene").unwrap();
    let create_effects = dispatch_menu_action(&harness.runtime, "CreateScene").unwrap();

    let journal = harness.runtime.journal();
    let events: Vec<_> = journal
        .records()
        .iter()
        .rev()
        .take(2)
        .map(|record| record.event.clone())
        .collect();
    assert_eq!(
        events.into_iter().rev().collect::<Vec<_>>(),
        vec![
            EditorEvent::WorkbenchMenu(MenuAction::OpenScene),
            EditorEvent::WorkbenchMenu(MenuAction::CreateScene),
        ]
    );
    assert!(open_effects.presentation_dirty);
    assert!(!open_effects.layout_dirty);
    assert!(!open_effects.render_dirty);
    assert!(create_effects.presentation_dirty);
    assert!(!create_effects.layout_dirty);
    assert!(!create_effects.render_dirty);
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Scene open/create workflow is not wired yet"
    );
}
