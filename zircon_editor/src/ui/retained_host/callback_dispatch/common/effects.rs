use crate::ui::retained_host::event_bridge::UiHostEventEffects;

pub(crate) fn merge_effects(target: &mut UiHostEventEffects, source: UiHostEventEffects) {
    target.merge_dirty_domains(source.dirty_domains());
    if source.active_layout_preset_name.is_some() {
        target.active_layout_preset_name = source.active_layout_preset_name;
    }
    target.present_welcome_surface |= source.present_welcome_surface;
    target.sync_asset_workspace |= source.sync_asset_workspace;
    target.refresh_asset_details |= source.refresh_asset_details;
    target.refresh_visible_asset_previews |= source.refresh_visible_asset_previews;
    target.import_model_requested |= source.import_model_requested;
    target.reset_active_layout_preset |= source.reset_active_layout_preset;
    target.open_command_palette_requested |= source.open_command_palette_requested;
    target.open_scene_picker_requested |= source.open_scene_picker_requested;
    target.create_scene_picker_requested |= source.create_scene_picker_requested;
    target
        .toast_notifications
        .extend(source.toast_notifications);
}
