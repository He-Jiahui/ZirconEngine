use std::sync::Arc;
use std::time::Instant;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{
    ProjectPluginManifest, ProjectPluginSelection, RuntimeProfileId,
};
use crate::plugin::{PluginMaturity, RuntimeProfileDescriptor};

#[path = "project/cache.rs"]
mod cache;
#[path = "project/fingerprint.rs"]
mod fingerprint;
#[path = "project/selection.rs"]
mod selection;

use super::extension_report::{
    runtime_extension_report_for_project, RuntimeExtensionCatalogReport,
};
use super::features::feature_dependency_report_for_effective_base;
use super::project_manifest::{
    catalog_project_manifest, complete_project_manifest as complete_catalog_project_manifest,
};
use super::RuntimePluginFeatureDependencyReport;
use super::{PluginCatalogGeneration, RuntimePluginCatalog};
pub(super) use cache::ProjectPlanCache;
use cache::ProjectPlanCacheEntry;
use fingerprint::project_manifest_fingerprint;
pub use selection::RuntimePluginModuleProposal;
pub(super) use selection::{CompiledRuntimePluginBaseSelection, CompiledRuntimePluginSelection};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginCatalogProjectPlanMetrics {
    pub catalog_generation: PluginCatalogGeneration,
    pub project_plan_builds: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct RuntimePluginCatalogProjectPlanCacheMetrics {
    pub catalog_generation: PluginCatalogGeneration,
    pub project_plan_builds: u64,
    pub total_build_elapsed_ns: u64,
    pub max_build_elapsed_ns: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_evictions: u64,
    pub cached_plan_count: usize,
}

/// Immutable plugin manifest, feature, and extension result for one catalog generation.
#[derive(Debug)]
pub struct CompiledProjectPluginPlan {
    catalog_generation: PluginCatalogGeneration,
    source_manifest_fingerprint: u64,
    target_mode: RuntimeTargetMode,
    runtime_profile_admission: Option<(RuntimeProfileId, PluginMaturity)>,
    pub(super) completed_manifest: Arc<ProjectPluginManifest>,
    pub(super) feature_report: Arc<RuntimePluginFeatureDependencyReport>,
    pub(super) extension_report: Arc<RuntimeExtensionCatalogReport>,
    selection: CompiledRuntimePluginSelection,
}

impl CompiledProjectPluginPlan {
    /// Catalog generation from which this immutable plan was compiled.
    pub fn catalog_generation(&self) -> PluginCatalogGeneration {
        self.catalog_generation
    }

    /// Deterministic fingerprint of the source manifest used as the cache key.
    pub fn source_manifest_fingerprint(&self) -> u64 {
        self.source_manifest_fingerprint
    }

    /// Runtime target whose feature and extension policy was applied.
    pub fn target_mode(&self) -> RuntimeTargetMode {
        self.target_mode
    }

    /// Exact profile identity and maturity policy applied before executable selection.
    pub fn runtime_profile_admission(&self) -> Option<(RuntimeProfileId, PluginMaturity)> {
        self.runtime_profile_admission
    }

    /// Completed project manifest retained by this plan generation.
    pub fn completed_manifest(&self) -> &ProjectPluginManifest {
        self.completed_manifest.as_ref()
    }

    /// Frozen feature dependency result retained by this plan generation.
    pub fn feature_dependency_report(&self) -> &RuntimePluginFeatureDependencyReport {
        self.feature_report.as_ref()
    }

    /// Frozen runtime extension result retained by this plan generation.
    pub fn runtime_extensions(&self) -> &RuntimeExtensionCatalogReport {
        self.extension_report.as_ref()
    }

    /// Shared frozen extension snapshot retained by this plan generation.
    pub fn runtime_extensions_handle(&self) -> Arc<RuntimeExtensionCatalogReport> {
        Arc::clone(&self.extension_report)
    }

    /// Linked base plugin providers selected by this plan, in module activation order.
    pub fn linked_provider_package_ids(&self) -> &[String] {
        self.selection.linked_provider_package_ids()
    }

    /// Native dynamic base plugin providers selected by this plan, in module activation order.
    pub fn native_dynamic_provider_package_ids(&self) -> &[String] {
        self.selection.native_dynamic_provider_package_ids()
    }

    /// Ordered modules proposed by the exact plugin and feature providers selected by this plan.
    pub fn module_proposals(&self) -> &[RuntimePluginModuleProposal] {
        self.selection.module_proposals()
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
        Arc::clone(
            &self
                .compiled_project_plan(manifest, target)
                .completed_manifest,
        )
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
        let cache_metrics = self
            .project_plans
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .metrics();
        RuntimePluginCatalogProjectPlanMetrics {
            catalog_generation: self.catalog_generation,
            project_plan_builds: cache_metrics.project_plan_builds,
        }
    }

    pub fn project_plan_cache_metrics(&self) -> RuntimePluginCatalogProjectPlanCacheMetrics {
        let cache_metrics = self
            .project_plans
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .metrics();
        RuntimePluginCatalogProjectPlanCacheMetrics {
            catalog_generation: self.catalog_generation,
            project_plan_builds: cache_metrics.project_plan_builds,
            total_build_elapsed_ns: cache_metrics.total_build_elapsed_ns,
            max_build_elapsed_ns: cache_metrics.max_build_elapsed_ns,
            cache_hits: cache_metrics.cache_hits,
            cache_misses: cache_metrics.cache_misses,
            cache_evictions: cache_metrics.cache_evictions,
            cached_plan_count: cache_metrics.cached_plan_count,
        }
    }

    #[cfg(test)]
    pub(crate) fn cached_project_plan_count(&self) -> usize {
        self.project_plans
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    /// Compiles or reuses the immutable plugin plan for one catalog generation and target.
    /// 对同一目录代、完整源清单和目标复用一次编译，产出一致的清单、依赖与扩展结果。
    pub fn compiled_project_plan(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Arc<CompiledProjectPluginPlan> {
        self.compiled_project_plan_with_admission(manifest, target, None)
    }

    /// Applies profile identity, target and maturity to one executable plugin plan.
    /// Custom required/default/optional requests must be supplied in `manifest`;
    /// those descriptor lists are not consumed by this admission API.
    pub fn compiled_project_plan_for_runtime_profile(
        &self,
        manifest: &ProjectPluginManifest,
        profile: &RuntimeProfileDescriptor,
    ) -> Arc<CompiledProjectPluginPlan> {
        self.compiled_project_plan_with_admission(
            manifest,
            profile.target_mode,
            Some((profile.id, profile.minimum_maturity)),
        )
    }

    fn compiled_project_plan_with_admission(
        &self,
        manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
        admission: Option<(RuntimeProfileId, PluginMaturity)>,
    ) -> Arc<CompiledProjectPluginPlan> {
        let fingerprint = project_manifest_fingerprint(manifest);
        let cache_entry = {
            let mut project_plans = self
                .project_plans
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if admission.is_none() {
                project_plans.lookup(self.catalog_generation, fingerprint, manifest, target)
            } else {
                project_plans.lookup_with_admission(
                    self.catalog_generation,
                    fingerprint,
                    manifest,
                    target,
                    admission,
                )
            }
        };
        let cache_entry = cache_entry.unwrap_or_else(|| {
            let candidate = Arc::new(if admission.is_none() {
                ProjectPlanCacheEntry::new(self.catalog_generation, fingerprint, manifest.clone())
            } else {
                ProjectPlanCacheEntry::new_with_admission(
                    self.catalog_generation,
                    fingerprint,
                    manifest.clone(),
                    admission,
                )
            });
            self.project_plans
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .insert_or_get(candidate, target)
        });

        let mut build_elapsed_ns = None;
        let plan = Arc::clone(cache_entry.plan.get_or_init(|| {
            let build_started = Instant::now();
            let completed_manifest = Arc::new(complete_catalog_project_manifest(
                &self.registrations,
                &self.projection,
                &cache_entry.source_manifest,
            ));
            // Keep raw request intent; one admitted snapshot drives all executable consumers.
            let (admitted_manifest, maturity_diagnostics) =
                selection::manifest_admitted_for_maturity(
                    &self.registrations,
                    completed_manifest.as_ref(),
                    target,
                    admission.map(|(_, minimum)| minimum),
                );
            let mut base_selection = CompiledRuntimePluginBaseSelection::compile(
                &self.registrations,
                &self.feature_registrations,
                &admitted_manifest,
                target,
            );
            let (effective_enabled_plugins, available_capabilities) =
                base_selection.take_feature_dependency_inputs();
            let feature_report = Arc::new(feature_dependency_report_for_effective_base(
                &self.projection,
                &admitted_manifest,
                target,
                effective_enabled_plugins,
                available_capabilities,
            ));
            let selection = base_selection.complete(
                &self.registrations,
                &self.feature_registrations,
                &self.projection,
                &admitted_manifest,
                target,
                &feature_report,
            );
            let mut extension_report = runtime_extension_report_for_project(
                &self.registrations,
                &self.feature_registrations,
                target,
                &feature_report,
                &selection,
            );
            extension_report
                .diagnostics
                .extend(maturity_diagnostics.iter().cloned());
            extension_report
                .fatal_diagnostics
                .extend(maturity_diagnostics);
            let extension_report = Arc::new(extension_report);
            let plan = Arc::new(CompiledProjectPluginPlan {
                catalog_generation: self.catalog_generation,
                source_manifest_fingerprint: fingerprint,
                target_mode: target,
                runtime_profile_admission: admission,
                completed_manifest,
                feature_report,
                extension_report,
                selection,
            });
            build_elapsed_ns =
                Some(build_started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64);
            plan
        }));
        if let Some(build_elapsed_ns) = build_elapsed_ns {
            self.project_plans
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .finish_build(&cache_entry, target, build_elapsed_ns);
        }
        plan
    }
}
