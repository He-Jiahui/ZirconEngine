use crate::ui::retained_host::apply_host_appearance_from_tokens;
use crate::ui::retained_host::host_contract::data::HostTextInputFocusData;
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::UiHostWindow;
use crate::ui::settings::{
    SETTINGS_COMMIT_CHORD_ACTION_ID, SETTINGS_COMMIT_NUMERIC_ACTION_ID,
    SETTINGS_COMMIT_STRING_ACTION_ID,
};
use crate::ui::v2_design_tokens::install_editor_v2_design_tokens;

use super::callback_dispatch::{WorkbenchSettingsOpenState, WorkbenchSettingsWindowRevision};
use super::{HostInvalidationMask, RetainedEditorHost};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn retire_settings_native_text_focus(&mut self) {
        retire_settings_native_draft(
            &self.ui,
            &mut self
                .workbench_window_bridge
                .content_drawer_native_previous_focus,
        );
        if let Some(committed) = self.committed_shell_state.as_ref() {
            for floating in &committed.model.floating_windows {
                if let Some(ui) = self.native_window_presenters.window(&floating.window_id) {
                    retire_settings_native_draft(
                        &ui,
                        &mut self
                            .workbench_window_bridge
                            .content_drawer_native_previous_focus,
                    );
                }
            }
        }
    }

    pub(super) fn sync_settings_projections(&mut self) {
        let snapshot = self.runtime.context().settings().snapshot();
        match self
            .workbench_window_bridge
            .set_preview_features(snapshot.show_preview_features())
        {
            Ok(true) => self.mark_presentation_dirty(),
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
        if install_editor_v2_design_tokens(snapshot.as_ref()) {
            apply_host_appearance_from_tokens(
                snapshot.design_tokens_handle(),
                Some(snapshot.generation()),
            );
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
        // Persistence can finish without changing setting values or publishing a toast.
        // Keep its presentation current even when the directory and displayed batch are current.
        let (health, retry_context) = match self
            .runtime
            .capture_settings_persistence_health_with_retry()
        {
            Ok(captured) => captured,
            Err(error) => {
                let context_is_current = self
                    .workbench_window_bridge
                    .settings_values_context()
                    .is_some_and(|context| {
                        self.runtime.settings_values_context_is_current(context)
                    });
                if self
                    .workbench_window_bridge
                    .settings_persistence_capture_failed(context_is_current)
                {
                    self.retire_settings_native_text_focus();
                }
                self.set_status_line(error.to_string());
                return;
            }
        };
        match self
            .workbench_window_bridge
            .refresh_settings_persistence_health(&health, retry_context.as_ref())
        {
            Ok(true) => self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA),
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
        let extensions_are_current = match self.runtime.extension_projection_is_current(
            revision.contribution_generation,
            &revision.enabled_capabilities,
        ) {
            Ok(current) => current,
            Err(error) => {
                self.retire_settings_native_text_focus();
                self.workbench_window_bridge
                    .invalidate_settings_values_context();
                self.set_status_line(error.to_string());
                return;
            }
        };
        let current_locale = self.runtime.context().i18n().active_locale();
        let directory_is_stale =
            !extensions_are_current || revision.locale != current_locale.as_str();
        let context_is_current = self
            .workbench_window_bridge
            .settings_values_context()
            .is_some_and(|context| self.runtime.settings_values_context_is_current(context));
        let values_are_stale =
            !context_is_current || revision.settings_generation != current_settings_generation;

        if directory_is_stale || values_are_stale {
            self.retire_settings_native_text_focus();
        }
        let refreshed = if directory_is_stale {
            self.workbench_window_bridge
                .invalidate_settings_values_context();
            let projection = (match self.runtime.capture_settings_window_projection() {
                Ok(value) => value,
                Err(error) => {
                    self.set_status_line(error.to_string());
                    return;
                }
            });
            let selected_category_id = WorkbenchSettingsOpenState::retain_category_id(
                &projection,
                &revision.selected_category_id,
            );
            let values = match self
                .runtime
                .capture_settings_values_for_projection_category(&projection, &selected_category_id)
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
                &health,
            )
            .with_persistence_retry_context(retry_context);
            self.workbench_window_bridge.refresh_settings_window(state)
        } else if values_are_stale {
            self.workbench_window_bridge
                .invalidate_settings_values_context();
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
        self.retire_settings_native_text_focus();
        self.workbench_window_bridge
            .invalidate_settings_values_context();
        self.workbench_window_bridge
            .invalidate_settings_persistence_retry_context();
        let projection = (match self.runtime.capture_settings_window_projection() {
            Ok(value) => value,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        });
        let selected_category_id = WorkbenchSettingsOpenState::initial_category_id(&projection);
        let values = match self
            .runtime
            .capture_settings_values_for_projection_category(&projection, &selected_category_id)
        {
            Ok(values) => values,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let (health, retry_context) = match self
            .runtime
            .capture_settings_persistence_health_with_retry()
        {
            Ok(captured) => captured,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let state = WorkbenchSettingsOpenState::from_projection(
            &projection,
            selected_category_id,
            &values,
            &health,
        )
        .with_persistence_retry_context(retry_context);
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

fn is_settings_text_focus(focus: &HostTextInputFocusData) -> bool {
    focus.is_active()
        && matches!(
            focus.commit_action_id.as_str(),
            SETTINGS_COMMIT_STRING_ACTION_ID
                | SETTINGS_COMMIT_NUMERIC_ACTION_ID
                | SETTINGS_COMMIT_CHORD_ACTION_ID
        )
}

fn retire_settings_native_draft(
    ui: &UiHostWindow,
    saved_drawer_focus: &mut Option<HostTextInputFocusData>,
) -> bool {
    // The drawer must not restore a draft from a replaced settings batch.
    let retired_saved_focus = saved_drawer_focus
        .as_ref()
        .is_some_and(is_settings_text_focus);
    if retired_saved_focus {
        *saved_drawer_focus = None;
    }
    if !is_settings_text_focus(&ui.global::<UiHostContext>().get_text_input_focus()) {
        return retired_saved_focus;
    }
    let result = ui.cancel_focused_text_input();
    if let Some(frame) = result.damage_region() {
        ui.request_frame_update_region(frame);
    } else {
        ui.request_frame_update();
    }
    true
}

#[cfg(test)]
#[path = "tests/native_focus.rs"]
mod native_focus_retirement_tests;
