use crate::core::editor_event::{EditorEvent, EditorEventEffect, LayoutCommand, MenuAction};
use crate::ui::slint_host::event_bridge::{apply_record_effects, SlintDispatchEffects};

use super::support::record_with_event_and_effects;

#[test]
fn layout_preset_events_project_active_preset_name_into_slint_effects() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_event_and_effects(
            EditorEvent::Layout(LayoutCommand::SavePreset {
                name: "rider".to_string(),
            }),
            vec![
                EditorEventEffect::LayoutChanged,
                EditorEventEffect::PresentationChanged,
            ],
        ),
    );

    assert_eq!(effects.active_layout_preset_name.as_deref(), Some("rider"));
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
}

#[test]
fn reset_layout_event_clears_active_preset_selection_in_host_effects() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_event_and_effects(
            EditorEvent::Layout(LayoutCommand::ResetToDefault),
            vec![
                EditorEventEffect::LayoutChanged,
                EditorEventEffect::PresentationChanged,
            ],
        ),
    );

    assert!(effects.reset_active_layout_preset);
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(effects.active_layout_preset_name, None);
}

#[test]
fn menu_reset_layout_event_also_clears_active_preset_selection_in_host_effects() {
    let mut effects = SlintDispatchEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_event_and_effects(
            EditorEvent::WorkbenchMenu(MenuAction::ResetLayout),
            vec![
                EditorEventEffect::LayoutChanged,
                EditorEventEffect::PresentationChanged,
            ],
        ),
    );

    assert!(effects.reset_active_layout_preset);
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(effects.active_layout_preset_name, None);
}
