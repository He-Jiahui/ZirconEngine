use super::super::super::*;
use crate::core::settings::{SettingColorChannel, SettingNumericStepDirection};
use crate::ui::settings::{
    SETTINGS_CATEGORY_CHANGED_ACTION_ID, SETTINGS_DECREMENT_COLOR_ALPHA_ACTION_ID,
    SETTINGS_DECREMENT_COLOR_BLUE_ACTION_ID, SETTINGS_DECREMENT_COLOR_GREEN_ACTION_ID,
    SETTINGS_DECREMENT_COLOR_RED_ACTION_ID, SETTINGS_DECREMENT_NUMBER_ACTION_ID,
    SETTINGS_INCREMENT_COLOR_ALPHA_ACTION_ID, SETTINGS_INCREMENT_COLOR_BLUE_ACTION_ID,
    SETTINGS_INCREMENT_COLOR_GREEN_ACTION_ID, SETTINGS_INCREMENT_COLOR_RED_ACTION_ID,
    SETTINGS_INCREMENT_NUMBER_ACTION_ID, SETTINGS_OPEN_COLOR_ACTION_ID,
    SETTINGS_OPEN_ENUM_ACTION_ID, SETTINGS_RESET_OVERRIDE_ACTION_ID,
    SETTINGS_RETRY_PERSISTENCE_ACTION_ID, SETTINGS_SELECT_ENUM_ACTION_ID,
    SETTINGS_TOGGLE_BOOL_ACTION_ID,
};
use crate::ui::template_runtime::builtin::WORKBENCH_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_componentized_workbench_option_selected(
        &mut self,
        control_id: &str,
        action_id: &str,
        option_id: &str,
    ) -> Option<Result<UiHostEventEffects, String>> {
        if !self.active_activity_window_template_document_is(WORKBENCH_WINDOW_DOCUMENT_ID) {
            return None;
        }
        if !self.workbench_window_bridge.has_control(control_id) {
            return None;
        }
        if control_id == callback_dispatch::WORKBENCH_SETTINGS_WINDOW_CONTROL_ID {
            if action_id == SETTINGS_CATEGORY_CHANGED_ACTION_ID {
                let values = match self.runtime.capture_settings_values_for_category(option_id) {
                    Ok(values) => values,
                    Err(error) => return Some(Err(error.to_string())),
                };
                match self
                    .workbench_window_bridge
                    .select_settings_category(control_id, option_id, &values)
                {
                    Ok(Some(selected)) => {
                        let mut effects = UiHostEventEffects::default();
                        if selected {
                            effects.request_paint_only();
                        }
                        return Some(Ok(effects));
                    }
                    Ok(None) => {}
                    Err(error) => return Some(Err(error.to_string())),
                }
            } else if action_id == SETTINGS_OPEN_ENUM_ACTION_ID
                || action_id == SETTINGS_OPEN_COLOR_ACTION_ID
            {
                let kind = if action_id == SETTINGS_OPEN_ENUM_ACTION_ID {
                    callback_dispatch::WorkbenchSettingsEditorKind::Enum
                } else {
                    callback_dispatch::WorkbenchSettingsEditorKind::Color
                };
                return Some(
                    self.workbench_window_bridge
                        .toggle_settings_editor(option_id, kind)
                        .map(|changed| {
                            let mut effects = UiHostEventEffects::default();
                            if changed {
                                effects.request_paint_only();
                            }
                            effects
                        })
                        .map_err(|error| error.to_string()),
                );
            } else if action_id == SETTINGS_SELECT_ENUM_ACTION_ID {
                return Some(self.select_settings_enum_option(option_id));
            } else if action_id == SETTINGS_RETRY_PERSISTENCE_ACTION_ID {
                return Some(self.retry_settings_persistence(option_id));
            } else if action_id == SETTINGS_TOGGLE_BOOL_ACTION_ID
                || action_id == SETTINGS_DECREMENT_NUMBER_ACTION_ID
                || action_id == SETTINGS_INCREMENT_NUMBER_ACTION_ID
                || action_id == SETTINGS_RESET_OVERRIDE_ACTION_ID
                || settings_color_step(action_id).is_some()
            {
                return Some(self.apply_settings_entry_action(action_id, option_id));
            }
        }
        Some(
            callback_dispatch::dispatch_componentized_workbench_option_selected(
                &self.runtime,
                &mut self.workbench_window_bridge,
                control_id,
                option_id,
            ),
        )
    }

    fn apply_settings_entry_action(
        &mut self,
        action_id: &str,
        setting_key: &str,
    ) -> Result<UiHostEventEffects, String> {
        let receipt = if let Some((channel, direction)) = settings_color_step(action_id) {
            Some(
                self.runtime
                    .step_color_setting(setting_key, channel, direction)
                    .map_err(|error| error.to_string())?,
            )
        } else {
            match action_id {
                SETTINGS_TOGGLE_BOOL_ACTION_ID => Some(
                    self.runtime
                        .toggle_bool_setting(setting_key)
                        .map_err(|error| error.to_string())?,
                ),
                SETTINGS_DECREMENT_NUMBER_ACTION_ID => Some(
                    self.runtime
                        .step_numeric_setting(setting_key, SettingNumericStepDirection::Decrement)
                        .map_err(|error| error.to_string())?,
                ),
                SETTINGS_INCREMENT_NUMBER_ACTION_ID => Some(
                    self.runtime
                        .step_numeric_setting(setting_key, SettingNumericStepDirection::Increment)
                        .map_err(|error| error.to_string())?,
                ),
                SETTINGS_RESET_OVERRIDE_ACTION_ID => self
                    .runtime
                    .reset_setting_override(setting_key)
                    .map_err(|error| error.to_string())?,
                _ => return Ok(UiHostEventEffects::default()),
            }
        };
        let Some(receipt) = receipt else {
            return Ok(UiHostEventEffects::default());
        };
        if !receipt.changed() {
            return Ok(UiHostEventEffects::default());
        }
        self.refresh_after_settings_receipt(false)
    }

    fn select_settings_enum_option(
        &mut self,
        option_id: &str,
    ) -> Result<UiHostEventEffects, String> {
        let Some(setting_key) = self
            .workbench_window_bridge
            .settings_editor_open_key(callback_dispatch::WorkbenchSettingsEditorKind::Enum)
        else {
            return Ok(UiHostEventEffects::default());
        };
        let receipt = self
            .runtime
            .set_enum_setting(&setting_key, option_id)
            .map_err(|error| error.to_string())?;
        if !receipt.changed() {
            let closed = self
                .workbench_window_bridge
                .close_settings_editor()
                .map_err(|error| error.to_string())?;
            let mut effects = UiHostEventEffects::default();
            if closed {
                effects.request_paint_only();
            }
            return Ok(effects);
        }
        self.refresh_after_settings_receipt(true)
    }

    pub(super) fn refresh_after_settings_receipt(
        &mut self,
        close_editor: bool,
    ) -> Result<UiHostEventEffects, String> {
        let revision = self
            .workbench_window_bridge
            .settings_window_revision()
            .ok_or_else(|| "Settings window is not open".to_owned())?;
        let values = self
            .runtime
            .capture_settings_values_for_category(&revision.selected_category_id)
            .map_err(|error| error.to_string())?;
        let health = self
            .runtime
            .capture_settings_persistence_health_projection();
        self.workbench_window_bridge
            .prepare_settings_persistence_health(&health)
            .map_err(|error| error.to_string())?;
        let refreshed = if close_editor {
            self.workbench_window_bridge
                .refresh_settings_values_and_close_editor(&values)
        } else {
            self.workbench_window_bridge
                .refresh_settings_values(&values)
        }
        .map_err(|error| error.to_string())?;
        let mut effects = UiHostEventEffects::default();
        if refreshed {
            effects.request_paint_only();
        }
        Ok(effects)
    }

    fn retry_settings_persistence(&mut self, scope: &str) -> Result<UiHostEventEffects, String> {
        self.runtime
            .retry_settings_persistence(scope)
            .map_err(|error| error.to_string())?;
        let health = self
            .runtime
            .capture_settings_persistence_health_projection();
        let changed = self
            .workbench_window_bridge
            .prepare_settings_persistence_health(&health)
            .map_err(|error| error.to_string())?;
        let mut effects = UiHostEventEffects::default();
        if changed {
            self.workbench_window_bridge
                .refresh_prepared_state_change()
                .map_err(|error| error.to_string())?;
            effects.request_paint_only();
        }
        Ok(effects)
    }
}

