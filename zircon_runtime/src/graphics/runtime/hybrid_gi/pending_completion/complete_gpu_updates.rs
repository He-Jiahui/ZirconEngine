use super::super::HybridGiRuntimeState;
use super::complete_pending_probes::complete_pending_probes;

impl HybridGiRuntimeState {
    pub(crate) fn complete_gpu_updates(
        &mut self,
        probe_ids: impl IntoIterator<Item = u32>,
        trace_region_ids: impl IntoIterator<Item = u32>,
        probe_irradiance_rgb: &[(u32, [u8; 3])],
        probe_trace_lighting_rgb: &[(u32, [u8; 3])],
        evictable_probe_ids: &[u32],
    ) {
        for (probe_id, irradiance_rgb) in probe_irradiance_rgb {
            if !self.probe_scene_data.contains_key(probe_id) {
                continue;
            }

            self.probe_irradiance_rgb.insert(*probe_id, *irradiance_rgb);
        }
        for (probe_id, trace_lighting_rgb) in probe_trace_lighting_rgb {
            if !self.probe_scene_data.contains_key(probe_id) {
                continue;
            }

            self.probe_rt_lighting_rgb
                .insert(*probe_id, *trace_lighting_rgb);
        }
        self.current_requested_probe_ids
            .retain(|probe_id| self.pending_probes.contains(probe_id));
        self.assign_scheduled_trace_regions(trace_region_ids);
        self.refresh_recent_lineage_trace_support();
        complete_pending_probes(self, probe_ids, evictable_probe_ids);
    }
}
