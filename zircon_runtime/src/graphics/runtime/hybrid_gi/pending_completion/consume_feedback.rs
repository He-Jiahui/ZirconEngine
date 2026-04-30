use crate::VisibilityHybridGiFeedback;

use super::super::HybridGiRuntimeState;
use super::complete_pending_probes::complete_pending_probes;

impl HybridGiRuntimeState {
    pub(crate) fn consume_feedback(&mut self, feedback: &VisibilityHybridGiFeedback) {
        let current_requested_probe_ids = feedback
            .requested_probe_ids
            .iter()
            .copied()
            .filter(|probe_id| {
                self.probe_scene_data().contains_key(probe_id)
                    && !self.has_resident_probe(*probe_id)
            })
            .collect();
        self.replace_current_requested_probe_ids(current_requested_probe_ids);
        self.assign_scheduled_trace_regions(feedback.scheduled_trace_region_ids.iter().copied());
        self.refresh_recent_lineage_trace_support();
        complete_pending_probes(
            self,
            feedback.requested_probe_ids.iter().copied(),
            &feedback.evictable_probe_ids,
        );
    }
}
