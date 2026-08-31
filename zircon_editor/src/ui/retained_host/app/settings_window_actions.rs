use crate::ui::retained_host::apply_host_appearance_from_tokens;
use crate::ui::v2_design_tokens::install_editor_v2_design_tokens;

use super::callback_dispatch::{WorkbenchSettingsOpenState, WorkbenchSettingsWindowRevision};
use super::{HostInvalidationMask, RetainedEditorHost};

impl RetainedEditorHost {
    pub(super) fn sync_settings_projections(&mut self) {
        let snapshot = self.editor_manager.context().settings().snapshot();
        if install_editor_v2_design_tokens(snapshot.as_ref()) {
            apply_host_appearance_from_tokens(snapshot.design_tokens());
            self.ui.sync_host_paint_theme();
            self.mark_presentation_dirty();
        }
        self.sync_open_settings_window(snapshot.generation());
    }

    fn sync_open_settings_window(&mut self, current_settings_generation: u64) {
        let Some(revision): Option<WorkbenchSettingsWindowRevision> =
            self.workbench_window_bridge.settings_window_revision()
        else {
            return;
        };
        let (current_contribution_generation, current_enabled_capabilities) =
            self.runtime.extension_projection_revision();
        let current_locale = self.runtime.context().i18n().active_locale();
        let directory_is_stale = revision.contribution_generation
            != current_contribution_generation
            || revision.enabled_capabilities != current_enabled_capabilities
            || revision.locale != current_locale.as_str();
        let values_are_stale = revision.settings_generation != current_settings_generation;

        let refreshed = if directory_is_stale {
            let projection = self.runtime.capture_settings_window_projection();
            let selected_category_id = WorkbenchSettingsOpenState::retain_category_id(
                &projection,
                &revision.selected_category_id,
            );
            let values = match self
                .runtime
                .capture_settings_values_for_category(&selected_category_id)
            {
                Ok(values) => values,
                Err(error) => {
                    self.set_status_line(error.to_string());
                    return;
                }
            };
            let state = WorkbenchSettingsOpenState::from_projection(
                &projection,
                selected_category_id,
                &values,
                &self
                    .runtime
                    .capture_settings_persistence_health_projection(),
            );
            self.workbench_window_bridge.refresh_settings_window(state)
        } else if values_are_stale {
            let values = match self
                .runtime
                .capture_settings_values_for_category(&revision.selected_category_id)
            {
                Ok(values) => values,
                Err(error) => {
                    self.set_status_line(error.to_string());
                    return;
                }
            };
            self.workbench_window_bridge
                .refresh_settings_values(&values)
        } else {
            return;
        };

        match refreshed {
            Ok(true) => self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA),
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn open_workbench_settings_window(&mut self) {
        let projection = self.runtime.capture_settings_window_projection();
        let selected_category_id = WorkbenchSettingsOpenState::initial_category_id(&projection);
        let values = match self
            .runtime
            .capture_settings_values_for_category(&selected_category_id)
        {
            Ok(values) => values,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let health = self
            .runtime
            .capture_settings_persistence_health_projection();
        let state = WorkbenchSettingsOpenState::from_projection(
            &projection,
            selected_category_id,
            &values,
            &health,
        );
        if let Err(error) = self.workbench_window_bridge.close_command_palette() {
            self.set_status_line(error.to_string());
            return;
        }
        match self.workbench_window_bridge.open_settings_window(state) {
            Ok(true) => {
                self.scene_picker_session = None;
                self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
            }
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn settings_window_scrolled(
        &mut self,
        category_scroll_offset: f32,
        setting_scroll_offset: f32,
    ) {
        match self
            .workbench_window_bridge
            .update_settings_scroll_offsets(category_scroll_offset, setting_scroll_offset)
        {
            Ok(true) => self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA),
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
