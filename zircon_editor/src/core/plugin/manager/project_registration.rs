//! Project-scoped native editor registration publication.

use std::collections::BTreeSet;
use std::sync::Arc;

use crate::core::plugin::EditorPluginRegistrationReport;

use super::super::admission::validate_catalog_admission;
use super::discovery::discovery_index;
use super::{
    EditorPluginCatalogSnapshot, EditorPluginDiscovery, EditorPluginDiscoveryError,
    EditorPluginManager, EditorPluginSource,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProjectPluginRegistrationCloseReceipt {
    retired_package_ids: Vec<String>,
    remaining_project_package_ids: Vec<String>,
    previous_manager_generation: u64,
    previous_catalog_generation: u64,
    manager_generation: u64,
    catalog_generation: u64,
}

impl ProjectPluginRegistrationCloseReceipt {
    pub(crate) fn retired_package_ids(&self) -> &[String] {
        &self.retired_package_ids
    }

    pub(crate) fn remaining_project_package_ids(&self) -> &[String] {
        &self.remaining_project_package_ids
    }

    pub(crate) const fn previous_manager_generation(&self) -> u64 {
        self.previous_manager_generation
    }

    pub(crate) const fn previous_catalog_generation(&self) -> u64 {
        self.previous_catalog_generation
    }

    pub(crate) const fn manager_generation(&self) -> u64 {
        self.manager_generation
    }

    pub(crate) const fn catalog_generation(&self) -> u64 {
        self.catalog_generation
    }

    pub(crate) fn is_terminal(&self) -> bool {
        self.remaining_project_package_ids.is_empty()
    }
}

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
        self.publish_project_registration_reports_locked(reports)
    }

    fn publish_project_registration_reports_locked(
        &self,
        reports: Vec<EditorPluginRegistrationReport>,
    ) -> Result<Arc<EditorPluginCatalogSnapshot>, EditorPluginDiscoveryError> {
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
    ) -> Result<ProjectPluginRegistrationCloseReceipt, EditorPluginDiscoveryError> {
        let _mutation = self
            .lifecycle_mutation
            .try_lock()
            .map_err(|_| EditorPluginDiscoveryError::MutationInProgress)?;
        let previous = self.state_snapshot();
        let retired_package_ids = previous
            .entries()
            .iter()
            .filter(|entry| entry.source() == EditorPluginSource::Project)
            .map(|entry| entry.package_id().to_string())
            .collect::<Vec<_>>();
        self.publish_project_registration_reports_locked(Vec::new())?;
        let terminal = self.state_snapshot();
        let remaining_project_package_ids = terminal
            .entries()
            .iter()
            .filter(|entry| entry.source() == EditorPluginSource::Project)
            .map(|entry| entry.package_id().to_string())
            .collect();
        Ok(ProjectPluginRegistrationCloseReceipt {
            retired_package_ids,
            remaining_project_package_ids,
            previous_manager_generation: previous.generation(),
            previous_catalog_generation: previous.catalog_generation(),
            manager_generation: terminal.generation(),
            catalog_generation: terminal.catalog_generation(),
        })
    }
}
