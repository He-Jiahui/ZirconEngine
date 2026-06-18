use crate::ui::retained_host::app::{HostInvalidationMask, RetainedEditorHost};
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::workbench_notifications::{
    workbench_dispatch_error_notification, workbench_import_model_completed_notification,
    workbench_import_model_failed_notification,
};
use crate::ui::workbench::snapshot::StatusTaskProgressSnapshot;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn set_status_line(
        &mut self,
        message: impl Into<String>,
    ) {
        let message = message.into();
        if self.runtime.status_line() == message {
            return;
        }
        self.runtime.set_status_line(message);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(in crate::ui::retained_host::app) fn set_status_task_progress(
        &mut self,
        progress: Option<StatusTaskProgressSnapshot>,
    ) {
        if self.runtime.status_task_progress() == progress {
            return;
        }
        self.runtime.set_status_task_progress(progress);
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(in crate::ui::retained_host::app) fn apply_dispatch_effects(
        &mut self,
        effects: UiHostEventEffects,
    ) {
        if let Some(name) = effects.active_layout_preset_name.clone() {
            self.active_layout_preset = Some(name);
        }
        if effects.reset_active_layout_preset {
            self.active_layout_preset = None;
        }
        self.invalidate_host(effects.dirty_domains());
        self.apply_dispatch_side_effects(&effects);
    }

    pub(super) fn apply_viewport_resize_effects_in_active_recompute(
        &mut self,
        result: Result<UiHostEventEffects, String>,
    ) {
        match result {
            Ok(effects) => {
                // The active recompute immediately rebuilds chrome/model after
                // viewport resize, so only the render-domain part should carry
                // into the next tick.
                if let Some(name) = effects.active_layout_preset_name.clone() {
                    self.active_layout_preset = Some(name);
                }
                if effects.reset_active_layout_preset {
                    self.active_layout_preset = None;
                }
                let mut dirty_domains = effects.dirty_domains();
                dirty_domains.remove(HostInvalidationMask::PRESENTATION_DATA);
                self.invalidate_host(dirty_domains);
                self.apply_dispatch_side_effects(&effects);
            }
            Err(error) => self.set_status_line(error),
        }
    }

    fn apply_dispatch_side_effects(&mut self, effects: &UiHostEventEffects) {
        if !effects.workbench_notifications.is_empty() {
            self.publish_workbench_notifications(&effects.workbench_notifications);
        }
        if effects.sync_asset_workspace {
            self.sync_asset_workspace();
        }
        if effects.refresh_asset_details {
            self.refresh_selected_asset_details();
        }
        if effects.refresh_visible_asset_previews {
            self.refresh_visible_asset_previews();
        }
        if effects.import_model_requested {
            match self.import_model_into_project() {
                Ok(()) => {
                    let notification = workbench_import_model_completed_notification();
                    self.publish_workbench_notifications(std::slice::from_ref(&notification));
                }
                Err(error) => {
                    let notification = workbench_import_model_failed_notification(&error);
                    self.publish_workbench_notifications(std::slice::from_ref(&notification));
                    self.set_status_line(error);
                }
            }
        }
        if effects.open_command_palette_requested {
            self.open_workbench_command_palette();
        }
        if effects.present_welcome_surface {
            if let Err(error) = self.present_welcome_surface(
                "Open an existing project or create a renderable empty project.",
            ) {
                self.set_status_line(error);
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn apply_dispatch_result(
        &mut self,
        result: Result<UiHostEventEffects, String>,
    ) {
        match result {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => {
                let notification = workbench_dispatch_error_notification(&error);
                self.publish_workbench_notifications(std::slice::from_ref(&notification));
                self.set_status_line(error);
            }
        }
    }
}
