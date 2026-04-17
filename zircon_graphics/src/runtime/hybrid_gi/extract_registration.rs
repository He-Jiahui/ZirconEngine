use zircon_scene::RenderHybridGiExtract;

use super::hybrid_gi_runtime_state::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn register_extract(&mut self, extract: Option<&RenderHybridGiExtract>) {
        self.evictable_probes.clear();
        self.scheduled_trace_regions.clear();

        let Some(extract) = extract else {
            self.probe_budget = 0;
            return;
        };

        self.probe_budget = (extract.probe_budget as usize)
            .max(extract.probes.iter().filter(|probe| probe.resident).count());

        for probe in &extract.probes {
            self.probe_ray_budgets
                .insert(probe.probe_id, probe.ray_budget);
            if probe.resident {
                self.promote_to_resident(probe.probe_id);
            }
        }
    }
}
