use crate::VisibilityHybridGiUpdatePlan;

use super::{HybridGiProbeUpdateRequest, HybridGiRuntimeState};

impl HybridGiRuntimeState {
    pub(crate) fn ingest_plan(&mut self, generation: u64, plan: &VisibilityHybridGiUpdatePlan) {
        for &probe_id in &plan.resident_probe_ids {
            if !self.has_live_probe_payload(probe_id) {
                continue;
            }

            self.promote_to_resident(probe_id);
        }

        for &probe_id in &plan.dirty_requested_probe_ids {
            if !self.has_live_probe_payload(probe_id) {
                continue;
            }

            if self.resident_slots.contains_key(&probe_id)
                || self.pending_probes.contains(&probe_id)
            {
                continue;
            }

            self.pending_probes.insert(probe_id);
            self.pending_updates.push(HybridGiProbeUpdateRequest {
                probe_id,
                ray_budget: self
                    .probe_ray_budgets
                    .get(&probe_id)
                    .copied()
                    .unwrap_or_default(),
                generation,
            });
        }

        self.current_requested_probe_ids = plan
            .requested_probe_ids
            .iter()
            .copied()
            .filter(|probe_id| {
                self.probe_scene_data.contains_key(probe_id)
                    && !self.resident_slots.contains_key(probe_id)
            })
            .collect();
        self.assign_scheduled_trace_regions(plan.scheduled_trace_region_ids.iter().copied());
        self.refresh_recent_lineage_trace_support();
        self.evictable_probes = plan
            .evictable_probe_ids
            .iter()
            .copied()
            .filter(|probe_id| self.resident_slots.contains_key(probe_id))
            .collect();
    }

    fn has_live_probe_payload(&self, probe_id: u32) -> bool {
        self.probe_scene_data.contains_key(&probe_id)
    }
}
