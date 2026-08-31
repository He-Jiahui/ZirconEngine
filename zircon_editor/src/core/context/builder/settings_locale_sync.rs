//! Settings-to-i18n synchronization installed by the context composition root.

use std::sync::Arc;

use crate::core::i18n::EditorI18nService;
use crate::core::recovery::{AutosavePolicy, EditorAutosaveService};
use crate::core::settings::{
    SettingChange, SettingsChangeSubscriber, SettingsSnapshot, EDITOR_AUTOSAVE_INTERVAL_SECS_KEY,
    EDITOR_LOCALE_KEY,
};

pub(super) struct EditorSettingsChangeSubscriber {
    pub(super) i18n: Arc<EditorI18nService>,
    pub(super) autosave: Arc<EditorAutosaveService>,
}

impl SettingsChangeSubscriber for EditorSettingsChangeSubscriber {
    fn settings_changed(&self, changes: &[SettingChange], snapshot: &SettingsSnapshot) {
        if changes
            .iter()
            .any(|change| change.key.as_str() == EDITOR_LOCALE_KEY)
        {
            if let Err(error) = self.i18n.synchronize_settings_snapshot(snapshot) {
                tracing::error!(%error, "validated editor locale could not be hot-applied");
            }
        }
        if changes
            .iter()
            .any(|change| change.key.as_str() == EDITOR_AUTOSAVE_INTERVAL_SECS_KEY)
        {
            match AutosavePolicy::new(snapshot.autosave_interval()) {
                Ok(policy) => self.autosave.update_policy(policy),
                Err(error) => {
                    tracing::error!(%error, "validated autosave cadence could not be hot-applied")
                }
            }
        }
    }
}
