use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crossbeam_channel::Receiver;
use zircon_runtime::core::framework::animation::AnimationPoseBone;
use zircon_runtime::core::resource::{ResourceEvent, ResourceId, ResourceKind, ResourceManager};

use super::cache::{CachedClip, CachedSkeleton};
use super::cache_policy::{
    DEFAULT_CLIP_CACHE_LIMIT, DEFAULT_DIAGNOSTIC_LIMIT, DEFAULT_SKELETON_CACHE_LIMIT,
};
use super::AnimationClipEvaluatorStats;
use crate::AnimationEvaluationDiagnostic;

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

    pub(super) fn invalidate_changed_resources(&mut self) {
        let Some(events) = self.resource_events.as_ref() else {
            return;
        };
        let events = events.try_iter().collect::<Vec<_>>();
        for event in events {
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
