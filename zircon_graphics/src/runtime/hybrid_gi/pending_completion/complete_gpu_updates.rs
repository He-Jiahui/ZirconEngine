use super::super::HybridGiRuntimeState;
use super::complete_pending_probes::complete_pending_probes;

impl HybridGiRuntimeState {
    pub(crate) fn complete_gpu_updates(
        &mut self,
        probe_ids: impl IntoIterator<Item = u32>,
        trace_region_ids: impl IntoIterator<Item = u32>,
        probe_irradiance_rgb: &[(u32, [u8; 3])],
        evictable_probe_ids: &[u32],
    ) {
        for (probe_id, irradiance_rgb) in probe_irradiance_rgb {
            self.probe_irradiance_rgb.insert(*probe_id, *irradiance_rgb);
        }
        self.scheduled_trace_regions = trace_region_ids.into_iter().collect();
        complete_pending_probes(self, probe_ids, evictable_probe_ids);
    }
}
