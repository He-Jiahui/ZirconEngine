use crate::core::extension::CapabilitySet;
use crate::core::settings::{ResolvedSettingsBatch, SettingsError};
use crate::ui::host::EditorHostEventController;
use crate::ui::settings::{SettingsPersistenceHealthProjection, SettingsWindowProjection};

impl EditorHostEventController {
    pub(crate) fn capture_settings_window_projection(&self) -> SettingsWindowProjection {
        let settings = self.context().settings().snapshot();
        let (contributions, capabilities) = {
            let inner = self.shell().lock();
            let capability_snapshot = inner.manager.capability_snapshot();
            let capabilities = capability_snapshot
                .enabled_capabilities()
                .iter()
                .cloned()
                .collect::<CapabilitySet>();
            (inner.contributions.snapshot(), capabilities)
        };
        SettingsWindowProjection::capture(
            settings.as_ref(),
            &contributions,
            &capabilities,
            self.context().i18n(),
        )
    }

    pub(crate) fn capture_settings_values_for_category(
        &self,
        category_id: &str,
    ) -> Result<ResolvedSettingsBatch, SettingsError> {
        let settings = self.context().settings();
        let snapshot = settings.snapshot();
        let keys = category_id
            .strip_prefix("builtin|")
            .filter(|category_path| !category_path.is_empty())
            .map_or(&[][..], |category_path| {
                snapshot.catalog().keys_for_category_path(category_path)
            });
        settings.resolved_settings(keys)
    }

    pub(crate) fn capture_settings_persistence_health_projection(
        &self,
    ) -> SettingsPersistenceHealthProjection {
        SettingsPersistenceHealthProjection::capture(
            self.context()
                .settings_mutations()
                .persistence_health_snapshot(),
            self.context().i18n(),
        )
    }
}
