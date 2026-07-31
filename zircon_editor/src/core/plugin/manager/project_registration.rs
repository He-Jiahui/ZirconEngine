//! Project-scoped native editor registration publication.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::core::plugin::EditorPluginRegistrationReport;

use super::super::admission::validate_catalog_admission;
use super::{
    EditorPluginCatalogSnapshot, EditorPluginDiscovery, EditorPluginDiscoveryError,
    EditorPluginManager, EditorPluginSource, discovery_index,
};

impl EditorPluginManager {
    /// Replaces only the registrations discovered for the currently opened project.
    ///
    /// The candidate remains under the lifecycle mutation lock through discovery validation and
    /// publication, so panels never observe a catalog built from a stale project snapshot.
    pub(crate) fn publish_project_registration_reports(
        &self,
        reports: impl IntoIterator<Item = EditorPluginRegistrationReport>,
    ) -> Result<Arc<EditorPluginCatalogSnapshot>, EditorPluginDiscoveryError> {
        let reports = reports.into_iter().collect::<Vec<_>>();
        let _mutation = self
            .lifecycle_mutation
            .try_lock()
            .map_err(|_| EditorPluginDiscoveryError::MutationInProgress)?;
        let previous = self.state_snapshot();
        let project_package_ids = previous
            .entries()
            .iter()
            .filter(|entry| entry.source() == EditorPluginSource::Project)
            .map(|entry| entry.package_id().to_string())
            .collect::<BTreeSet<_>>();
        if reports.is_empty() && project_package_ids.is_empty() {
            return Ok(Arc::clone(previous.catalog_snapshot()));
        }

        let discoveries = reports
            .iter()
            .map(|report| EditorPluginDiscovery::project(report.package_manifest.id.clone()))
            .collect::<Vec<_>>();
        let mut catalog = previous.catalog_snapshot().clone_catalog();
        catalog.replace_project_registration_reports(&project_package_ids, reports);
        validate_catalog_admission(&catalog)?;
        let discoveries = discovery_index(&catalog, discoveries)?;
        self.publish_catalog_with_indexed_discoveries(catalog, discoveries)
    }

    /// Clears native registrations from a project that has just been closed or rolled back.
    pub(crate) fn clear_project_registration_reports(
        &self,
    ) -> Result<Arc<EditorPluginCatalogSnapshot>, EditorPluginDiscoveryError> {
        self.publish_project_registration_reports(std::iter::empty())
    }
}
