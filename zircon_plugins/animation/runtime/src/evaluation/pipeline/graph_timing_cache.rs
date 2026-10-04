//! 状态机退出时间通过图中剪辑时长和播放速度估算；缓存签名覆盖每个可计时剪辑修订。
use zircon_runtime::asset::{AssetId, ProjectAssetManager};
use zircon_runtime::core::framework::animation::AnimationClipAsset;
use zircon_runtime::core::math::Real;
use zircon_runtime::core::resource::{AnimationClipMarker, ResourceHandle, ResourceSnapshot};

use crate::CompiledAnimationGraphEvaluation;

use super::AnimationEvaluationPipeline;

const GRAPH_TIMING_CACHE_LIMIT: usize = 128;

#[derive(Debug)]
pub(super) struct CachedGraphTiming {
    signature: Box<[ClipTimingRevision]>,
    duration_seconds: Real,
    last_used: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClipTimingRevision {
    clip_id: AssetId,
    revision: u64,
    playback_speed_bits: u32,
}

impl AnimationEvaluationPipeline {
    /// 估算状态机退出门槛的参考时长；缺失或不可计时剪辑会使结果不可用。
    pub(super) fn graph_duration_seconds(
        &mut self,
        assets: &ProjectAssetManager,
        graph_id: AssetId,
        skeleton_id: AssetId,
        evaluation: &CompiledAnimationGraphEvaluation,
    ) -> Option<Real> {
        let key = (graph_id, skeleton_id);
        if let Some(signature) = self
            .graph_timing_cache
            .get(&key)
            .map(|cached| cached.signature.as_ref())
        {
            if graph_timing_signature_matches(assets, evaluation, signature)? {
                self.graph_timing_access_sequence =
                    self.graph_timing_access_sequence.saturating_add(1);
                let access = self.graph_timing_access_sequence;
                let cached = self.graph_timing_cache.get_mut(&key)?;
                cached.last_used = access;
                return Some(cached.duration_seconds);
            }
        }

        let mut signature = Vec::with_capacity(evaluation.clips().len());
        let mut duration_seconds: Real = 0.0;
        for clip in evaluation.clips() {
            let clip_id = assets.resolve_asset_id(&clip.clip().locator)?;
            let snapshot = load_clip_snapshot(assets, clip_id)?;
            let speed = clip.playback_speed().abs();
            if speed <= Real::EPSILON || !speed.is_finite() {
                continue;
            }
            signature.push(ClipTimingRevision {
                clip_id,
                revision: snapshot.revision(),
                playback_speed_bits: speed.to_bits(),
            });
            let adjusted_duration = snapshot.duration_seconds / speed;
            if adjusted_duration.is_finite() && adjusted_duration > Real::EPSILON {
                duration_seconds = duration_seconds.max(adjusted_duration);
            }
        }
        if duration_seconds <= Real::EPSILON {
            return None;
        }

        self.graph_timing_access_sequence = self.graph_timing_access_sequence.saturating_add(1);
        let access = self.graph_timing_access_sequence;
        let signature = signature.into_boxed_slice();
        self.graph_timing_cache.insert(
            key,
            CachedGraphTiming {
                signature,
                duration_seconds,
                last_used: access,
            },
        );
        self.enforce_graph_timing_cache_limit();
        let cached = self.graph_timing_cache.get_mut(&key)?;
        cached.last_used = access;
        Some(cached.duration_seconds)
    }

    fn enforce_graph_timing_cache_limit(&mut self) {
        while self.graph_timing_cache.len() > GRAPH_TIMING_CACHE_LIMIT {
            let Some(oldest) = self
                .graph_timing_cache
                .iter()
                .min_by_key(|(key, cached)| (cached.last_used, **key))
                .map(|(key, _)| *key)
            else {
                break;
            };
            self.graph_timing_cache.remove(&oldest);
        }
    }
}

fn graph_timing_signature_matches(
    assets: &ProjectAssetManager,
    evaluation: &CompiledAnimationGraphEvaluation,
    signature: &[ClipTimingRevision],
) -> Option<bool> {
    let mut expected = signature.iter();
    for clip in evaluation.clips() {
        let clip_id = assets.resolve_asset_id(&clip.clip().locator)?;
        let snapshot = load_clip_snapshot(assets, clip_id)?;
        let speed = clip.playback_speed().abs();
        if speed <= Real::EPSILON || !speed.is_finite() {
            continue;
        }
        let revision = ClipTimingRevision {
            clip_id,
            revision: snapshot.revision(),
            playback_speed_bits: speed.to_bits(),
        };
        if expected.next() != Some(&revision) {
            return Some(false);
        }
    }
    Some(expected.next().is_none())
}

fn load_clip_snapshot(
    assets: &ProjectAssetManager,
    clip_id: AssetId,
) -> Option<ResourceSnapshot<AnimationClipAsset>> {
    let resources = assets.resource_manager();
    let handle = ResourceHandle::<AnimationClipMarker>::new(clip_id);
    resources.snapshot(handle).or_else(|| {
        assets.load_animation_clip_asset(clip_id).ok()?;
        resources.snapshot(handle)
    })
}

#[cfg(test)]
#[path = "tests/graph_timing_cache_optimization_tests.rs"]
mod optimization_tests;
