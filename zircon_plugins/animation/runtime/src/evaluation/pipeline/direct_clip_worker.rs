use std::collections::BTreeMap;
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::Arc;

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::scene::EntityId;

use super::clip_sample::sample_pose_requests;
use super::requests::PendingPoseSample;
use super::AnimationEvaluationPipeline;
use crate::AnimationClipEvaluator;

/// Maximum worker tasks the animation owner may submit for one direct-clip frame.
pub const MAX_DIRECT_CLIP_WORKER_SHARDS: usize = 4;

fn direct_clip_shard_capacity(item_count: usize, shard_count: usize) -> usize {
    if shard_count == 0 {
        return 0;
    }
    item_count / shard_count + usize::from(item_count % shard_count != 0)
}

fn new_direct_clip_batches<T>(item_count: usize, shard_count: usize) -> Vec<Vec<T>> {
    let shard_capacity = direct_clip_shard_capacity(item_count, shard_count);
    (0..shard_count)
        .map(|_| Vec::with_capacity(shard_capacity))
        .collect()
}

/// Per-frame and cumulative direct-clip work accepted by Runtime11 workers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirectClipWorkerStats {
    pub total_batch_count: u64,
    pub total_owner_submission_count: u64,
    pub total_instance_count: u64,
    pub last_instance_count: usize,
    pub last_shard_count: usize,
    pub last_owner_submission_count: usize,
    pub last_min_shard_len: usize,
    pub last_max_shard_len: usize,
}

struct DirectClipWorkerResult {
    evaluator: AnimationClipEvaluator,
    poses: BTreeMap<EntityId, AnimationPoseOutput>,
}

struct DirectClipWorkerTask {
    shard_index: usize,
    result: Receiver<DirectClipWorkerResult>,
}

pub(super) fn sample_direct_clip_pose_requests(
    core: &CoreHandle,
    pipeline: &mut AnimationEvaluationPipeline,
    asset_manager: Arc<ProjectAssetManager>,
    pending_samples: Vec<PendingPoseSample>,
) -> BTreeMap<EntityId, AnimationPoseOutput> {
    if pending_samples.is_empty() {
        pipeline.record_direct_clip_worker_batches(std::iter::empty(), 0);
        return BTreeMap::new();
    }

    let shard_count = core
        .scheduler()
        .parallelism()
        .min(MAX_DIRECT_CLIP_WORKER_SHARDS)
        .min(pending_samples.len())
        .max(1);
    let mut batches = new_direct_clip_batches(pending_samples.len(), shard_count);
    for (index, pending) in pending_samples.into_iter().enumerate() {
        batches[index % shard_count].push(pending);
    }
    pipeline.record_direct_clip_worker_batches(batches.iter().map(Vec::len), shard_count);

    let mut tasks = Vec::with_capacity(shard_count);
    for (shard_index, batch) in batches.into_iter().enumerate() {
        let mut evaluator = pipeline.take_direct_clip_worker_evaluator(shard_index);
        let asset_manager = Arc::clone(&asset_manager);
        let (result_sender, result_receiver) = sync_channel(1);
        let _ = core.scheduler().schedule(move || {
            evaluator.bind_resources(&asset_manager.resource_manager());
            let poses = sample_pose_requests(&mut evaluator, asset_manager.as_ref(), batch);
            let _ = result_sender.send(DirectClipWorkerResult { evaluator, poses });
        });
        tasks.push(DirectClipWorkerTask {
            shard_index,
            result: result_receiver,
        });
    }

    let mut poses = BTreeMap::new();
    for task in tasks {
        let result = task.result.recv().unwrap_or_else(|_| {
            panic!(
                "direct clip worker shard {} terminated before returning its evaluator",
                task.shard_index
            )
        });
        pipeline.restore_direct_clip_worker_evaluator(task.shard_index, result.evaluator);
        poses.extend(result.poses);
    }
    poses
}

#[cfg(test)]
mod optimization_batch_20260830co_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::{direct_clip_shard_capacity, new_direct_clip_batches};

    const SAMPLE_PAIRS: usize = 21;
    const BENCH_ITEMS: usize = 65_536;
    const BENCH_SHARDS: usize = 4;

    #[test]
    fn optimization_batch_20260830co_direct_clip_capacity_matches_round_robin_peak() {
        assert_eq!(direct_clip_shard_capacity(0, 0), 0);
        assert_eq!(direct_clip_shard_capacity(1, 1), 1);
        assert_eq!(direct_clip_shard_capacity(10, 4), 3);

        let mut batches = new_direct_clip_batches::<usize>(10, 4);
        for item in 0..10 {
            batches[item % 4].push(item);
        }
        assert_eq!(
            batches.iter().map(Vec::len).collect::<Vec<_>>(),
            [3, 3, 2, 2]
        );
        assert!(batches.iter().all(|batch| batch.capacity() >= 3));
    }

    #[test]
    #[ignore = "release-only direct clip shard capacity benchmark"]
    fn optimization_batch_20260830co_direct_clip_capacity_release_benchmark() {
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut legacy_growth_events = 0usize;
        let mut optimized_growth_events = 0usize;

        for pair in 0..SAMPLE_PAIRS {
            let (legacy, optimized) = if pair % 2 == 0 {
                (measure_legacy(), measure_optimized())
            } else {
                let optimized = measure_optimized();
                let legacy = measure_legacy();
                (legacy, optimized)
            };
            assert_eq!(legacy.2, optimized.2);
            legacy_samples.push(legacy.0);
            optimized_samples.push(optimized.0);
            legacy_growth_events = legacy_growth_events.saturating_add(legacy.1);
            optimized_growth_events = optimized_growth_events.saturating_add(optimized.1);
        }

        let legacy_p95_ns = percentile(&legacy_samples, 95);
        let optimized_p95_ns = percentile(&optimized_samples, 95);
        println!(
            "RUNTIME170_DIRECT_CLIP_SHARD_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} items={BENCH_ITEMS} shards={BENCH_SHARDS} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} legacy_raw_ns={} optimized_raw_ns={}",
            raw(&legacy_samples),
            raw(&optimized_samples),
        );
        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
    }

    fn measure_legacy() -> (u128, usize, usize) {
        let started = Instant::now();
        let mut growth_events = 0usize;
        let mut batches = (0..BENCH_SHARDS)
            .map(|_| Vec::new())
            .collect::<Vec<Vec<usize>>>();
        for item in 0..BENCH_ITEMS {
            let batch = &mut batches[item % BENCH_SHARDS];
            growth_events += usize::from(batch.len() == batch.capacity());
            batch.push(item);
        }
        let checksum = batches.iter().flatten().copied().sum::<usize>();
        (
            started.elapsed().as_nanos(),
            growth_events,
            black_box(checksum),
        )
    }

    fn measure_optimized() -> (u128, usize, usize) {
        let started = Instant::now();
        let mut growth_events = 0usize;
        let mut batches = new_direct_clip_batches(BENCH_ITEMS, BENCH_SHARDS);
        for item in 0..BENCH_ITEMS {
            let batch = &mut batches[item % BENCH_SHARDS];
            growth_events += usize::from(batch.len() == batch.capacity());
            batch.push(item);
        }
        let checksum = batches.iter().flatten().copied().sum::<usize>();
        (
            started.elapsed().as_nanos(),
            growth_events,
            black_box(checksum),
        )
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() - 1) * percentile / 100]
    }

    fn raw(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
