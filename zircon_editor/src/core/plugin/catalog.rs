//! Mutable catalog owner that publishes immutable extension materializations.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::sync::Arc;

use zircon_runtime::plugin::PluginPackageManifest;
use zircon_runtime_interface::RegistrationDiagnostic;

use super::capability_report::EditorCapabilityReport;
use super::descriptor::{EditorPlugin, EditorPluginDescriptor};
use super::registration::EditorPluginRegistrationReport;
use super::sdk::lifecycle::{EditorPluginLifecycleEvent, EditorPluginLifecycleReport};

/// A manager-owned plugin instance that remains valid through lifecycle dispatch.
pub type EditorPluginHandle = Arc<dyn EditorPlugin + Send + Sync>;

#[derive(Clone, Default)]
pub(crate) struct EditorPluginCatalog {
    pub(super) registrations: Vec<EditorPluginRegistrationReport>,
    diagnostics: Vec<String>,
    pub(super) generation: u64,
    admission_duplicate_package_ids: BTreeSet<String>,
    lifecycle_plugins: BTreeMap<String, EditorPluginHandle>,
}

impl fmt::Debug for EditorPluginCatalog {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorPluginCatalog")
            .field("registrations", &self.registrations)
            .field("diagnostics", &self.diagnostics)
            .field("generation", &self.generation)
            .field(
                "admission_duplicate_package_ids",
                &self.admission_duplicate_package_ids,
            )
            .field("lifecycle_plugin_ids", &self.lifecycle_plugins.keys())
            .finish()
    }
}

impl EditorPluginCatalog {
    pub(crate) fn from_plugins(
        plugins: impl IntoIterator<Item = (EditorPluginHandle, PluginPackageManifest)>,
    ) -> Self {
        let mut catalog = Self::default();
        for (plugin, runtime_manifest) in plugins {
            catalog.register(plugin, runtime_manifest);
        }
        catalog
    }

    pub(crate) fn from_descriptors(
        descriptors: impl IntoIterator<Item = EditorPluginDescriptor>,
        runtime_manifests: impl IntoIterator<Item = PluginPackageManifest>,
    ) -> Self {
        let descriptors = descriptors.into_iter().collect::<Vec<_>>();
        let editor_package_ids = descriptors
            .iter()
            .map(|descriptor| descriptor.package_id.as_str())
            .collect::<BTreeSet<_>>();
        let runtime_manifests = runtime_manifests.into_iter().collect::<Vec<_>>();
        let mut runtime_manifest_by_package = HashMap::with_capacity(runtime_manifests.len());
        let mut admission_duplicate_package_ids = BTreeSet::new();
        for manifest in &runtime_manifests {
            if editor_package_ids.contains(manifest.id.as_str())
                && runtime_manifest_by_package.contains_key(manifest.id.as_str())
            {
                admission_duplicate_package_ids.insert(manifest.id.clone());
                continue;
            }
            runtime_manifest_by_package
                .entry(manifest.id.as_str())
                .or_insert(manifest);
        }
        let mut catalog = Self {
            admission_duplicate_package_ids,
            ..Self::default()
        };
        for descriptor in descriptors {
            let plugin: EditorPluginHandle = Arc::new(descriptor);
            let runtime_manifest = runtime_manifest_by_package
                .get(plugin.descriptor().package_id.as_str())
                .copied()
                .cloned()
                .unwrap_or_else(|| plugin.descriptor().standalone_package_manifest());
            catalog.register(plugin, runtime_manifest);
        }
        catalog
    }

    pub(crate) fn builtin(
        runtime_manifests: impl IntoIterator<Item = PluginPackageManifest>,
    ) -> Self {
        Self::from_descriptors(EditorPluginDescriptor::builtin_catalog(), runtime_manifests)
    }

    pub(crate) fn register(
        &mut self,
        plugin: EditorPluginHandle,
        runtime_manifest: PluginPackageManifest,
    ) {
        let package_id = plugin.descriptor().package_id.clone();
        let report = EditorPluginRegistrationReport::from_plugin(plugin.as_ref(), runtime_manifest);
        self.diagnostics.extend(report.diagnostics.iter().cloned());
        self.registrations.push(report);
        self.lifecycle_plugins.insert(package_id, plugin);
        self.generation = self.generation.saturating_add(1);
    }

    /// Replaces the catalog rows that are scoped to one opened project.
    pub(super) fn replace_project_registration_reports(
        &mut self,
        project_package_ids: &BTreeSet<String>,
        reports: impl IntoIterator<Item = EditorPluginRegistrationReport>,
    ) {
        self.registrations.retain(|registration| {
            !project_package_ids.contains(&registration.package_manifest.id)
        });
        self.lifecycle_plugins
            .retain(|package_id, _| !project_package_ids.contains(package_id));
        self.registrations.extend(reports);
        self.diagnostics = self
            .registrations
            .iter()
            .flat_map(|registration| registration.diagnostics.iter().cloned())
            .collect();
        self.generation = self.generation.saturating_add(1);
    }

