use super::gpu_scene::GpuScene;
use super::layout::GPU_INSTANCE_DATA_STRIDE;
use super::update_queue::GpuSceneUpdateQueue;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuScenePrevTransformRollReport {
    pub(crate) live_instance_count: u32,
    pub(crate) visited_entry_count: u32,
    pub(crate) rolled_instance_count: u32,
    pub(crate) dirty_instance_range_count: usize,
}

impl GpuScene {
    pub(crate) fn previous_world_from_local(
        &self,
        entry: super::gpu_scene::GpuSceneEntry,
    ) -> Option<[[f32; 4]; 4]> {
        if !entry.has_rolled_previous_transform {
            return None;
        }
        self.instance_shadow
            .get(entry.first_instance_index as usize)
            .map(|instance| instance.prev_world_from_local)
    }

    pub(crate) fn roll_prev_transforms_after_success(&mut self) -> GpuScenePrevTransformRollReport {
        let live_instance_count = self.stats().instance_count;
        let mut pending = std::mem::take(&mut self.pending_prev_transform_rolls);
        roll_prev_transforms_for_keys(
            &mut self.entries,
            &mut self.instance_shadow,
            &mut pending,
            &mut self.updates,
            live_instance_count,
        )
    }

    #[cfg(test)]
    pub(crate) fn debug_previous_world_from_local_at(&self, instance_index: u32) -> [[f32; 4]; 4] {
        self.instance_shadow[instance_index as usize].prev_world_from_local
    }

    #[cfg(test)]
    pub(crate) fn debug_dirty_instance_upload_byte_count(&mut self) -> u64 {
        self.updates
            .drain_instance_upload_ranges(GPU_INSTANCE_DATA_STRIDE as u64)
            .into_iter()
            .map(|range| range.byte_len)
            .sum()
    }
}

fn roll_prev_transforms_for_keys(
    entries: &mut HashMap<u64, super::gpu_scene::GpuSceneEntry>,
    instance_shadow: &mut [super::layout::GpuInstanceData],
    pending: &mut HashSet<u64>,
    updates: &mut GpuSceneUpdateQueue,
    live_instance_count: u32,
) -> GpuScenePrevTransformRollReport {
    let mut report = GpuScenePrevTransformRollReport {
        live_instance_count,
        ..GpuScenePrevTransformRollReport::default()
    };

    for stable_instance_key in pending.drain() {
        let Some(entry) = entries.get_mut(&stable_instance_key) else {
            continue;
        };
        report.visited_entry_count += 1;
        {
            let start = entry.first_instance_index;
            let end = start
                .checked_add(entry.instance_count)
                .expect("gpu scene instance span overflowed during prev transform roll");
            let mut span_changed = false;
            for instance_index in start..end {
                let instance = &mut instance_shadow[instance_index as usize];
                if instance.prev_world_from_local != instance.world_from_local {
                    instance.prev_world_from_local = instance.world_from_local;
                    span_changed = true;
                    report.rolled_instance_count += 1;
                }
            }
            entry.has_rolled_previous_transform = true;
            if span_changed {
                updates.mark_instances(start, entry.instance_count);
                report.dirty_instance_range_count += 1;
            }
        }
    }

    report
}

#[cfg(test)]
#[path = "tests/prev_transform.rs"]
mod tests;
