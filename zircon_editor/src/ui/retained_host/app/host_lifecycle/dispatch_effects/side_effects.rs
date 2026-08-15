use crate::ui::retained_host::app::RetainedEditorHost;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::super::workbench_notifications::{
    import_model_completed_toast, import_model_failed_toast,
};

use super::super::super::scene_picker_session::ScenePickerMode;

impl RetainedEditorHost {
    pub(super) fn apply_dispatch_side_effects(&mut self, effects: &UiHostEventEffects) {
        if !effects.toast_notifications.is_empty() {
            self.publish_activity_toasts(&effects.toast_notifications);
        }
        if effects.close_active_project {
            if let Err(error) = self.close_project_from_workbench() {
                self.set_status_line(error);
            }
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
                    if let Some(notification) = import_model_completed_toast() {
                        self.publish_activity_toasts(std::slice::from_ref(&notification));
                    }
                }
                Err(error) => {
                    if let Some(notification) = import_model_failed_toast(&error) {
                        self.publish_activity_toasts(std::slice::from_ref(&notification));
                    }
                    self.set_status_line(error);
                }
            }
        }
        if effects.open_command_palette_requested {
            self.open_workbench_command_palette();
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
        self.sync_activity_notifications();
    }
}
