use super::super::support::*;

#[test]
fn save_preset_menu_action_dispatch_updates_active_preset_name_and_status_line() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_save_preset");
    let effects = dispatch_menu_action(&harness.runtime, "SavePreset.rider").unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(crate::core::editor_event::LayoutCommand::SavePreset {
            name: "rider".to_string(),
        })
    );
    assert_eq!(effects.active_layout_preset_name.as_deref(), Some("rider"));
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Saved layout preset asset rider"
    );
}

#[test]
fn load_preset_menu_action_without_suffix_falls_back_to_current_name() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_load_preset");
    dispatch_menu_action(&harness.runtime, "SavePreset.current").unwrap();
    let effects = dispatch_menu_action(&harness.runtime, "LoadPreset.").unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Layout(crate::core::editor_event::LayoutCommand::LoadPreset {
            name: "current".to_string(),
        })
    );
    assert_eq!(
        effects.active_layout_preset_name.as_deref(),
        Some("current")
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
    assert_eq!(
        harness.runtime.editor_snapshot().status_line,
        "Loaded layout preset current"
    );
}
