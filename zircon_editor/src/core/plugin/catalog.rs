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
    registration_index: BTreeMap<String, usize>,
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
            .field("registration_index", &self.registration_index)
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
        let registration_index = self.registrations.len();
        self.registration_index
            .entry(report.package_manifest.id.clone())
            .or_insert(registration_index);
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
        self.rebuild_registration_index();
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
        let Some(registration_index) = self.registration_index.get(package_id).copied() else {
            let mut report = EditorPluginLifecycleReport::default();
            let diagnostic = format!("editor plugin `{package_id}` is not registered");
            report.push_diagnostic(diagnostic.clone());
            self.diagnostics.push(diagnostic);
            return report;
        };
        let plugin = self.lifecycle_plugins.get(package_id).cloned();
        let registration = self
            .registrations
            .get_mut(registration_index)
            .expect("registration index must reference the current catalog generation");
        if let Some(plugin) = plugin {
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
        self.registration_for_package(package_id)
            .is_some_and(|registration| registration.lifecycle_stage_succeeded(stage))
    }

    pub(crate) fn lifecycle_stage_failed(
        &self,
        package_id: &str,
        stage: &super::sdk::lifecycle::EditorPluginLifecycleStage,
    ) -> bool {
        self.registration_for_package(package_id)
            .is_some_and(|registration| registration.lifecycle_stage_failed(stage))
    }

    pub(crate) fn is_package_faulted(&self, package_id: &str) -> bool {
        self.registration_for_package(package_id)
            .is_some_and(|registration| !registration.is_success())
    }

    fn registration_for_package(
        &self,
        package_id: &str,
    ) -> Option<&EditorPluginRegistrationReport> {
        self.registration_index
            .get(package_id)
            .and_then(|index| self.registrations.get(*index))
    }

    fn rebuild_registration_index(&mut self) {
        self.registration_index.clear();
        for (index, registration) in self.registrations.iter().enumerate() {
            self.registration_index
                .entry(registration.package_manifest.id.clone())
                .or_insert(index);
        }
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
        let diagnostic_capacity = self
            .registrations
            .iter()
            .map(|registration| registration.capabilities.len())
            .sum();
        let mut diagnostics = Vec::with_capacity(diagnostic_capacity);
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
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::Instant;

    use super::EditorPluginCatalog;
    use crate::core::plugin::descriptor::EditorPluginDescriptor;
    use crate::core::plugin::registration::EditorPluginRegistrationReport;
    use crate::core::plugin::sdk::lifecycle::{
        EditorPluginLifecycleEvent, EditorPluginLifecycleStage,
    };

    #[test]
    fn descriptor_catalog_indexes_runtime_manifests_once() {
        let source = include_str!("catalog.rs");
        let linear_lookup = [".find(|manifest| manifest.id == descriptor.", "package_id)"].concat();

        assert!(source.contains("runtime_manifest_by_package"));
        assert!(!source.contains(&linear_lookup));
    }

    #[test]
    fn registration_index_rebuilds_after_project_report_replacement() {
        let mut catalog = EditorPluginCatalog::from_descriptors(
            [
                EditorPluginDescriptor::new("plugin.builtin", "Builtin", "builtin"),
                EditorPluginDescriptor::new("plugin.project", "Project", "project"),
            ],
            std::iter::empty(),
        );
        let replacement = EditorPluginDescriptor::new("plugin.project", "Project v2", "project_v2");
        let replacement_report = EditorPluginRegistrationReport::from_plugin(
            &replacement,
            replacement.standalone_package_manifest(),
        );

        catalog.replace_project_registration_reports(
            &BTreeSet::from(["plugin.project".to_string()]),
            [replacement_report],
        );

        assert_eq!(catalog.registration_index.len(), 2);
        let report = catalog.record_lifecycle_event(
            "plugin.project",
            EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Loaded),
        );
        assert!(report.is_success());
        assert!(
            catalog
                .lifecycle_stage_succeeded("plugin.project", &EditorPluginLifecycleStage::Loaded)
        );
    }

    #[test]
    #[ignore = "managed release performance evidence"]
    fn optimization_wave_20260825_editor06_registration_index_evidence() {
        const PLUGINS: usize = 1_000;
        const LOOKUPS: usize = 100_000;
        const MAX_ELAPSED_NS: u128 = 3_000_000_000;

        let descriptors = (0..PLUGINS).map(|index| {
            let package_id = format!("plugin.bench.{index:04}");
            EditorPluginDescriptor::new(&package_id, &package_id, package_id.clone())
        });
        let catalog = EditorPluginCatalog::from_descriptors(descriptors, std::iter::empty());
        let target = format!("plugin.bench.{:04}", PLUGINS - 1);
        let stage = EditorPluginLifecycleStage::Loaded;
        let started = Instant::now();
        for _ in 0..LOOKUPS {
            black_box(catalog.lifecycle_stage_succeeded(&target, &stage));
        }
        let elapsed_ns = started.elapsed().as_nanos();
        let legacy_candidate_checks = PLUGINS * LOOKUPS;
        let indexed_registration_probes = LOOKUPS;
        let probe_reduction_bps = legacy_candidate_checks
            .saturating_sub(indexed_registration_probes)
            .saturating_mul(10_000)
            / legacy_candidate_checks;

        println!(
            "EDITOR06_PLUGIN_REGISTRATION_INDEX_BENCH_V1 plugins={PLUGINS} lookups={LOOKUPS} legacy_candidate_checks={legacy_candidate_checks} indexed_registration_probes={indexed_registration_probes} probe_reduction_bps={probe_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
        );

        assert_eq!(catalog.registration_index.len(), PLUGINS);
        assert_eq!(probe_reduction_bps, 9_990);
        assert!(elapsed_ns <= MAX_ELAPSED_NS);
    }
}

#[cfg(test)]
mod optimization_batch_20260830cm_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const CAPABILITIES_PER_SAMPLE: usize = 16_384;

    #[test]
    fn optimization_batch_20260830cm_editor_capability_diagnostics_reserve_upper_bound() {
        let source = include_str!("catalog.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("editor plugin catalog implementation");

        assert!(implementation.contains("let diagnostic_capacity = self"));
        assert!(implementation.contains(".map(|registration| registration.capabilities.len())"));
        assert!(implementation.contains("Vec::with_capacity(diagnostic_capacity)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cm_editor_capability_diagnostic_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR500_CAPABILITY_DIAGNOSTIC_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} capabilities_per_sample={CAPABILITIES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut diagnostics = if use_capacity {
            Vec::with_capacity(CAPABILITIES_PER_SAMPLE)
        } else {
            Vec::new()
        };
        for capability in 0..CAPABILITIES_PER_SAMPLE {
            diagnostics.push(capability);
        }
        std::hint::black_box(diagnostics);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
