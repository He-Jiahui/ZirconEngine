use std::collections::{HashMap, HashSet};

use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ExportPackagingStrategy;
use crate::plugin::{
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

use super::RuntimeProfileDescriptor;

mod evaluation;
mod generation;
mod selection;

use generation::RuntimePluginAvailabilityDescriptorRef;
pub use generation::{
    RuntimePluginAvailabilityGeneration, RuntimePluginAvailabilityRow,
    RuntimePluginAvailabilitySummary,
};
#[cfg(test)]
pub(crate) use selection::RuntimePluginAvailabilitySelectionMetrics;

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
