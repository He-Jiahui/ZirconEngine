use crate::VisibilityHybridGiUpdatePlan;

use super::{HybridGiProbeUpdateRequest, HybridGiRuntimeState};

impl HybridGiRuntimeState {
    pub(crate) fn ingest_plan(&mut self, generation: u64, plan: &VisibilityHybridGiUpdatePlan) {
        for &probe_id in &plan.resident_probe_ids {
            self.promote_to_resident(probe_id);
        }

        for &probe_id in &plan.dirty_requested_probe_ids {
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

        self.scheduled_trace_regions = plan.scheduled_trace_region_ids.clone();
        self.evictable_probes = plan
            .evictable_probe_ids
            .iter()
            .copied()
            .filter(|probe_id| self.resident_slots.contains_key(probe_id))
            .collect();
    }
}
