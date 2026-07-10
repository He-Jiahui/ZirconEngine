use zircon_runtime::core::resource::ResourceId;

use super::AnimationClipEvaluator;

pub(super) const DEFAULT_SKELETON_CACHE_LIMIT: usize = 64;
pub(super) const DEFAULT_CLIP_CACHE_LIMIT: usize = 256;
pub(super) const DEFAULT_DIAGNOSTIC_LIMIT: usize = 1_024;

impl AnimationClipEvaluator {
    pub(super) fn next_access_sequence(&mut self) -> u64 {
        self.access_sequence = self.access_sequence.saturating_add(1);
        self.access_sequence
    }

    pub(super) fn enforce_skeleton_cache_limit(&mut self) {
        while self.skeletons.len() > self.skeleton_cache_limit {
            let Some(evicted) = self
                .skeletons
                .iter()
                .min_by_key(|(id, cached)| (cached.last_used, **id))
                .map(|(id, _)| *id)
            else {
                break;
            };
            self.skeletons.remove(&evicted);
            self.clips
                .retain(|(skeleton_id, _), _| *skeleton_id != evicted);
            self.remove_diagnostics_for_resource(evicted);
            self.stats.skeleton_eviction_count =
                self.stats.skeleton_eviction_count.saturating_add(1);
        }
    }

    pub(super) fn enforce_clip_cache_limit(&mut self) {
        while self.clips.len() > self.clip_cache_limit {
            let Some(evicted) = self
                .clips
                .iter()
                .min_by_key(|(key, cached)| (cached.last_used, **key))
                .map(|(key, _)| *key)
            else {
                break;
            };
            self.clips.remove(&evicted);
            self.stats.clip_eviction_count = self.stats.clip_eviction_count.saturating_add(1);
        }
    }

    pub(super) fn remove_diagnostics_for_resource(&mut self, resource: ResourceId) {
        self.reported_diagnostics
            .retain(|(skeleton_id, _, clip_id, _, _)| {
                *skeleton_id != resource && *clip_id != resource
            });
        self.diagnostic_order
            .retain(|(skeleton_id, _, clip_id, _, _)| {
                *skeleton_id != resource && *clip_id != resource
            });
    }

    pub(super) fn enforce_diagnostic_limit(&mut self) {
        while self.reported_diagnostics.len() > self.diagnostic_limit {
            let Some(oldest) = self.diagnostic_order.pop_front() else {
                break;
            };
            self.reported_diagnostics.remove(&oldest);
        }
    }
}
