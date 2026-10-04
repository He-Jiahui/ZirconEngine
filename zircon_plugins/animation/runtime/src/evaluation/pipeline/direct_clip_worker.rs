//! 直接剪辑请求在核心调度器的有界分片中并行采样；每个分片暂借独立评估器，完成后归还管线以保留修订缓存。
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

/// 由动画帧所有者提交并等待全部分片归还评估器；求值工作不触碰场景写回。
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
#[path = "tests/direct_clip_worker_optimization_batch_20260830co_tests.rs"]
mod optimization_batch_20260830co_tests;
