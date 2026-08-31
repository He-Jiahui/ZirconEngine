use std::collections::HashMap;

use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    PluginMaturity, RuntimePluginAvailabilityCategory, RuntimePluginAvailabilityReport,
};

use super::generation::{
    row_from_descriptor, row_from_runtime, RuntimePluginAvailabilityDescriptorRef,
    RuntimePluginAvailabilityGenerationBuilder, RuntimePluginAvailabilityReason,
};
use super::selection::{merge_runtime_plugin_selection, project_manifest_plugin_selections};
#[cfg(test)]
use super::RuntimePluginAvailabilitySelectionMetrics;
use super::{
    RuntimePluginAvailabilityGeneration, RuntimePluginAvailabilityProjection,
    RuntimeProfileDescriptor,
};

impl<'descriptor, 'provider> RuntimePluginAvailabilityProjection<'descriptor, 'provider> {
    pub fn report_for_profile_defaults(
        &self,
        profile: &RuntimeProfileDescriptor,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityReport {
        self.generation_for_profile_defaults(profile, require_external_provider)
            .materialize_report()
    }

    pub fn generation_for_profile_defaults(
        &self,
        profile: &RuntimeProfileDescriptor,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityGeneration<'descriptor> {
        let mut plugins = Vec::with_capacity(
            profile
                .default_plugins
                .len()
                .saturating_add(profile.optional_plugins.len()),
        );
        let mut positions = HashMap::with_capacity(plugins.capacity());
        for (plugin_id, required) in profile
            .default_plugins
            .iter()
            .map(|plugin| (plugin.id.clone(), plugin.required))
            .chain(
                profile
                    .optional_plugins
                    .iter()
                    .cloned()
                    .map(|plugin_id| (plugin_id, false)),
            )
        {
            merge_runtime_plugin_selection(&mut plugins, &mut positions, plugin_id, required);
        }
        self.generation_for_runtime_plugins(profile, plugins, require_external_provider)
    }

    pub fn report_for_manifest(
        &self,
        profile: &RuntimeProfileDescriptor,
        manifest: &ProjectPluginManifest,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityReport {
        self.generation_for_manifest(profile, manifest, require_external_provider)
            .materialize_report()
    }

    pub fn generation_for_manifest(
        &self,
        profile: &RuntimeProfileDescriptor,
        manifest: &ProjectPluginManifest,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityGeneration<'descriptor> {
        let selection_projection = project_manifest_plugin_selections(profile, manifest);
        self.generation_for_runtime_plugins(
            profile,
            selection_projection.plugins,
            require_external_provider,
        )
    }

    #[cfg(test)]
    pub fn report_for_manifest_with_metrics(
        &self,
        profile: &RuntimeProfileDescriptor,
        manifest: &ProjectPluginManifest,
        require_external_provider: bool,
    ) -> (
        RuntimePluginAvailabilityReport,
        RuntimePluginAvailabilitySelectionMetrics,
    ) {
        let selection_projection = project_manifest_plugin_selections(profile, manifest);
        (
            self.generation_for_runtime_plugins(
                profile,
                selection_projection.plugins,
                require_external_provider,
            )
            .materialize_report(),
            selection_projection.metrics,
        )
    }

    fn generation_for_runtime_plugins(
        &self,
        profile: &RuntimeProfileDescriptor,
        plugins: impl IntoIterator<Item = (RuntimePluginId, bool)>,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityGeneration<'descriptor> {
        let mut generation = RuntimePluginAvailabilityGenerationBuilder::new();
        for (plugin_id, required) in plugins {
            self.append_plugin_availability(
                profile,
                plugin_id,
                required,
                require_external_provider,
                &mut generation,
            );
        }
        generation.finish()
    }

    fn append_plugin_availability(
        &self,
        profile: &RuntimeProfileDescriptor,
        plugin_id: RuntimePluginId,
        required: bool,
        require_external_provider: bool,
        generation: &mut RuntimePluginAvailabilityGenerationBuilder<'descriptor>,
    ) {
        let Some(descriptor) = self.descriptors.get(&plugin_id) else {
            if plugin_id == RuntimePluginId::Ui && !cfg!(feature = "ui") {
                generation.push(
                    required,
                    row_from_runtime(
                        plugin_id,
                        required,
                        PluginMaturity::Core,
                        RuntimePluginAvailabilityCategory::Stub,
                        RuntimePluginAvailabilityReason::BuiltinUnavailable,
                    ),
                );
                return;
            }
            if plugin_id == RuntimePluginId::Ui && cfg!(feature = "ui") {
                generation.push(
                    false,
                    row_from_runtime(
                        plugin_id,
                        required,
                        PluginMaturity::Core,
                        RuntimePluginAvailabilityCategory::Available,
                        RuntimePluginAvailabilityReason::BuiltinAvailable,
                    ),
                );
                return;
            }
            generation.push(
                required,
                row_from_runtime(
                    plugin_id,
                    required,
                    PluginMaturity::Stub,
                    RuntimePluginAvailabilityCategory::Stub,
                    RuntimePluginAvailabilityReason::MissingCatalog,
                ),
            );
            return;
        };
        if !supports_target(descriptor, profile.target_mode) {
            generation.push(
                required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::BlockedByTarget,
                    RuntimePluginAvailabilityReason::TargetUnsupported(profile.target_mode),
                ),
            );
            return;
        }
        if descriptor.maturity == PluginMaturity::Externalized {
            generation.push(
                !profile.allow_externalized_required_plugins && required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::ExternalizedMissing,
                    RuntimePluginAvailabilityReason::Externalized,
                ),
            );
            return;
        }
        if descriptor.maturity == PluginMaturity::Stub {
            generation.push(
                required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::Stub,
                    RuntimePluginAvailabilityReason::Stub,
                ),
            );
            return;
        }
        if !descriptor.maturity.meets_minimum(profile.minimum_maturity) {
            generation.push(
                required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::BlockedByMaturity,
                    RuntimePluginAvailabilityReason::BelowMinimum(profile.minimum_maturity),
                ),
            );
            return;
        }
        if self.linked_plugin_ids.contains(descriptor.package_id) {
            generation.push(
                false,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::Linked,
                    RuntimePluginAvailabilityReason::Linked,
                ),
            );
            return;
        }
        if self
            .native_dynamic_plugin_ids
            .contains(descriptor.package_id)
        {
            generation.push(
                false,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::NativeDynamic,
                    RuntimePluginAvailabilityReason::NativeDynamic,
                ),
            );
            return;
        }
        if require_external_provider && !builtin_runtime_domain_is_available(&descriptor.runtime_id)
        {
            generation.push(
                !profile.allow_externalized_required_plugins && required,
                row_from_descriptor(
                    descriptor,
                    required,
                    RuntimePluginAvailabilityCategory::ExternalizedMissing,
                    RuntimePluginAvailabilityReason::MissingProvider,
                ),
            );
            return;
        }
        generation.push(
            false,
            row_from_descriptor(
                descriptor,
                required,
                RuntimePluginAvailabilityCategory::Available,
                RuntimePluginAvailabilityReason::Available,
            ),
        );
    }
}

fn builtin_runtime_domain_is_available(id: &RuntimePluginId) -> bool {
    id == &RuntimePluginId::Ui && cfg!(feature = "ui")
}

fn supports_target(
    descriptor: &RuntimePluginAvailabilityDescriptorRef<'_>,
    target: RuntimeTargetMode,
) -> bool {
    descriptor.target_modes.is_empty() || descriptor.target_modes.contains(&target)
}
