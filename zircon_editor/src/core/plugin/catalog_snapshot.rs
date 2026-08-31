//! Immutable, indexed read model for one editor-plugin catalog generation.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime::plugin::PluginPackageManifest;

use super::capability_report::EditorCapabilityReport;
use super::catalog::EditorPluginCatalog;
use super::projection::EditorPluginCatalogProjection;
use super::registration::EditorPluginRegistrationReport;

#[derive(Clone, Debug)]
pub struct EditorPluginCatalogSnapshot {
    generation: u64,
    catalog: Arc<EditorPluginCatalog>,
    package_manifests: Vec<PluginPackageManifest>,
    package_index: BTreeMap<String, usize>,
    registration_index: BTreeMap<String, usize>,
    projection: Arc<EditorPluginCatalogProjection>,
    faulted_packages: BTreeSet<String>,
    capabilities: Vec<String>,
    capabilities_by_package: BTreeMap<String, Vec<String>>,
    packages_by_capability: BTreeMap<String, Vec<String>>,
}

impl EditorPluginCatalogSnapshot {
    pub(super) fn from_catalog(generation: u64, catalog: EditorPluginCatalog) -> Self {
        let catalog = Arc::new(catalog);
        let package_manifests = catalog.package_manifests();
        let package_index = package_manifests
            .iter()
            .enumerate()
            .map(|(index, package)| (package.id.clone(), index))
            .collect();
        let registration_index = catalog
            .registrations()
            .iter()
            .enumerate()
            .map(|(index, registration)| (registration.package_manifest.id.clone(), index))
            .collect();
        let projection = Arc::new(EditorPluginCatalogProjection::from_registrations(
            catalog.registrations(),
        ));
        let faulted_packages = catalog
            .registrations()
            .iter()
            .filter(|registration| !registration.is_success())
            .map(|registration| registration.package_manifest.id.clone())
            .collect();
        let capabilities = catalog.capabilities();
        let mut capabilities_by_package = BTreeMap::new();
        let mut packages_by_capability = BTreeMap::<String, Vec<String>>::new();
        for registration in catalog.registrations() {
            let package_id = registration.package_manifest.id.clone();
            for capability in &registration.capabilities {
                packages_by_capability
                    .entry(capability.clone())
                    .or_default()
                    .push(package_id.clone());
            }
            capabilities_by_package.insert(package_id, registration.capabilities.clone());
        }
        for package_ids in packages_by_capability.values_mut() {
            package_ids.sort_unstable();
            package_ids.dedup();
        }
        Self {
            generation,
            catalog,
            package_manifests,
            package_index,
            registration_index,
            projection,
            faulted_packages,
            capabilities,
            capabilities_by_package,
            packages_by_capability,
        }
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn package_manifests(&self) -> &[PluginPackageManifest] {
        &self.package_manifests
    }

    pub fn package(&self, package_id: &str) -> Option<&PluginPackageManifest> {
        self.package_index
            .get(package_id)
            .and_then(|index| self.package_manifests.get(*index))
    }

    /// Returns one registration report without rebuilding a panel projection or cloning diagnostics.
    pub fn registration(&self, package_id: &str) -> Option<&EditorPluginRegistrationReport> {
        self.registration_index
            .get(package_id)
            .and_then(|index| self.catalog.registrations().get(*index))
    }

    /// Manager-only access preserves catalog registration order while materializing active phases.
    pub(crate) fn registrations(&self) -> &[EditorPluginRegistrationReport] {
        self.catalog.registrations()
    }

    /// Produces the manager's next catalog candidate without exposing mutable state to readers.
    pub(crate) fn clone_catalog(&self) -> EditorPluginCatalog {
        (*self.catalog).clone()
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    pub fn projection(&self) -> &Arc<EditorPluginCatalogProjection> {
        &self.projection
    }

    pub fn capabilities_for_package(&self, package_id: &str) -> &[String] {
        self.capabilities_by_package
            .get(package_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn packages_for_capability(&self, capability: &str) -> &[String] {
        self.packages_by_capability
            .get(capability)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(crate) fn is_package_faulted(&self, package_id: &str) -> bool {
        self.faulted_packages.contains(package_id)
    }

    pub(crate) fn lifecycle_stage_failed(
        &self,
        package_id: &str,
        stage: &super::sdk::lifecycle::EditorPluginLifecycleStage,
    ) -> bool {
        self.catalog.lifecycle_stage_failed(package_id, stage)
    }

    pub fn diagnostics(&self) -> &[String] {
        self.catalog.diagnostics()
    }

    pub fn validate_capabilities<I, S>(&self, enabled_capabilities: I) -> EditorCapabilityReport
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.catalog.validate_capabilities(enabled_capabilities)
    }
}

#[cfg(test)]
#[path = "catalog_snapshot/optimization_tests.rs"]
mod optimization_tests;
