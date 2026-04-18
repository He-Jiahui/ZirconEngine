use crate::VisibilityHybridGiFeedback;

use super::super::HybridGiRuntimeState;
use super::complete_pending_probes::complete_pending_probes;

impl HybridGiRuntimeState {
    pub(crate) fn consume_feedback(&mut self, feedback: &VisibilityHybridGiFeedback) {
        self.scheduled_trace_regions = feedback.scheduled_trace_region_ids.clone();
        complete_pending_probes(
            self,
            feedback.requested_probe_ids.iter().copied(),
            &feedback.evictable_probe_ids,
        );
    }
}
