use crate::ui::slint_host::event_bridge::UiHostEventEffects;

pub(crate) fn merge_effects(target: &mut UiHostEventEffects, source: UiHostEventEffects) {
    target.layout_dirty |= source.layout_dirty;
    target.render_dirty |= source.render_dirty;
    target.presentation_dirty |= source.presentation_dirty;
    target.sync_asset_workspace |= source.sync_asset_workspace;
    target.refresh_asset_details |= source.refresh_asset_details;
    target.refresh_visible_asset_previews |= source.refresh_visible_asset_previews;
    target.import_model_requested |= source.import_model_requested;
    target.reset_active_layout_preset |= source.reset_active_layout_preset;
}
