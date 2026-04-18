use std::collections::BTreeSet;

use zircon_scene::RenderHybridGiExtract;

use super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn register_extract(&mut self, extract: Option<&RenderHybridGiExtract>) {
        self.evictable_probes.clear();
        self.scheduled_trace_regions.clear();

        let Some(extract) = extract else {
            *self = Self::default();
            return;
        };

        let live_probe_ids = extract
            .probes
            .iter()
            .map(|probe| probe.probe_id)
            .collect::<BTreeSet<_>>();
        let stale_resident_probe_ids = self
            .resident_slots
            .keys()
            .copied()
            .filter(|probe_id| !live_probe_ids.contains(probe_id))
            .collect::<Vec<_>>();
        for probe_id in stale_resident_probe_ids {
            self.evict_one([probe_id]);
        }
        self.pending_probes
            .retain(|probe_id| live_probe_ids.contains(probe_id));
        self.pending_updates
            .retain(|update| live_probe_ids.contains(&update.probe_id));
        self.probe_ray_budgets
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));
        self.probe_irradiance_rgb
            .retain(|probe_id, _| live_probe_ids.contains(probe_id));

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
