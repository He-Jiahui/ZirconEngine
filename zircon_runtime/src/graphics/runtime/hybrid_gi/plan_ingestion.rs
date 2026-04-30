use std::collections::BTreeSet;

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

            if self.has_resident_probe(probe_id) || self.has_pending_probe(probe_id) {
                continue;
            }

            let ray_budget = self
                .probe_ray_budgets()
                .get(&probe_id)
                .copied()
                .unwrap_or_default();
            self.insert_pending_probe(probe_id);
            self.push_pending_update_request(HybridGiProbeUpdateRequest::new(
                probe_id, ray_budget, generation,
            ));
        }

        let current_requested_probe_ids = if self.scene_representation_owns_runtime() {
            BTreeSet::new()
        } else {
            plan.requested_probe_ids
                .iter()
                .copied()
                .filter(|probe_id| {
                    self.probe_scene_data().contains_key(probe_id)
                        && !self.has_resident_probe(*probe_id)
                })
                .collect()
        };
        self.replace_current_requested_probe_ids(current_requested_probe_ids);
        self.assign_scheduled_trace_regions(plan.scheduled_trace_region_ids.iter().copied());
        self.refresh_recent_lineage_trace_support();
        let evictable_probes = plan
            .evictable_probe_ids
            .iter()
            .copied()
            .filter(|probe_id| self.has_resident_probe(*probe_id))
            .collect();
        self.replace_evictable_probes(evictable_probes);
    }

    fn has_live_probe_payload(&self, probe_id: u32) -> bool {
        !self.scene_representation_owns_runtime() && self.probe_scene_data().contains_key(&probe_id)
    }
}
