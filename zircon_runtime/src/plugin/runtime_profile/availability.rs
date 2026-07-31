use std::collections::HashSet;

use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

use super::availability_projection::{
    RuntimePluginAvailabilityGeneration, RuntimePluginAvailabilityProjection,
};
#[cfg(test)]
use super::availability_projection::{
    RuntimePluginAvailabilityProjectionMetrics, RuntimePluginAvailabilitySelectionMetrics,
};
use super::availability_report::RuntimePluginAvailabilityReport;
use super::descriptor::RuntimeProfileDescriptor;

impl RuntimeProfileDescriptor {
    pub fn availability_report<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            std::iter::empty::<String>(),
        )
        .report_for_profile_defaults(self, false)
    }

    pub fn availability_report_with_providers<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .report_for_profile_defaults(self, true)
    }

    pub fn availability_report_for_registration_reports<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        registrations: impl IntoIterator<Item = &'b RuntimePluginRegistrationReport>,
    ) -> RuntimePluginAvailabilityReport {
        self.availability_report_for_manifest_and_registration_reports(
            descriptors,
            &self.project_manifest(),
            registrations,
        )
    }

    pub fn availability_report_for_manifest_and_registration_reports<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        registrations: impl IntoIterator<Item = &'b RuntimePluginRegistrationReport>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::from_registration_reports(
            descriptors,
            registrations,
            self.target_mode,
        )
        .report_for_manifest(self, manifest, true)
    }

    pub fn availability_report_for_manifest_with_providers<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .report_for_manifest(self, manifest, true)
    }

    /// Builds one immutable availability generation for consumers that poll or
    /// render status repeatedly. Materialize a report only at an export or
    /// diagnostic boundary that needs owned, serializable rows.
    pub fn availability_generation_for_manifest_with_providers<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityGeneration<'a> {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .generation_for_manifest(self, manifest, true)
    }

    pub(crate) fn availability_report_for_manifest_with_linked_membership<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: &'b HashSet<String>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::from_descriptors_with_provider_membership(
            descriptors,
            linked_plugin_ids,
            std::iter::empty::<&str>(),
        )
        .report_for_manifest(self, manifest, true)
    }

    pub(crate) fn availability_report_for_catalog_with_provider_membership<'a, 'b>(
        &self,
        catalog: &'a RuntimePluginCatalog,
        linked_plugin_ids: &'b HashSet<String>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = &'b str>,
    ) -> RuntimePluginAvailabilityReport {
        RuntimePluginAvailabilityProjection::from_catalog_with_provider_membership(
            catalog,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .report_for_profile_defaults(self, true)
    }

    #[cfg(test)]
    pub(crate) fn availability_report_for_manifest_with_providers_and_metrics<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> (
        RuntimePluginAvailabilityReport,
        RuntimePluginAvailabilitySelectionMetrics,
    ) {
        RuntimePluginAvailabilityProjection::new(
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
        )
        .report_for_manifest_with_metrics(self, manifest, true)
    }

    #[cfg(test)]
    pub(crate) fn availability_report_for_manifest_with_linked_membership_and_metrics<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: &'b HashSet<String>,
    ) -> (
        RuntimePluginAvailabilityReport,
        RuntimePluginAvailabilityProjectionMetrics,
    ) {
        let projection =
            RuntimePluginAvailabilityProjection::from_descriptors_with_provider_membership(
                descriptors,
                linked_plugin_ids,
                std::iter::empty::<&str>(),
            );
        let report = projection.report_for_manifest(self, manifest, true);
        (report, projection.metrics())
    }

    #[cfg(test)]
    pub(crate) fn availability_report_for_manifest_and_registration_reports_with_metrics<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        registrations: impl IntoIterator<Item = &'b RuntimePluginRegistrationReport>,
    ) -> (
        RuntimePluginAvailabilityReport,
        RuntimePluginAvailabilityProjectionMetrics,
    ) {
        let projection = RuntimePluginAvailabilityProjection::from_registration_reports(
            descriptors,
            registrations,
            self.target_mode,
        );
        let report = projection.report_for_manifest(self, manifest, true);
        (report, projection.metrics())
    }
}