    pub(super) fn record_lifecycle_event(
        &mut self,
        package_id: &str,
        event: EditorPluginLifecycleEvent,
    ) -> EditorPluginLifecycleReport {
        let Some(registration) = self
            .registrations
            .iter_mut()
            .find(|registration| registration.package_manifest.id == package_id)
        else {
            let mut report = EditorPluginLifecycleReport::default();
            let diagnostic = format!("editor plugin `{package_id}` is not registered");
            report.push_diagnostic(diagnostic.clone());
            self.diagnostics.push(diagnostic);
            return report;
        };
        if let Some(plugin) = self.lifecycle_plugins.get(package_id).cloned() {
            registration.record_lifecycle_event(plugin.as_ref(), event)
        } else {
            registration.record_host_lifecycle_event(event)
        }
    }

    pub(crate) fn registrations(&self) -> &[EditorPluginRegistrationReport] {
        &self.registrations
    }

    pub(super) fn admission_duplicate_package_ids(&self) -> &BTreeSet<String> {
        &self.admission_duplicate_package_ids
    }

    pub(crate) fn lifecycle_stage_succeeded(
        &self,
        package_id: &str,
        stage: &super::sdk::lifecycle::EditorPluginLifecycleStage,
    ) -> bool {
        self.registrations
            .iter()
            .find(|registration| registration.package_manifest.id == package_id)
            .is_some_and(|registration| registration.lifecycle_stage_succeeded(stage))
    }

    pub(crate) fn lifecycle_stage_failed(
        &self,
        package_id: &str,
        stage: &super::sdk::lifecycle::EditorPluginLifecycleStage,
    ) -> bool {
        self.registrations
            .iter()
            .find(|registration| registration.package_manifest.id == package_id)
            .is_some_and(|registration| registration.lifecycle_stage_failed(stage))
    }

    pub(crate) fn is_package_faulted(&self, package_id: &str) -> bool {
        self.registrations
            .iter()
            .find(|registration| registration.package_manifest.id == package_id)
            .is_some_and(|registration| !registration.is_success())
    }

    pub(crate) fn has_same_lifecycle_plugin(&self, other: &Self, package_id: &str) -> bool {
        match (
            self.lifecycle_plugins.get(package_id),
            other.lifecycle_plugins.get(package_id),
        ) {
            (Some(current), Some(previous)) => Arc::ptr_eq(current, previous),
            // A native host registration has no Rust object identity. Treat a republished row
            // as a new host-owned lifecycle instance so its Loaded/Enabled records remain paired
            // with the catalog generation that exposes it.
            (None, None) => false,
            _ => false,
        }
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn package_manifests(&self) -> Vec<PluginPackageManifest> {
        self.registrations
            .iter()
            .map(|registration| registration.package_manifest.clone())
            .collect()
    }

    pub(crate) fn capabilities(&self) -> Vec<String> {
        let mut capabilities = self
            .registrations
            .iter()
            .flat_map(|registration| registration.capabilities.iter().cloned())
            .collect::<Vec<_>>();
        capabilities.sort();
        capabilities.dedup();
        capabilities
    }

    pub(crate) fn capabilities_for_package(&self, package_id: &str) -> Vec<String> {
        self.registrations
            .iter()
            .filter(|registration| registration.package_manifest.id == package_id)
            .flat_map(|registration| registration.capabilities.iter().cloned())
            .collect()
    }

    pub(crate) fn validate_capabilities<I, S>(
        &self,
        enabled_capabilities: I,
    ) -> EditorCapabilityReport
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let enabled_capabilities = enabled_capabilities
            .into_iter()
            .map(|capability| capability.as_ref().to_string())
            .collect::<BTreeSet<_>>();
        let mut diagnostics = Vec::new();
        for registration in &self.registrations {
            for capability in &registration.capabilities {
                if !enabled_capabilities.contains(capability) {
                    diagnostics.push(RegistrationDiagnostic::missing_capability(
                        registration.package_manifest.id.clone(),
                        capability.clone(),
                    ));
                }
            }
        }
        EditorCapabilityReport { diagnostics }
    }

    pub(crate) fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub(crate) fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn descriptor_catalog_indexes_runtime_manifests_once() {
        let source = include_str!("catalog.rs");
        let linear_lookup = [".find(|manifest| manifest.id == descriptor.", "package_id)"].concat();

        assert!(source.contains("runtime_manifest_by_package"));
        assert!(!source.contains(&linear_lookup));
    }
}
