use std::time::{Duration, Instant};

use crate::{
    asset::{AssetEvent, SceneAsset},
    scene::{
        EntityRemap,
        dynamic_scene::{DynamicSceneError, PreparedDynamicSceneSpawn},
    },
};

#[cfg(test)]
use crate::scene::{LevelSystem, World};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DynamicSceneAssetReloadResult {
    event: AssetEvent<SceneAsset>,
    result: Result<PreparedDynamicSceneSpawn, DynamicSceneError>,
    estimated_bytes: usize,
    queued_at: Instant,
}

impl DynamicSceneAssetReloadResult {
    pub(crate) fn new(
        event: AssetEvent<SceneAsset>,
        result: Result<PreparedDynamicSceneSpawn, DynamicSceneError>,
    ) -> Self {
        let estimated_bytes = estimate_result_bytes(&result);
        Self {
            event,
            result,
            estimated_bytes,
            queued_at: Instant::now(),
        }
    }

    pub(crate) fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub(crate) fn result(&self) -> &Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        &self.result
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        AssetEvent<SceneAsset>,
        Result<PreparedDynamicSceneSpawn, DynamicSceneError>,
    ) {
        (self.event, self.result)
    }

    pub(crate) fn into_result(self) -> Result<PreparedDynamicSceneSpawn, DynamicSceneError> {
        self.result
    }

    pub(crate) fn estimated_bytes(&self) -> usize {
        self.estimated_bytes
    }

    pub(crate) fn age(&self) -> Duration {
        self.queued_at.elapsed()
    }

    pub(crate) fn bounded_to(self, limit_bytes: usize) -> Self {
        let estimated_bytes = self.estimated_bytes();
        if estimated_bytes <= limit_bytes {
            return self;
        }
        let result = Err(DynamicSceneError::ReloadResultTooLarge {
            estimated_bytes,
            limit_bytes,
        });
        Self {
            event: self.event,
            estimated_bytes: estimate_result_bytes(&result),
            result,
            queued_at: self.queued_at,
        }
    }

    #[cfg(test)]
    pub(crate) fn spawn_into(
        self,
        world: &mut World,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        self.spawn_into_with_target_limit(world, usize::MAX)
    }

    #[cfg(test)]
    pub(crate) fn spawn_into_with_target_limit(
        self,
        world: &mut World,
        target_snapshot_limit_bytes: usize,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        let Self { event, result, .. } = self;
        let prepared = match result {
            Ok(prepared) => prepared,
            Err(error) => return Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        };
        let component_type_count = prepared.component_type_count();
        let entity_count = prepared.entity_count();
        let resource_count = prepared.resource_count();

        let staged = match prepared.stage_into_with_limit(world, target_snapshot_limit_bytes) {
            Ok(staged) => staged,
            Err(error) => return Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        };
        match staged.commit_into(world) {
            Ok(remap) => Ok(DynamicSceneAssetReloadAppliedScene::new(
                event,
                remap,
                component_type_count,
                entity_count,
                resource_count,
            )),
            Err(error) => Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        }
    }

    #[cfg(test)]
    pub(crate) fn spawn_into_level(
        self,
        level: &LevelSystem,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        self.spawn_into_level_with_target_limit(level, usize::MAX)
    }

    #[cfg(test)]
    pub(crate) fn spawn_into_level_with_target_limit(
        self,
        level: &LevelSystem,
        target_snapshot_limit_bytes: usize,
    ) -> Result<DynamicSceneAssetReloadAppliedScene, DynamicSceneAssetReloadApplyFailure> {
        let Self { event, result, .. } = self;
        let prepared = match result {
            Ok(prepared) => prepared,
            Err(error) => return Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        };
        let component_type_count = prepared.component_type_count();
        let entity_count = prepared.entity_count();
        let resource_count = prepared.resource_count();
        let staged = match prepared.stage_into_level(level, target_snapshot_limit_bytes) {
            Ok(staged) => staged,
            Err(error) => return Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        };
        match staged.commit_into_level(level) {
            Ok(remap) => Ok(DynamicSceneAssetReloadAppliedScene::new(
                event,
                remap,
                component_type_count,
                entity_count,
                resource_count,
            )),
            Err(error) => Err(DynamicSceneAssetReloadApplyFailure::new(event, error)),
        }
    }
}

