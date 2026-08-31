use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use crossbeam_channel::Receiver;
use zircon_runtime::core::framework::animation::AnimationPoseBone;
use zircon_runtime::core::resource::{ResourceEvent, ResourceId, ResourceKind, ResourceManager};

use super::cache::{CachedClip, CachedSkeleton};
use super::cache_policy::{
    DEFAULT_CLIP_CACHE_LIMIT, DEFAULT_DIAGNOSTIC_LIMIT, DEFAULT_SKELETON_CACHE_LIMIT,
};
use super::AnimationClipEvaluatorStats;
use crate::AnimationEvaluationDiagnostic;
use crate::SkeletonTargetTable;

pub(super) type EvaluationDiagnosticKey = (ResourceId, u64, ResourceId, u64, String);

/// Revision-aware production clip evaluator with skeleton-scoped compile caches.
#[derive(Debug)]
pub struct AnimationClipEvaluator {
    pub(super) skeletons: BTreeMap<ResourceId, CachedSkeleton>,
    pub(super) clips: BTreeMap<(ResourceId, ResourceId), CachedClip>,
    pub(super) pool_size: usize,
    pub(super) skeleton_cache_limit: usize,
    pub(super) clip_cache_limit: usize,
    pub(super) diagnostic_limit: usize,
    pub(super) access_sequence: u64,
    pub(super) stats: AnimationClipEvaluatorStats,
    resource_events: Option<Receiver<ResourceEvent>>,
    pub(super) reported_diagnostics: BTreeSet<EvaluationDiagnosticKey>,
    pub(super) diagnostic_order: VecDeque<EvaluationDiagnosticKey>,
    pub(super) pending_diagnostics: Vec<AnimationEvaluationDiagnostic>,
}

impl Default for AnimationClipEvaluator {
    fn default() -> Self {
        Self::with_pool_size(4)
    }
}

impl AnimationClipEvaluator {
    pub fn with_pool_size(pool_size: usize) -> Self {
        Self::with_limits(
            pool_size,
            DEFAULT_SKELETON_CACHE_LIMIT,
            DEFAULT_CLIP_CACHE_LIMIT,
            DEFAULT_DIAGNOSTIC_LIMIT,
        )
    }

    pub fn with_limits(
        pool_size: usize,
        skeleton_cache_limit: usize,
        clip_cache_limit: usize,
        diagnostic_limit: usize,
    ) -> Self {
        Self {
            skeletons: BTreeMap::new(),
            clips: BTreeMap::new(),
            pool_size: pool_size.max(1),
            skeleton_cache_limit: skeleton_cache_limit.max(1),
            clip_cache_limit: clip_cache_limit.max(1),
            diagnostic_limit: diagnostic_limit.max(1),
            access_sequence: 0,
            stats: AnimationClipEvaluatorStats::default(),
            resource_events: None,
            reported_diagnostics: BTreeSet::new(),
            diagnostic_order: VecDeque::new(),
            pending_diagnostics: Vec::new(),
        }
    }

    pub fn stats(&self) -> AnimationClipEvaluatorStats {
        AnimationClipEvaluatorStats {
            cached_skeleton_count: self.skeletons.len(),
            cached_clip_count: self.clips.len(),
            ..self.stats
        }
    }

    pub fn for_resources(resources: &ResourceManager) -> Self {
        let mut evaluator = Self::default();
        evaluator.bind_resources(resources);
        evaluator
    }

    pub fn bind_resources(&mut self, resources: &ResourceManager) {
        if self.resource_events.is_none() {
            self.resource_events = Some(resources.subscribe());
        }
    }

    pub fn bind_pose(&self, skeleton_id: ResourceId) -> Option<&[AnimationPoseBone]> {
        self.skeletons
            .get(&skeleton_id)
            .map(|cached| cached.bind_pose.as_ref())
    }

    pub(crate) fn target_table(&self, skeleton_id: ResourceId) -> Option<Arc<SkeletonTargetTable>> {
        self.skeletons
            .get(&skeleton_id)
            .map(|cached| Arc::clone(&cached.targets))
    }

    pub(super) fn invalidate_changed_resources(&mut self) {
        let Some(events) = self.resource_events.as_ref() else {
            return;
        };
        let events = events.clone();
        for event in events.try_iter() {
            match event.resource_kind {
                ResourceKind::AnimationSkeleton => {
                    self.skeletons.remove(&event.id);
                    self.clips
                        .retain(|(skeleton_id, _), _| *skeleton_id != event.id);
                    self.remove_diagnostics_for_resource(event.id);
                }
                ResourceKind::AnimationClip => {
                    self.clips.retain(|(_, clip_id), _| *clip_id != event.id);
                    self.remove_diagnostics_for_resource(event.id);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::VecDeque;
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    #[test]
    fn optimization_batch_20260830ce_resource_events_drain_without_intermediate_vec() {
        let source = include_str!("animation_clip_evaluator.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        assert!(production.contains("let events = events.clone();"));
        assert!(production.contains("for event in events.try_iter()"));
        assert!(!production.contains("events.try_iter().collect::<Vec<_>>()"));
    }

    #[test]
    fn optimization_batch_20260830ce_resource_event_drain_preserves_all_kinds() {
        let source = include_str!("animation_clip_evaluator.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");

        for kind in [
            "ResourceKind::AnimationSkeleton",
            "ResourceKind::AnimationClip",
        ] {
            assert!(production.contains(kind));
        }
    }

    #[test]
    #[ignore = "Release-only Runtime170 performance contract"]
    fn optimization_batch_20260830ce_resource_event_direct_drain_p95() {
        const EVENT_COUNT: u64 = 65_536;
        const SAMPLES: usize = 17;
        let events = (0..EVENT_COUNT).collect::<VecDeque<_>>();
        let mut baseline_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for sample in 0..SAMPLES {
            let baseline = || {
                let mut input = events.clone();
                let started = Instant::now();
                let drained = input.drain(..).collect::<Vec<_>>();
                black_box(drained.into_iter().fold(0_u64, u64::wrapping_add));
                started.elapsed().as_nanos()
            };
            let optimized = || {
                let mut input = events.clone();
                let started = Instant::now();
                black_box(input.drain(..).fold(0_u64, u64::wrapping_add));
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
            "RUNTIME170_EVENT_DRAIN_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(75),
            "expected direct event drain to reduce P95 by at least 25%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
    }
}
