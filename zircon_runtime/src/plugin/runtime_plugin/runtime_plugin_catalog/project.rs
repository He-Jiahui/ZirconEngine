use std::sync::{atomic::Ordering, Arc};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginManifest, ProjectPluginSelection,
};

use super::extension_report::{
    runtime_extension_report_for_project, RuntimeExtensionCatalogReport,
};
use super::features::feature_dependency_report;
use super::project_manifest::{
    catalog_project_manifest, complete_project_manifest as complete_catalog_project_manifest,
};
use super::RuntimePluginCatalog;
use super::RuntimePluginFeatureDependencyReport;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginCatalogProjectPlanMetrics {
    pub catalog_generation: u64,
    pub project_plan_builds: u64,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledProjectPluginPlan {
    catalog_generation: u64,
    source_fingerprint: u64,
    source_manifest: ProjectPluginManifest,
    completed_manifest: Arc<ProjectPluginManifest>,
    feature_report: Arc<RuntimePluginFeatureDependencyReport>,
    extension_report: Arc<RuntimeExtensionCatalogReport>,
}

impl CompiledProjectPluginPlan {
    fn matches(
        &self,
        catalog_generation: u64,
        fingerprint: u64,
        manifest: &ProjectPluginManifest,
    ) -> bool {
        self.catalog_generation == catalog_generation
            && self.source_fingerprint == fingerprint
            && &self.source_manifest == manifest
    }

    pub(super) fn completed_manifest(&self) -> &Arc<ProjectPluginManifest> {
        &self.completed_manifest
    }

    pub(super) fn feature_report(&self) -> &Arc<RuntimePluginFeatureDependencyReport> {
        &self.feature_report
    }

    pub(super) fn extension_report(&self) -> &Arc<RuntimeExtensionCatalogReport> {
        &self.extension_report
    }
}

impl RuntimePluginCatalog {
    pub fn project_manifest(&self) -> ProjectPluginManifest {
        catalog_project_manifest(&self.registrations, &self.projection)
    }

    pub fn complete_project_manifest(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Arc<ProjectPluginManifest> {
        Arc::clone(self.project_plan_for(manifest, target).completed_manifest())
    }

    pub fn project_selection_for_package(
        &self,
        package_id: &str,
    ) -> Option<ProjectPluginSelection> {
        self.projection
            .registration_index_for_package(package_id)
            .map(|index| self.registrations[index].project_selection.clone())
    }

    pub fn project_plan_metrics(&self) -> RuntimePluginCatalogProjectPlanMetrics {
        RuntimePluginCatalogProjectPlanMetrics {
            catalog_generation: self.catalog_generation,
            project_plan_builds: self.project_plan_builds.load(Ordering::Relaxed),
        }
    }

    #[cfg(test)]
    pub(crate) fn cached_project_plan_count(&self) -> usize {
        self.project_plans
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub(super) fn project_plan_for(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Arc<CompiledProjectPluginPlan> {
        let fingerprint = project_manifest_fingerprint(manifest);
        let target_key = project_plan_target_key(target);
        let mut project_plans = self
            .project_plans
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(plan) = project_plans.get(&target_key) {
            if plan.matches(self.catalog_generation, fingerprint, manifest) {
                return Arc::clone(plan);
            }
        }

        let completed_manifest = Arc::new(complete_catalog_project_manifest(
            &self.registrations,
            &self.projection,
            manifest,
        ));
        let feature_report = Arc::new(feature_dependency_report(
            &self.projection,
            &completed_manifest,
            target,
        ));
        let extension_report = Arc::new(runtime_extension_report_for_project(
            &self.registrations,
            &self.feature_registrations,
            &self.projection,
            &completed_manifest,
            target,
            &feature_report,
        ));
        let plan = Arc::new(CompiledProjectPluginPlan {
            catalog_generation: self.catalog_generation,
            source_fingerprint: fingerprint,
            source_manifest: manifest.clone(),
            completed_manifest,
            feature_report,
            extension_report,
        });
        project_plans.insert(target_key, Arc::clone(&plan));
        self.project_plan_builds.fetch_add(1, Ordering::Relaxed);
        plan
    }
}

fn project_manifest_fingerprint(manifest: &ProjectPluginManifest) -> u64 {
    let mut fingerprint = ManifestFingerprint::new();
    fingerprint.write_len(manifest.selections.len());
    for selection in &manifest.selections {
        fingerprint.write_str(&selection.id);
        fingerprint.write_bool(selection.enabled);
        fingerprint.write_bool(selection.required);
        fingerprint.write_target_modes(&selection.target_modes);
        fingerprint.write_packaging(selection.packaging);
        fingerprint.write_optional_str(selection.runtime_crate.as_deref());
        fingerprint.write_optional_str(selection.editor_crate.as_deref());
        fingerprint.write_len(selection.features.len());
        for feature in &selection.features {
            fingerprint.write_str(&feature.id);
            fingerprint.write_bool(feature.enabled);
            fingerprint.write_bool(feature.required);
            fingerprint.write_target_modes(&feature.target_modes);
            fingerprint.write_packaging(feature.packaging);
            fingerprint.write_optional_str(feature.runtime_crate.as_deref());
            fingerprint.write_optional_str(feature.editor_crate.as_deref());
            fingerprint.write_optional_str(feature.provider_package_id.as_deref());
        }
    }
    fingerprint.finish()
}

struct ManifestFingerprint(u64);

impl ManifestFingerprint {
    const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    fn new() -> Self {
        Self(Self::OFFSET_BASIS)
    }

    fn finish(self) -> u64 {
        self.0
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn write_len(&mut self, value: usize) {
        self.write_bytes(&(value as u64).to_le_bytes());
    }

    fn write_str(&mut self, value: &str) {
        self.write_len(value.len());
        self.write_bytes(value.as_bytes());
    }

    fn write_bool(&mut self, value: bool) {
        self.write_bytes(&[u8::from(value)]);
    }

    fn write_optional_str(&mut self, value: Option<&str>) {
        self.write_bool(value.is_some());
        if let Some(value) = value {
            self.write_str(value);
        }
    }

    fn write_target_modes(&mut self, values: &[RuntimeTargetMode]) {
        self.write_len(values.len());
        for value in values {
            self.write_bytes(&[project_plan_target_key(*value)]);
        }
    }

    fn write_packaging(&mut self, value: ExportPackagingStrategy) {
        self.write_bytes(&[match value {
            ExportPackagingStrategy::SourceTemplate => 0,
            ExportPackagingStrategy::LibraryEmbed => 1,
            ExportPackagingStrategy::NativeDynamic => 2,
        }]);
    }
}

fn project_plan_target_key(target: RuntimeTargetMode) -> u8 {
    match target {
        RuntimeTargetMode::ClientRuntime => 0,
        RuntimeTargetMode::ServerRuntime => 1,
        RuntimeTargetMode::EditorHost => 2,
    }
}
