mod action_ids;
mod localized_setting;
mod persistence_health_projection;
mod settings_localization_domain;
mod settings_navigation_category;
mod settings_window_projection;

pub(crate) use action_ids::{
    SETTINGS_CAPTURE_CHORD_ACTION_ID, SETTINGS_CATEGORY_CHANGED_ACTION_ID,
    SETTINGS_COMMIT_CHORD_ACTION_ID, SETTINGS_COMMIT_STRING_ACTION_ID,
    SETTINGS_DECREMENT_COLOR_ALPHA_ACTION_ID, SETTINGS_DECREMENT_COLOR_BLUE_ACTION_ID,
    SETTINGS_DECREMENT_COLOR_GREEN_ACTION_ID, SETTINGS_DECREMENT_COLOR_RED_ACTION_ID,
    SETTINGS_DECREMENT_NUMBER_ACTION_ID, SETTINGS_EDIT_STRING_ACTION_ID,
    SETTINGS_INCREMENT_COLOR_ALPHA_ACTION_ID, SETTINGS_INCREMENT_COLOR_BLUE_ACTION_ID,
    SETTINGS_INCREMENT_COLOR_GREEN_ACTION_ID, SETTINGS_INCREMENT_COLOR_RED_ACTION_ID,
    SETTINGS_INCREMENT_NUMBER_ACTION_ID, SETTINGS_OPEN_COLOR_ACTION_ID,
    SETTINGS_OPEN_ENUM_ACTION_ID, SETTINGS_RESET_OVERRIDE_ACTION_ID,
    SETTINGS_RETRY_PERSISTENCE_ACTION_ID, SETTINGS_SELECT_ENUM_ACTION_ID,
    SETTINGS_TOGGLE_BOOL_ACTION_ID,
};
pub use localized_setting::LocalizedSetting;
pub(crate) use persistence_health_projection::SettingsPersistenceHealthProjection;
pub use settings_localization_domain::SettingsLocalizationDomain;
pub use settings_navigation_category::SettingsNavigationCategory;
pub use settings_window_projection::SettingsWindowProjection;

#[cfg(test)]
mod tests;
