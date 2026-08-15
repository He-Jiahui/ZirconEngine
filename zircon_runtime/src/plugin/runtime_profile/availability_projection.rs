use std::collections::{HashMap, HashSet};

use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ExportPackagingStrategy, ProjectPluginManifest};
use crate::plugin::{
    PluginMaturity, RuntimePluginAvailabilityCategory, RuntimePluginAvailabilityReport,
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

use super::RuntimeProfileDescriptor;

mod generation;

use generation::{
    row_from_descriptor, row_from_runtime, RuntimePluginAvailabilityDescriptorRef,
    RuntimePluginAvailabilityGenerationBuilder, RuntimePluginAvailabilityReason,
};
pub use generation::{
    RuntimePluginAvailabilityGeneration, RuntimePluginAvailabilityRow,
    RuntimePluginAvailabilitySummary,
};

/// Immutable membership/index state shared by every availability consumer in
/// one bootstrap or export generation.
pub(crate) struct RuntimePluginAvailabilityProjection<'descriptor, 'provider> {
    descriptors: HashMap<RuntimePluginId, RuntimePluginAvailabilityDescriptorRef<'descriptor>>,
    linked_plugin_ids: RuntimePluginProviderMembership<'provider>,
    native_dynamic_plugin_ids: RuntimePluginProviderMembership<'provider>,
    #[cfg(test)]
    metrics: RuntimePluginAvailabilityProjectionMetrics,
}

enum RuntimePluginProviderMembership<'a> {
    Owned(HashSet<String>),
    BorrowedSet(&'a HashSet<String>),
    BorrowedIndex(HashSet<&'a str>),
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginAvailabilityProjectionMetrics {
    pub descriptor_rows: usize,
    pub linked_provider_rows: usize,
    pub native_dynamic_provider_rows: usize,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginAvailabilitySelectionMetrics {
    pub manifest_selection_rows: usize,
    pub indexed_lookup_rows: usize,
    pub unique_plugin_rows: usize,
    pub duplicate_merge_rows: usize,
}

#[cfg(test)]
impl RuntimePluginAvailabilitySelectionMetrics {
    pub fn selection_build_steps(self) -> usize {
        self.indexed_lookup_rows
    }
}

#[cfg(test)]
impl RuntimePluginAvailabilityProjectionMetrics {
    pub fn membership_build_steps(self) -> usize {
        self.descriptor_rows
            .saturating_add(self.linked_provider_rows)
            .saturating_add(self.native_dynamic_provider_rows)
    }
}

impl RuntimePluginProviderMembership<'_> {
    fn contains(&self, package_id: &str) -> bool {
        match self {
            Self::Owned(ids) => ids.contains(package_id),
            Self::BorrowedSet(ids) => ids.contains(package_id),
            Self::BorrowedIndex(ids) => ids.contains(package_id),
        }
    }
}

impl<'descriptor> RuntimePluginAvailabilityProjection<'descriptor, 'static> {
    pub fn new(
        descriptors: impl IntoIterator<Item = &'descriptor RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Self {
        #[cfg(test)]
        let mut descriptor_rows = 0;
        #[cfg(test)]
        let descriptors = descriptors
            .into_iter()
            .inspect(|_| descriptor_rows += 1)
            .map(RuntimePluginAvailabilityDescriptorRef::from_descriptor)
            .map(|descriptor| (descriptor.runtime_id.clone(), descriptor))
            .collect();
        #[cfg(not(test))]
        let descriptors = descriptors
            .into_iter()
            .map(RuntimePluginAvailabilityDescriptorRef::from_descriptor)
            .map(|descriptor| (descriptor.runtime_id.clone(), descriptor))
            .collect();
        #[cfg(test)]
        let (linked_plugin_ids, linked_provider_rows) = collect_provider_ids(linked_plugin_ids);
        #[cfg(not(test))]
        let linked_plugin_ids = collect_provider_ids(linked_plugin_ids);
        #[cfg(test)]
        let (native_dynamic_plugin_ids, native_dynamic_provider_rows) =
            collect_provider_ids(native_dynamic_plugin_ids);
        #[cfg(not(test))]
        let native_dynamic_plugin_ids = collect_provider_ids(native_dynamic_plugin_ids);
        Self {
            descriptors,
            linked_plugin_ids: RuntimePluginProviderMembership::Owned(linked_plugin_ids),
            native_dynamic_plugin_ids: RuntimePluginProviderMembership::Owned(
                native_dynamic_plugin_ids,
            ),
            #[cfg(test)]
            metrics: RuntimePluginAvailabilityProjectionMetrics {
                descriptor_rows,
                linked_provider_rows,
                native_dynamic_provider_rows,
            },
        }
    }
}

impl<'descriptor, 'provider> RuntimePluginAvailabilityProjection<'descriptor, 'provider> {
    pub fn from_descriptors_with_provider_membership(
        descriptors: impl IntoIterator<Item = &'descriptor RuntimePluginDescriptor>,
        linked_plugin_ids: &'provider HashSet<String>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = &'provider str>,
    ) -> Self {
        #[cfg(test)]
        let mut descriptor_rows = 0;
        #[cfg(test)]
        let descriptors = descriptors
            .into_iter()
            .inspect(|_| descriptor_rows += 1)
            .map(RuntimePluginAvailabilityDescriptorRef::from_descriptor)
            .map(|descriptor| (descriptor.runtime_id.clone(), descriptor))
            .collect();
        #[cfg(not(test))]
        let descriptors = descriptors
            .into_iter()
            .map(RuntimePluginAvailabilityDescriptorRef::from_descriptor)
            .map(|descriptor| (descriptor.runtime_id.clone(), descriptor))
            .collect();
        #[cfg(test)]
        let linked_provider_rows = linked_plugin_ids.len();
        #[cfg(test)]
        let mut native_dynamic_provider_rows = 0;
        #[cfg(test)]
        let native_dynamic_plugin_ids = native_dynamic_plugin_ids
            .into_iter()
            .inspect(|_| native_dynamic_provider_rows += 1)
            .collect();
        #[cfg(not(test))]
        let native_dynamic_plugin_ids = native_dynamic_plugin_ids.into_iter().collect();
        Self {
            descriptors,
            linked_plugin_ids: RuntimePluginProviderMembership::BorrowedSet(linked_plugin_ids),
            native_dynamic_plugin_ids: RuntimePluginProviderMembership::BorrowedIndex(
                native_dynamic_plugin_ids,
            ),
            #[cfg(test)]
            metrics: RuntimePluginAvailabilityProjectionMetrics {
                descriptor_rows,
                linked_provider_rows,
                native_dynamic_provider_rows,
            },
        }
    }

    pub fn from_catalog_with_provider_membership(
        catalog: &'descriptor RuntimePluginCatalog,
        linked_plugin_ids: &'provider HashSet<String>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = &'provider str>,
    ) -> Self {
        #[cfg(test)]
        let mut descriptor_rows = 0;
        #[cfg(test)]
        let descriptors = catalog
            .registrations()
            .iter()
            .filter_map(RuntimePluginAvailabilityDescriptorRef::from_registration)
            .inspect(|_| descriptor_rows += 1)
            .map(|descriptor| (descriptor.runtime_id.clone(), descriptor))
            .collect();
        #[cfg(not(test))]
        let descriptors = catalog
            .registrations()
            .iter()
            .filter_map(RuntimePluginAvailabilityDescriptorRef::from_registration)
            .map(|descriptor| (descriptor.runtime_id.clone(), descriptor))
            .collect();
        #[cfg(test)]
        let linked_provider_rows = linked_plugin_ids.len();
        #[cfg(test)]
        let mut native_dynamic_provider_rows = 0;
        #[cfg(test)]
        let native_dynamic_plugin_ids = native_dynamic_plugin_ids
            .into_iter()
            .inspect(|_| native_dynamic_provider_rows += 1)
            .collect();
        #[cfg(not(test))]
        let native_dynamic_plugin_ids = native_dynamic_plugin_ids.into_iter().collect();
        Self {
            descriptors,
            linked_plugin_ids: RuntimePluginProviderMembership::BorrowedSet(linked_plugin_ids),
            native_dynamic_plugin_ids: RuntimePluginProviderMembership::BorrowedIndex(
                native_dynamic_plugin_ids,
            ),
            #[cfg(test)]
            metrics: RuntimePluginAvailabilityProjectionMetrics {
                descriptor_rows,
                linked_provider_rows,
                native_dynamic_provider_rows,
            },
        }
    }

    pub fn from_registration_reports(
        descriptors: impl IntoIterator<Item = &'descriptor RuntimePluginDescriptor>,
        registrations: impl IntoIterator<Item = &'provider RuntimePluginRegistrationReport>,
        target: RuntimeTargetMode,
    ) -> Self {
        #[cfg(test)]
        let mut descriptor_rows = 0;
        #[cfg(test)]
        let descriptors = descriptors
            .into_iter()
            .inspect(|_| descriptor_rows += 1)
            .map(RuntimePluginAvailabilityDescriptorRef::from_descriptor)
            .map(|descriptor| (descriptor.runtime_id.clone(), descriptor))
            .collect();
        #[cfg(not(test))]
        let descriptors = descriptors
            .into_iter()
            .map(RuntimePluginAvailabilityDescriptorRef::from_descriptor)
            .map(|descriptor| (descriptor.runtime_id.clone(), descriptor))
            .collect();
        let mut linked_plugin_ids = HashSet::<&str>::new();
        let mut native_dynamic_plugin_ids = HashSet::<&str>::new();
        #[cfg(test)]
        let mut linked_provider_rows = 0;
        #[cfg(test)]
        let mut native_dynamic_provider_rows = 0;
        for registration in registrations {
            if !registration.project_selection.enabled
                || !registration.project_selection.supports_target(target)
            {
                continue;
            }
            if registration.project_selection.packaging == ExportPackagingStrategy::NativeDynamic {
                #[cfg(test)]
                {
                    native_dynamic_provider_rows += 1;
                }
                native_dynamic_plugin_ids.insert(registration.package_manifest.id.as_str());
            } else {
                #[cfg(test)]
                {
                    linked_provider_rows += 1;
                }
                linked_plugin_ids.insert(registration.package_manifest.id.as_str());
            }
        }
        Self {
            descriptors,
            linked_plugin_ids: RuntimePluginProviderMembership::BorrowedIndex(linked_plugin_ids),
            native_dynamic_plugin_ids: RuntimePluginProviderMembership::BorrowedIndex(
                native_dynamic_plugin_ids,
            ),
            #[cfg(test)]
            metrics: RuntimePluginAvailabilityProjectionMetrics {
                descriptor_rows,
                linked_provider_rows,
                native_dynamic_provider_rows,
            },
        }
    }

    #[cfg(test)]
    pub fn metrics(&self) -> RuntimePluginAvailabilityProjectionMetrics {
        self.metrics
    }

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

#[cfg(test)]
fn collect_provider_ids(
    ids: impl IntoIterator<Item = impl AsRef<str>>,
) -> (HashSet<String>, usize) {
    let mut rows = 0;
    let ids = ids
        .into_iter()
        .inspect(|_| rows += 1)
        .map(|id| id.as_ref().to_string())
        .collect();
    (ids, rows)
}

#[cfg(not(test))]
fn collect_provider_ids(ids: impl IntoIterator<Item = impl AsRef<str>>) -> HashSet<String> {
    ids.into_iter().map(|id| id.as_ref().to_string()).collect()
}

struct RuntimePluginManifestSelectionProjection {
    plugins: Vec<(RuntimePluginId, bool)>,
    #[cfg(test)]
    metrics: RuntimePluginAvailabilitySelectionMetrics,
}

fn project_manifest_plugin_selections(
    profile: &RuntimeProfileDescriptor,
    manifest: &ProjectPluginManifest,
) -> RuntimePluginManifestSelectionProjection {
    let mut plugins = Vec::<(RuntimePluginId, bool)>::new();
    let mut positions = HashMap::<RuntimePluginId, usize>::new();
    #[cfg(test)]
    let mut metrics = RuntimePluginAvailabilitySelectionMetrics::default();
    for selection in manifest.enabled_for_target(profile.target_mode) {
        #[cfg(test)]
        {
            metrics.manifest_selection_rows += 1;
        }
        let Some(runtime_id) = RuntimePluginId::parse_key(&selection.id) else {
            continue;
        };
        #[cfg(test)]
        {
            metrics.indexed_lookup_rows += 1;
        }
        if merge_runtime_plugin_selection(
            &mut plugins,
            &mut positions,
            runtime_id,
            selection.required,
        ) {
            #[cfg(test)]
            {
                metrics.duplicate_merge_rows += 1;
            }
        } else {
            #[cfg(test)]
            {
                metrics.unique_plugin_rows += 1;
            }
        }
    }
    RuntimePluginManifestSelectionProjection {
        plugins,
        #[cfg(test)]
        metrics,
    }
}

/// Preserves the first selection position while merging required state across every occurrence.
/// Both profile-default and manifest entry points use this one operation so their availability
/// generation and indexed lookup semantics cannot drift.
fn merge_runtime_plugin_selection(
    plugins: &mut Vec<(RuntimePluginId, bool)>,
    positions: &mut HashMap<RuntimePluginId, usize>,
    runtime_id: RuntimePluginId,
    required: bool,
) -> bool {
    if let Some(index) = positions.get(&runtime_id).copied() {
        plugins[index].1 = plugins[index].1 || required;
        true
    } else {
        positions.insert(runtime_id.clone(), plugins.len());
        plugins.push((runtime_id, required));
        false
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
