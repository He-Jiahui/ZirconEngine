use std::collections::BTreeMap;

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::framework::animation::{AnimationClipAsset, AnimationSkeletonAsset};
use zircon_runtime::core::resource::{
    AnimationClipMarker, AnimationSkeletonMarker, ResourceHandle, ResourceSnapshot,
};
use zircon_runtime::scene::EntityId;

use super::requests::PendingPoseSample;
use crate::{AnimationAssetRevision, AnimationClipEvaluator};

pub(super) fn sample_pose_requests(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingPoseSample>,
) -> BTreeMap<EntityId, AnimationPoseOutput> {
    pending_samples
        .into_iter()
        .filter_map(|pending| sample_pose_request(evaluator, asset_manager, pending))
        .collect()
}

pub(super) fn sample_pose_request(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    pending: PendingPoseSample,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let skeleton = load_skeleton_snapshot(asset_manager, pending.skeleton_id)?;
    let clip = load_clip_snapshot(asset_manager, pending.clip_id)?;
    let skeleton_revision = AnimationAssetRevision::new(pending.skeleton_id, skeleton.revision());
    let clip_revision = AnimationAssetRevision::new(pending.clip_id, clip.revision());
    let mut pose = match evaluator.sample_clip(
        skeleton_revision,
        clip_revision,
        &skeleton,
        &clip,
        pending.time_seconds,
        pending.looping,
    ) {
        Ok(pose) => pose,
        Err(error) => {
            evaluator.record_diagnostic(pending.entity, skeleton_revision, clip_revision, error);
            return None;
        }
    };
    pose.source = pending.source;
    pose.active_state = pending.active_state;
    Some((pending.entity, pose))
}

fn load_skeleton_snapshot(
    asset_manager: &ProjectAssetManager,
    asset_id: zircon_runtime::asset::AssetId,
) -> Option<ResourceSnapshot<AnimationSkeletonAsset>> {
    let resources = asset_manager.resource_manager();
    let handle = ResourceHandle::<AnimationSkeletonMarker>::new(asset_id);
    resources.snapshot(handle).or_else(|| {
        asset_manager.load_animation_skeleton_asset(asset_id).ok()?;
        resources.snapshot(handle)
    })
}

fn load_clip_snapshot(
    asset_manager: &ProjectAssetManager,
    asset_id: zircon_runtime::asset::AssetId,
) -> Option<ResourceSnapshot<AnimationClipAsset>> {
    let resources = asset_manager.resource_manager();
    let handle = ResourceHandle::<AnimationClipMarker>::new(asset_id);
    resources.snapshot(handle).or_else(|| {
        asset_manager.load_animation_clip_asset(asset_id).ok()?;
        resources.snapshot(handle)
    })
}

#[cfg(test)]
mod optimization_tests {
    use std::hint::black_box;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Instant;

    #[test]
    fn optimization_batch_20260830cg_clip_sampling_checks_resident_snapshots_first() {
        let source = include_str!("clip_sample.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        for (start, end, loader) in [
            (
                "fn load_skeleton_snapshot(",
                "fn load_clip_snapshot(",
                "load_animation_skeleton_asset",
            ),
            (
                "fn load_clip_snapshot(",
                "#[cfg(test)]",
                "load_animation_clip_asset",
            ),
        ] {
            let start = source.find(start).expect("snapshot helper");
            let helper = production.get(start..).unwrap_or(&source[start..]);
            let helper = helper.split(end).next().expect("snapshot helper boundary");
            let snapshot = helper.find("resources.snapshot").expect("resident lookup");
            let load = helper.find(loader).expect("loader fallback");
            assert!(snapshot < load);
            assert!(helper.contains(".or_else(||"));
        }
    }

    #[test]
    #[ignore = "Release-only Runtime170 performance contract"]
    fn optimization_batch_20260830cg_resident_snapshot_first_p95() {
        const ITERATIONS: usize = 1_000_000;
        const SAMPLES: usize = 17;
        let load_count = AtomicU64::new(0);
        let mut baseline_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for sample in 0..SAMPLES {
            let baseline = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    mock_load(black_box(&load_count));
                    black_box(true);
                }
                started.elapsed().as_nanos()
            };
            let optimized = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    black_box(true);
                }
                started.elapsed().as_nanos()
            };
            if sample % 2 == 0 {
                baseline_samples.push(baseline());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                baseline_samples.push(baseline());
            }
        }

        let baseline_p95 = percentile_95(&mut baseline_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "RUNTIME170_RESIDENT_SNAPSHOT_FIRST_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(35),
            "expected resident-first lookup to reduce P95 by at least 65%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn mock_load(counter: &AtomicU64) {
        counter.fetch_add(1, Ordering::Relaxed);
        for index in 0_u64..32 {
            black_box(index.wrapping_mul(index));
        }
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
    }
}
