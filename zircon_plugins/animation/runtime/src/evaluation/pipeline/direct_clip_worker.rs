use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, sync_channel};

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::CoreHandle;
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::scene::EntityId;

use super::AnimationEvaluationPipeline;
use super::clip_sample::sample_pose_requests;
use super::requests::PendingPoseSample;
use crate::AnimationClipEvaluator;

/// Maximum worker tasks the animation owner may submit for one direct-clip frame.
pub const MAX_DIRECT_CLIP_WORKER_SHARDS: usize = 4;

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
    let mut batches = (0..shard_count)
        .map(|_| Vec::new())
        .collect::<Vec<Vec<PendingPoseSample>>>();
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
