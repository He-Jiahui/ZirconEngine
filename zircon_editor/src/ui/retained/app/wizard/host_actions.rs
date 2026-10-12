use super::super::RetainedEditorHost;
use super::surface_actions::{build_export_wizard_surface_action, export_wizard_status_message};
use crate::ui::host::{ExportWizardPanelAction, ExportWizardPanelUpdate};
use crate::ui::retained_host::primary_host_window_id;
use zircon_runtime_interface::ui::dispatch::UiWindowId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_build_export_surface_action(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        let Some(surface_action) = build_export_wizard_surface_action(control_id, action_id) else {
            self.dispatch_build_export_action(action_id);
            return;
        };

        let options = match surface_action.action {
            ExportWizardPanelAction::GeneratePlan | ExportWizardPanelAction::Start => {
                match self.export_wizard_options(surface_action.profile_name) {
                    Ok(options) => Some(options),
                    Err(error) => {
                        self.set_status_line(format!("Build/export wizard action failed: {error}"));
                        return;
                    }
                }
            }
            ExportWizardPanelAction::Cancel | ExportWizardPanelAction::Poll => None,
        };

        let window_id = self
            .callback_source_window
            .as_ref()
            .map(|window_id| UiWindowId::new(window_id.0.clone()))
            .unwrap_or_else(primary_host_window_id);
        match self.desktop_export_wizard_sessions.dispatch_profile_action(
            surface_action.profile_name,
            surface_action.action,
            options,
            window_id,
        ) {
            Ok(update) => {
                self.apply_export_wizard_update(surface_action.profile_name, &update);
            }
            Err(error) => {
                self.set_status_line(format!("Build/export wizard action failed: {error}"));
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn poll_desktop_export_wizard_sessions(&mut self) {
        let updates = self.desktop_export_wizard_sessions.poll_all();
        if updates.is_empty() {
            return;
        }
        for (profile_name, result) in updates {
            match result {
                Ok(update) => {
                    if update.events_drained > 0 || update.snapshot.is_terminal() {
                        self.set_status_line(export_wizard_status_message(
                            profile_name.as_str(),
                            &update,
                        ));
                    }
                }
                Err(error) => self.set_status_line(format!(
                    "Build/export wizard poll failed for {profile_name}: {error}"
                )),
            }
        }
        self.mark_layout_dirty();
    }

    fn apply_export_wizard_update(&mut self, profile_name: &str, update: &ExportWizardPanelUpdate) {
        self.mark_layout_dirty();
        self.set_status_line(export_wizard_status_message(profile_name, update));
    }
}
