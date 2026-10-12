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
