use crate::ui::retained_host::app::RetainedEditorHost;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::retained_host::workbench_notifications::{
    workbench_import_model_completed_notification, workbench_import_model_failed_notification,
};

use super::super::super::scene_picker_session::ScenePickerMode;

impl RetainedEditorHost {
    pub(super) fn apply_dispatch_side_effects(&mut self, effects: &UiHostEventEffects) {
        if !effects.workbench_notifications.is_empty() {
            self.publish_workbench_notifications(&effects.workbench_notifications);
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
        self.sync_pending_play_decisions();
    }
}
