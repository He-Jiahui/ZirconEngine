use crate::core::editor_event::EditorEventEffect;
use crate::ui::slint_host::event_bridge::{apply_record_effects, UiHostEventEffects};

use super::support::record_with_effects;

#[test]
fn record_effects_mark_render_layout_and_asset_refresh_flags_for_slint_host() {
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_effects(vec![
            EditorEventEffect::ProjectOpenRequested,
            EditorEventEffect::LayoutChanged,
            EditorEventEffect::RenderChanged,
            EditorEventEffect::PresentationChanged,
        ]),
    );

    assert!(effects.presentation_dirty);
    assert!(effects.layout_dirty);
    assert!(effects.render_dirty);
    assert!(effects.sync_asset_workspace);
    assert!(effects.reset_active_layout_preset);
}

#[test]
fn asset_preview_refresh_effect_is_local_to_presentation_without_backend_sync() {
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_effects(vec![
            EditorEventEffect::AssetPreviewRefreshRequested,
            EditorEventEffect::PresentationChanged,
        ]),
    );

    assert!(effects.presentation_dirty);
    assert!(!effects.sync_asset_workspace);
    assert!(!effects.refresh_asset_details);
    assert!(effects.refresh_visible_asset_previews);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
}

#[test]
fn asset_details_refresh_effect_does_not_require_backend_sync() {
    let mut effects = UiHostEventEffects::default();
    apply_record_effects(
        &mut effects,
        &record_with_effects(vec![
            EditorEventEffect::AssetDetailsRefreshRequested,
            EditorEventEffect::PresentationChanged,
        ]),
    );

    assert!(effects.presentation_dirty);
    assert!(!effects.sync_asset_workspace);
    assert!(effects.refresh_asset_details);
    assert!(!effects.refresh_visible_asset_previews);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.reset_active_layout_preset);
}