fn settings_color_step(
    action_id: &str,
) -> Option<(SettingColorChannel, SettingNumericStepDirection)> {
    match action_id {
        SETTINGS_DECREMENT_COLOR_RED_ACTION_ID => Some((
            SettingColorChannel::Red,
            SettingNumericStepDirection::Decrement,
        )),
        SETTINGS_INCREMENT_COLOR_RED_ACTION_ID => Some((
            SettingColorChannel::Red,
            SettingNumericStepDirection::Increment,
        )),
        SETTINGS_DECREMENT_COLOR_GREEN_ACTION_ID => Some((
            SettingColorChannel::Green,
            SettingNumericStepDirection::Decrement,
        )),
        SETTINGS_INCREMENT_COLOR_GREEN_ACTION_ID => Some((
            SettingColorChannel::Green,
            SettingNumericStepDirection::Increment,
        )),
        SETTINGS_DECREMENT_COLOR_BLUE_ACTION_ID => Some((
            SettingColorChannel::Blue,
            SettingNumericStepDirection::Decrement,
        )),
        SETTINGS_INCREMENT_COLOR_BLUE_ACTION_ID => Some((
            SettingColorChannel::Blue,
            SettingNumericStepDirection::Increment,
        )),
        SETTINGS_DECREMENT_COLOR_ALPHA_ACTION_ID => Some((
            SettingColorChannel::Alpha,
            SettingNumericStepDirection::Decrement,
        )),
        SETTINGS_INCREMENT_COLOR_ALPHA_ACTION_ID => Some((
            SettingColorChannel::Alpha,
            SettingNumericStepDirection::Increment,
        )),
        _ => None,
    }
}
