use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, OnceLock,
};

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;

use super::super::PluginCatalogGeneration;
use super::CompiledProjectPluginPlan;

// Retain a small alternating-consumer window without allowing arbitrary manifests to pin graphs.
const PROJECT_PLAN_CACHE_WAYS_PER_TARGET: usize = 4;

#[derive(Debug, Default)]
pub(in super::super) struct ProjectPlanCache {
    client_runtime: VecDeque<Arc<ProjectPlanCacheEntry>>,
    server_runtime: VecDeque<Arc<ProjectPlanCacheEntry>>,
    editor_host: VecDeque<Arc<ProjectPlanCacheEntry>>,
    cache_hits: u64,
    cache_misses: u64,
    cache_evictions: u64,
    project_plan_builds: u64,
    total_build_elapsed_ns: u64,
    max_build_elapsed_ns: u64,
}

impl ProjectPlanCache {
    pub(super) fn lookup(
        &mut self,
        catalog_generation: PluginCatalogGeneration,
        source_fingerprint: u64,
        source_manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Option<Arc<ProjectPlanCacheEntry>> {
        let entry = self.find(
            catalog_generation,
            source_fingerprint,
            source_manifest,
            target,
        );
        if entry.is_some() {
            self.cache_hits = self.cache_hits.saturating_add(1);
        } else {
            self.cache_misses = self.cache_misses.saturating_add(1);
        }
        entry
    }

    pub(super) fn find(
        &mut self,
        catalog_generation: PluginCatalogGeneration,
        source_fingerprint: u64,
        source_manifest: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Option<Arc<ProjectPlanCacheEntry>> {
        let plans = self.plans_for_target(target);
        if let Some(index) = plans.iter().position(|entry| {
            entry.matches(catalog_generation, source_fingerprint, source_manifest)
        }) {
            let entry = plans
                .remove(index)
                .expect("matched project plan cache entry should exist");
            plans.push_back(Arc::clone(&entry));
            return Some(entry);
        }
        None
    }

    pub(super) fn insert_or_get(
        &mut self,
        candidate: Arc<ProjectPlanCacheEntry>,
        target: RuntimeTargetMode,
    ) -> Arc<ProjectPlanCacheEntry> {
        if let Some(entry) = self.find(
            candidate.catalog_generation,
            candidate.source_fingerprint,
            &candidate.source_manifest,
            target,
        ) {
            return entry;
        }

        let evictions = {
            let plans = self.plans_for_target(target);
            plans.push_back(Arc::clone(&candidate));
            Self::trim_completed_plans(plans)
        };
        self.cache_evictions = self.cache_evictions.saturating_add(evictions);
        candidate
    }

    pub(super) fn finish_build(
        &mut self,
        entry: &Arc<ProjectPlanCacheEntry>,
        target: RuntimeTargetMode,
        build_elapsed_ns: u64,
    ) {
        let evictions = {
            let plans = self.plans_for_target(target);
            let index = plans
                .iter()
                .position(|candidate| Arc::ptr_eq(candidate, entry))
                .expect("built project plan reservation should remain cached until publication");
            let entry = plans
                .remove(index)
                .expect("initialized project plan cache entry should exist");
            entry.evictable.store(true, Ordering::Release);
            plans.push_back(entry);
            Self::trim_completed_plans(plans)
        };
        self.project_plan_builds = self.project_plan_builds.saturating_add(1);
        self.total_build_elapsed_ns = self.total_build_elapsed_ns.saturating_add(build_elapsed_ns);
        self.max_build_elapsed_ns = self.max_build_elapsed_ns.max(build_elapsed_ns);
        self.cache_evictions = self.cache_evictions.saturating_add(evictions);
    }

    fn plans_for_target(
        &mut self,
        target: RuntimeTargetMode,
    ) -> &mut VecDeque<Arc<ProjectPlanCacheEntry>> {
        match target {
            RuntimeTargetMode::ClientRuntime => &mut self.client_runtime,
            RuntimeTargetMode::ServerRuntime => &mut self.server_runtime,
            RuntimeTargetMode::EditorHost => &mut self.editor_host,
        }
    }

    fn trim_completed_plans(plans: &mut VecDeque<Arc<ProjectPlanCacheEntry>>) -> u64 {
        let mut evictions = 0_u64;
        while plans.len() > PROJECT_PLAN_CACHE_WAYS_PER_TARGET {
            let Some(index) = plans.iter().position(|entry| {
                entry.evictable.load(Ordering::Acquire) || Arc::strong_count(entry) == 1
            }) else {
                break;
            };
            plans.remove(index);
            evictions = evictions.saturating_add(1);
        }
        evictions
    }

    pub(in super::super) fn len(&self) -> usize {
        self.client_runtime.len() + self.server_runtime.len() + self.editor_host.len()
    }

    pub(in super::super) fn clear(&mut self) {
        self.client_runtime.clear();
        self.server_runtime.clear();
        self.editor_host.clear();
    }

    pub(super) fn metrics(&self) -> ProjectPlanCacheMetrics {
        ProjectPlanCacheMetrics {
            project_plan_builds: self.project_plan_builds,
            total_build_elapsed_ns: self.total_build_elapsed_ns,
            max_build_elapsed_ns: self.max_build_elapsed_ns,
            cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            cache_evictions: self.cache_evictions,
            cached_plan_count: self.len(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProjectPlanCacheMetrics {
    pub(super) project_plan_builds: u64,
    pub(super) total_build_elapsed_ns: u64,
    pub(super) max_build_elapsed_ns: u64,
    pub(super) cache_hits: u64,
    pub(super) cache_misses: u64,
    pub(super) cache_evictions: u64,
    pub(super) cached_plan_count: usize,
}

#[derive(Debug)]
pub(super) struct ProjectPlanCacheEntry {
    pub(super) catalog_generation: PluginCatalogGeneration,
    pub(super) source_fingerprint: u64,
    pub(super) source_manifest: ProjectPluginManifest,
    pub(super) plan: OnceLock<Arc<CompiledProjectPluginPlan>>,
    evictable: AtomicBool,
}

impl ProjectPlanCacheEntry {
    pub(super) fn new(
        catalog_generation: PluginCatalogGeneration,
        source_fingerprint: u64,
        source_manifest: ProjectPluginManifest,
    ) -> Self {
        Self {
            catalog_generation,
            source_fingerprint,
            source_manifest,
            plan: OnceLock::new(),
            evictable: AtomicBool::new(false),
        }
    }

    fn matches(
        &self,
        catalog_generation: PluginCatalogGeneration,
        source_fingerprint: u64,
        source_manifest: &ProjectPluginManifest,
    ) -> bool {
        self.catalog_generation == catalog_generation
            && self.source_fingerprint == source_fingerprint
            && &self.source_manifest == source_manifest
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::framework::platform::RuntimeTargetMode;
    use crate::core::framework::project::ProjectPluginManifest;

    use super::{
        PluginCatalogGeneration, ProjectPlanCache, ProjectPlanCacheEntry,
        PROJECT_PLAN_CACHE_WAYS_PER_TARGET,
    };

    #[test]
    fn cache_retains_uninitialized_single_flight_entries_past_the_resident_limit() {
        let mut cache = ProjectPlanCache::default();
        let manifest = ProjectPluginManifest::default();
        let first = Arc::new(ProjectPlanCacheEntry::new(
            PluginCatalogGeneration::INITIAL,
            0,
            manifest.clone(),
        ));
        cache.insert_or_get(Arc::clone(&first), RuntimeTargetMode::ClientRuntime);
        let mut active_reservations = vec![Arc::clone(&first)];

        for fingerprint in 1..=PROJECT_PLAN_CACHE_WAYS_PER_TARGET as u64 {
            let reservation = Arc::new(ProjectPlanCacheEntry::new(
                PluginCatalogGeneration::INITIAL,
                fingerprint,
                manifest.clone(),
            ));
            cache.insert_or_get(Arc::clone(&reservation), RuntimeTargetMode::ClientRuntime);
            active_reservations.push(reservation);
        }

        assert_eq!(cache.len(), PROJECT_PLAN_CACHE_WAYS_PER_TARGET + 1);
        assert_eq!(active_reservations.len(), cache.len());
        let retained = cache
            .find(
                PluginCatalogGeneration::INITIAL,
                0,
                &manifest,
                RuntimeTargetMode::ClientRuntime,
            )
            .expect("uninitialized single-flight entry should remain discoverable");
        assert!(Arc::ptr_eq(&first, &retained));
    }

    #[test]
    fn cache_prunes_abandoned_uninitialized_reservations() {
        let mut cache = ProjectPlanCache::default();
        let manifest = ProjectPluginManifest::default();

        for fingerprint in 0..=PROJECT_PLAN_CACHE_WAYS_PER_TARGET as u64 {
            cache.insert_or_get(
                Arc::new(ProjectPlanCacheEntry::new(
                    PluginCatalogGeneration::INITIAL,
                    fingerprint,
                    manifest.clone(),
                )),
                RuntimeTargetMode::ClientRuntime,
            );
        }

        assert_eq!(cache.len(), PROJECT_PLAN_CACHE_WAYS_PER_TARGET);
        let metrics = cache.metrics();
        assert_eq!(metrics.cache_evictions, 1);
        assert_eq!(
            metrics.cached_plan_count,
            PROJECT_PLAN_CACHE_WAYS_PER_TARGET
        );
    }
}