fn estimate_result_bytes(result: &Result<PreparedDynamicSceneSpawn, DynamicSceneError>) -> usize {
    match result {
        Ok(prepared) => prepared.estimated_bytes(),
        Err(error) => std::mem::size_of::<DynamicSceneAssetReloadResult>()
            .saturating_add(error.to_string().len()),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadAppliedScene {
    event: AssetEvent<SceneAsset>,
    remap: EntityRemap,
    component_type_count: usize,
    entity_count: usize,
    resource_count: usize,
}

impl DynamicSceneAssetReloadAppliedScene {
    pub fn new(
        event: AssetEvent<SceneAsset>,
        remap: EntityRemap,
        component_type_count: usize,
        entity_count: usize,
        resource_count: usize,
    ) -> Self {
        Self {
            event,
            remap,
            component_type_count,
            entity_count,
            resource_count,
        }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn remap(&self) -> &EntityRemap {
        &self.remap
    }

    pub fn component_type_count(&self) -> usize {
        self.component_type_count
    }

    pub fn entity_count(&self) -> usize {
        self.entity_count
    }

    pub fn resource_count(&self) -> usize {
        self.resource_count
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadApplyFailure {
    event: AssetEvent<SceneAsset>,
    error: DynamicSceneError,
}

impl DynamicSceneAssetReloadApplyFailure {
    pub fn new(event: AssetEvent<SceneAsset>, error: DynamicSceneError) -> Self {
        Self { event, error }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn error(&self) -> &DynamicSceneError {
        &self.error
    }

    pub fn into_error(self) -> DynamicSceneError {
        self.error
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadStaleResult {
    event: AssetEvent<SceneAsset>,
    latest_revision: u64,
}

impl DynamicSceneAssetReloadStaleResult {
    pub fn new(event: AssetEvent<SceneAsset>, latest_revision: u64) -> Self {
        Self {
            event,
            latest_revision,
        }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn latest_revision(&self) -> u64 {
        self.latest_revision
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicSceneAssetReloadSupersededTask {
    event: AssetEvent<SceneAsset>,
    latest_revision: u64,
    cancellation_requested: bool,
    previous_state: crate::core::TaskState,
}

impl DynamicSceneAssetReloadSupersededTask {
    pub fn new(
        event: AssetEvent<SceneAsset>,
        latest_revision: u64,
        cancellation_requested: bool,
        previous_state: crate::core::TaskState,
    ) -> Self {
        Self {
            event,
            latest_revision,
            cancellation_requested,
            previous_state,
        }
    }

    pub fn event(&self) -> &AssetEvent<SceneAsset> {
        &self.event
    }

    pub fn latest_revision(&self) -> u64 {
        self.latest_revision
    }

    pub fn cancellation_requested(&self) -> bool {
        self.cancellation_requested
    }

    pub fn previous_state(&self) -> crate::core::TaskState {
        self.previous_state
    }
}

#[cfg(test)]
mod tests {
    use std::{
        hint::black_box,
        time::{Duration, Instant},
    };

    use crate::{
        asset::{AssetEvent, Handle, SceneAsset},
        core::resource::ResourceId,
        scene::dynamic_scene::DynamicSceneError,
    };

    use super::DynamicSceneAssetReloadResult;

    const PERF_SAMPLE_PAIRS: usize = 21;

    fn error_result(reason: String) -> DynamicSceneAssetReloadResult {
        let event = AssetEvent::Modified {
            handle: Handle::<SceneAsset>::new(ResourceId::from_stable_label(
                "reload result size cache",
            )),
            locator: None,
            revision: 7,
        };
        DynamicSceneAssetReloadResult::new(event, Err(DynamicSceneError::Parse { reason }))
    }

    #[inline(never)]
    fn legacy_estimated_bytes(result: &DynamicSceneAssetReloadResult, reads: usize) -> usize {
        (0..reads).fold(0usize, |sum, _| {
            let bytes = match black_box(result.result()) {
                Ok(prepared) => prepared.estimated_bytes(),
                Err(error) => std::mem::size_of::<DynamicSceneAssetReloadResult>()
                    .saturating_add(error.to_string().len()),
            };
            sum.wrapping_add(black_box(bytes))
        })
    }

    #[inline(never)]
    fn cached_estimated_bytes(result: &DynamicSceneAssetReloadResult, reads: usize) -> usize {
        (0..reads).fold(0usize, |sum, _| {
            sum.wrapping_add(black_box(result.estimated_bytes()))
        })
    }

    fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (percentile * sorted.len()).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    #[test]
    fn dynamic_scene_asset_reload_oversized_result_becomes_bounded_failure() {
        let result = error_result("x".repeat(8 * 1024)).bounded_to(1_024);

        assert!(result.estimated_bytes() <= 1_024);
        assert!(matches!(
            result.result(),
            Err(DynamicSceneError::ReloadResultTooLarge { .. })
        ));
    }

    #[test]
    fn dynamic_scene_asset_reload_result_reuses_cached_size_estimate() {
        let result = error_result("cached-size".repeat(512));
        let expected = legacy_estimated_bytes(&result, 1);

        assert_eq!(result.estimated_bytes(), expected);
        assert_eq!(result.estimated_bytes(), expected);
    }

    #[test]
    #[ignore = "managed Runtime53 performance evidence"]
    fn dynamic_scene_asset_reload_runtime53_performance_cached_result_size() {
        const READS_PER_SAMPLE: usize = 1_024;
        let result = error_result("cached-size-benchmark".repeat(256));
        assert_eq!(
            legacy_estimated_bytes(&result, 1),
            cached_estimated_bytes(&result, 1)
        );

        black_box(legacy_estimated_bytes(&result, READS_PER_SAMPLE));
        black_box(cached_estimated_bytes(&result, READS_PER_SAMPLE));

        let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        let mut cached_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
        for pair in 0..PERF_SAMPLE_PAIRS {
            let mut measure_legacy = || {
                let started = Instant::now();
                black_box(legacy_estimated_bytes(&result, READS_PER_SAMPLE));
                legacy_samples.push(started.elapsed());
            };
            let mut measure_cached = || {
                let started = Instant::now();
                black_box(cached_estimated_bytes(&result, READS_PER_SAMPLE));
                cached_samples.push(started.elapsed());
            };
            if pair % 2 == 0 {
                measure_legacy();
                measure_cached();
            } else {
                measure_cached();
                measure_legacy();
            }
        }

        let legacy_p50 = nearest_rank(&legacy_samples, 50);
        let legacy_p95 = nearest_rank(&legacy_samples, 95);
        let cached_p50 = nearest_rank(&cached_samples, 50);
        let cached_p95 = nearest_rank(&cached_samples, 95);
        let legacy_ns = legacy_samples
            .iter()
            .map(Duration::as_nanos)
            .collect::<Vec<_>>();
        let cached_ns = cached_samples
            .iter()
            .map(Duration::as_nanos)
            .collect::<Vec<_>>();
        let legacy_csv = legacy_ns
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let cached_csv = cached_ns
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",");

        eprintln!(
            "RUNTIME53_RESULT_SIZE_CACHE_BENCH_V1 sample_pairs={PERF_SAMPLE_PAIRS} reads_per_sample={READS_PER_SAMPLE} pair_order=alternating_legacy_even legacy_p50_ns={} legacy_p95_ns={} cached_p50_ns={} cached_p95_ns={} legacy_ns={legacy_csv} cached_ns={cached_csv}",
            legacy_p50.as_nanos(),
            legacy_p95.as_nanos(),
            cached_p50.as_nanos(),
            cached_p95.as_nanos(),
        );
        assert!(
            cached_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(20),
            "cached result-size reads must reduce P95 by at least 80%: legacy={legacy_p95:?}, cached={cached_p95:?}"
        );
    }
}
