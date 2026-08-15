use crate::ui::retained_host::event_bridge::UiHostEventEffects;

pub(crate) fn merge_effects(target: &mut UiHostEventEffects, source: UiHostEventEffects) {
    target.merge_shell_content_scope_state_from(&source);
    target.merge_dirty_domains(source.dirty_domains());
    target.sync_viewport_chrome |= source.sync_viewport_chrome;
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

#[cfg(test)]
mod tests {
    use super::merge_effects;
    use crate::ui::retained_host::event_bridge::{HostShellContentScope, UiHostEventEffects};
    use crate::ui::workbench::layout::ActivityDrawerSlot;
    use crate::ui::workbench::view::ViewInstanceId;

    #[test]
    fn merging_distinct_shell_content_scopes_disables_the_single_target_scope() {
        let mut first = UiHostEventEffects::default();
        first.reuse_layout_for_shell_content(HostShellContentScope::new(
            ActivityDrawerSlot::LeftTop,
            ViewInstanceId::new("editor.hierarchy#main"),
        ));
        let mut second = UiHostEventEffects::default();
        second.reuse_layout_for_shell_content(HostShellContentScope::new(
            ActivityDrawerSlot::RightTop,
            ViewInstanceId::new("editor.inspector#main"),
        ));

        merge_effects(&mut first, second);

        assert_eq!(first.shell_content_scope(), None);
    }

    #[test]
    fn merging_unscoped_presentation_disables_the_shell_content_scope() {
        let mut scoped = UiHostEventEffects::default();
        scoped.reuse_layout_for_shell_content(HostShellContentScope::new(
            ActivityDrawerSlot::LeftTop,
            ViewInstanceId::new("editor.hierarchy#main"),
        ));
        let mut global = UiHostEventEffects::default();
        global.request_presentation();

        merge_effects(&mut scoped, global);

        assert_eq!(scoped.shell_content_scope(), None);
    }

    #[test]
    fn merging_paint_only_preserves_the_shell_content_scope() {
        let scope = HostShellContentScope::new(
            ActivityDrawerSlot::LeftTop,
            ViewInstanceId::new("editor.hierarchy#main"),
        );
        let mut scoped = UiHostEventEffects::default();
        scoped.reuse_layout_for_shell_content(scope.clone());
        let mut paint_only = UiHostEventEffects::default();
        paint_only.request_paint_only();

        merge_effects(&mut scoped, paint_only);

        assert_eq!(scoped.shell_content_scope(), Some(scope));
    }
}
