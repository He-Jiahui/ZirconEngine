use crate::core::editor_event::EditorEventEffect;
use crate::ui::slint_host::event_bridge::{apply_record_effects, UiHostEventEffects};

use super::support::record_with_effects;

#[test]
fn present_welcome_effect_only_marks_welcome_presentation_path() {
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_effects(vec![
            EditorEventEffect::PresentWelcomeRequested,
            EditorEventEffect::PresentationChanged,
        ]),
    );

    assert!(effects.present_welcome_surface);
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.sync_asset_workspace);
    assert!(!effects.reset_active_layout_preset);
}
