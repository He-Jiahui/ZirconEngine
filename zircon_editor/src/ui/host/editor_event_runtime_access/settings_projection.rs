use crate::core::extension::CapabilitySet;
use crate::core::settings::{
    SettingsMutationContext, SettingsMutationCoordinator, SettingsMutationError,
    SettingsPersistenceRetryContext,
};
use crate::ui::host::EditorHostEventController;
use crate::ui::settings::{SettingsPersistenceHealthProjection, SettingsWindowProjection};

impl EditorHostEventController {
    pub(crate) fn capture_settings_window_projection(
        &self,
    ) -> Result<SettingsWindowProjection, crate::ui::host::EditorError> {
        let manager = self.enter_editor_manager()?;
        Ok({
            let settings = self.context().settings().snapshot();
            let (contributions, capabilities) = {
                let inner = self.shell().lock();
                let capability_snapshot = manager.capability_snapshot();
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
        })
    }

    pub(crate) fn capture_settings_values_for_category(
        &self,
        category_id: &str,
    ) -> Result<SettingsMutationContext, SettingsMutationError> {
        capture_values_for_category(self.context().settings_mutations(), category_id)
    }

    pub(crate) fn capture_settings_values_for_projection_category(
        &self,
        projection: &SettingsWindowProjection,
        category_id: &str,
    ) -> Result<SettingsMutationContext, SettingsMutationError> {
        capture_values_for_projection_category(
            self.context().settings_mutations(),
            projection,
            category_id,
        )
    }

    pub(crate) fn settings_values_context_is_current(
        &self,
        context: &SettingsMutationContext,
    ) -> bool {
        self.context()
            .settings_mutations()
            .context_is_current(context)
    }

    pub(crate) fn check_settings_values_context(
        &self,
        context: &SettingsMutationContext,
    ) -> Result<(), SettingsMutationError> {
        self.context()
            .settings_mutations()
            .check_values_context(context)
    }

    pub(crate) fn capture_settings_persistence_health_with_retry(
        &self,
    ) -> Result<
        (
            SettingsPersistenceHealthProjection,
            Option<SettingsPersistenceRetryContext>,
        ),
        SettingsMutationError,
    > {
        let (snapshot, contexts) = self
            .context()
            .settings_mutations()
            .capture_persistence_health_with_retry()?;
        let health = SettingsPersistenceHealthProjection::capture(snapshot, self.context().i18n());
        let context = contexts.into_iter().find(|context| {
            let scope = match context.scope() {
                crate::core::settings::SettingsScope::Project => "project",
                crate::core::settings::SettingsScope::User => "user",
                crate::core::settings::SettingsScope::Session => "",
            };
            scope == health.retry_scope_name()
        });
        Ok((health, context))
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

fn capture_values_for_category(
    mutations: &SettingsMutationCoordinator,
    category_id: &str,
) -> Result<SettingsMutationContext, SettingsMutationError> {
    let snapshot = mutations.authority().snapshot();
    let category_path = category_id
        .strip_prefix("builtin|")
        .filter(|category_path| !category_path.is_empty());
    if let Some(category_path) = category_path {
        mutations.capture_values(snapshot.catalog().keys_for_category_subtree(category_path))
    } else {
        mutations.capture_values(std::iter::empty())
    }
}

fn capture_values_for_projection_category(
    mutations: &SettingsMutationCoordinator,
    projection: &SettingsWindowProjection,
    category_id: &str,
) -> Result<SettingsMutationContext, SettingsMutationError> {
    let snapshot = mutations.authority().snapshot();
    let keys = category_id
        .strip_prefix("builtin|")
        .filter(|path| !path.is_empty())
        .into_iter()
        .flat_map(|path| snapshot.catalog().keys_for_category_subtree(path));
    // Even an empty/plugin category must belong to this projection's exact batch.
    mutations.capture_values_at_generation(projection.settings_generation(), keys)
}

#[cfg(test)]
#[path = "../tests/settings_projection.rs"]
mod tests;
