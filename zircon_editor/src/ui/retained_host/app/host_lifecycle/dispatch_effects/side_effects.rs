use crate::ui::retained_host::app::RetainedEditorHost;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::super::workbench_notifications::{
    import_model_completed_toast, import_model_failed_toast,
};

use super::super::super::scene_picker_session::ScenePickerMode;

impl RetainedEditorHost {
    pub(super) fn apply_dispatch_side_effects(&mut self, effects: &UiHostEventEffects) {
        let mut activity_toasts_published = false;
        if !effects.toast_notifications.is_empty() {
            activity_toasts_published |= self.enqueue_activity_toasts(&effects.toast_notifications);
        }
        if effects.close_active_project {
            if let Err(error) = self.request_project_close() {
                self.set_status_line(error);
            }
        }
        if effects.save_all_documents {
            self.request_document_save_all();
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
                Ok(()) => self.set_status_line("Model import queued"),
                Err(error) => {
                    if let Some(notification) = import_model_failed_toast(&error) {
                        activity_toasts_published |=
                            self.enqueue_activity_toasts(std::slice::from_ref(&notification));
                    }
                    self.set_status_line(error);
                }
            }
        }
        if let Some(request) = effects.asset_deletion_requested.as_ref() {
            match self.request_asset_deletion(&request.asset_uuid) {
                Ok(()) => self.set_status_line(format!("Deleting asset {}", request.asset_uuid)),
                Err(error) => self.set_status_line(error),
            }
        }
        if let Some(request) = effects.asset_relocation_requested.as_ref() {
            match self.request_asset_relocation(&request.asset_uuid, &request.target_locator) {
                Ok(()) => {
                    self.set_status_line(format!("Moving asset to {}", request.target_locator))
                }
                Err(error) => self.set_status_line(error),
            }
        }
        if effects.open_command_palette_requested {
            self.open_workbench_command_palette();
        }
        if effects.open_settings_window_requested {
            self.open_workbench_settings_window();
        }
        if effects.open_scene_picker_requested {
            self.open_workbench_scene_picker(ScenePickerMode::Open);
        }
        if effects.create_scene_picker_requested {
            self.open_workbench_scene_picker(ScenePickerMode::Create);
        }
        if effects.present_welcome_surface {
            if let Err(error) = self.present_welcome_surface(
                "Open an existing project or create a renderable empty project.",
            ) {
                self.set_status_line(error);
            }
        }
        if activity_toasts_published {
            self.refresh_activity_notification_presentation();
        } else {
            self.sync_activity_notifications();
        }
    }
}
