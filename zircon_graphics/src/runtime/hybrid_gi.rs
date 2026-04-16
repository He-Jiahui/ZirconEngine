use std::collections::{BTreeMap, BTreeSet};

use zircon_scene::RenderHybridGiExtract;

use crate::{
    types::{HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiPrepareUpdateRequest},
    VisibilityHybridGiFeedback, VisibilityHybridGiUpdatePlan,
};

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HybridGiProbeResidencyState {
    Resident,
    PendingUpdate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HybridGiProbeUpdateRequest {
    pub(crate) probe_id: u32,
    pub(crate) ray_budget: u32,
    pub(crate) generation: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeSnapshot {
    pub(crate) cache_entry_count: usize,
    pub(crate) resident_probe_count: usize,
    pub(crate) pending_update_count: usize,
    pub(crate) scheduled_trace_region_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiRuntimeState {
    probe_budget: usize,
    probe_ray_budgets: BTreeMap<u32, u32>,
    probe_irradiance_rgb: BTreeMap<u32, [u8; 3]>,
    resident_slots: BTreeMap<u32, u32>,
    pending_updates: Vec<HybridGiProbeUpdateRequest>,
    pending_probes: BTreeSet<u32>,
    scheduled_trace_regions: Vec<u32>,
    evictable_probes: Vec<u32>,
    free_slots: BTreeSet<u32>,
    next_slot: u32,
}

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
            self.probe_irradiance_rgb
                .entry(probe.probe_id)
                .or_insert_with(|| default_probe_irradiance_rgb(probe.ray_budget));
            if probe.resident {
                self.promote_to_resident(probe.probe_id);
            }
        }
    }

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

    pub(crate) fn consume_feedback(&mut self, feedback: &VisibilityHybridGiFeedback) {
        self.scheduled_trace_regions = feedback.scheduled_trace_region_ids.clone();
        self.complete_pending_probes(
            feedback.requested_probe_ids.iter().copied(),
            &feedback.evictable_probe_ids,
        );
    }

    pub(crate) fn build_prepare_frame(&self) -> HybridGiPrepareFrame {
        HybridGiPrepareFrame {
            resident_probes: self
                .resident_slots
                .iter()
                .map(|(&probe_id, &slot)| HybridGiPrepareProbe {
                    probe_id,
                    slot,
                    ray_budget: self
                        .probe_ray_budgets
                        .get(&probe_id)
                        .copied()
                        .unwrap_or_default(),
                    irradiance_rgb: self
                        .probe_irradiance_rgb
                        .get(&probe_id)
                        .copied()
                        .unwrap_or_else(|| {
                            default_probe_irradiance_rgb(
                                self.probe_ray_budgets
                                    .get(&probe_id)
                                    .copied()
                                    .unwrap_or_default(),
                            )
                        }),
                })
                .collect(),
            pending_updates: self
                .pending_updates
                .iter()
                .map(|update| HybridGiPrepareUpdateRequest {
                    probe_id: update.probe_id,
                    ray_budget: update.ray_budget,
                    generation: update.generation,
                })
                .collect(),
            scheduled_trace_region_ids: self.scheduled_trace_regions.clone(),
            evictable_probe_ids: self.evictable_probes.clone(),
        }
    }

    pub(crate) fn complete_gpu_updates(
        &mut self,
        probe_ids: impl IntoIterator<Item = u32>,
        trace_region_ids: impl IntoIterator<Item = u32>,
        evictable_probe_ids: &[u32],
    ) {
        self.scheduled_trace_regions = trace_region_ids.into_iter().collect();
        self.complete_pending_probes(probe_ids, evictable_probe_ids);
    }

    #[cfg(test)]
    pub(crate) fn probe_slot(&self, probe_id: u32) -> Option<u32> {
        self.resident_slots.get(&probe_id).copied()
    }

    #[cfg(test)]
    pub(crate) fn probe_residency(&self, probe_id: u32) -> Option<HybridGiProbeResidencyState> {
        if self.resident_slots.contains_key(&probe_id) {
            return Some(HybridGiProbeResidencyState::Resident);
        }
        if self.pending_probes.contains(&probe_id) {
            return Some(HybridGiProbeResidencyState::PendingUpdate);
        }
        None
    }

    #[cfg(test)]
    pub(crate) fn pending_updates(&self) -> Vec<HybridGiProbeUpdateRequest> {
        self.pending_updates.clone()
    }

    #[cfg(test)]
    pub(crate) fn scheduled_trace_regions(&self) -> Vec<u32> {
        self.scheduled_trace_regions.clone()
    }

    #[cfg(test)]
    pub(crate) fn evictable_probes(&self) -> Vec<u32> {
        self.evictable_probes.clone()
    }

    #[cfg(test)]
    pub(crate) fn apply_evictions(&mut self, probe_ids: impl IntoIterator<Item = u32>) {
        for probe_id in probe_ids {
            if let Some(slot) = self.resident_slots.remove(&probe_id) {
                self.free_slots.insert(slot);
            }
        }
        self.evictable_probes
            .retain(|probe_id| self.resident_slots.contains_key(probe_id));
    }

    #[cfg(test)]
    pub(crate) fn fulfill_updates(&mut self, probe_ids: impl IntoIterator<Item = u32>) {
        for probe_id in probe_ids {
            if !self.pending_probes.remove(&probe_id) {
                continue;
            }

            self.pending_updates
                .retain(|update| update.probe_id != probe_id);
            self.promote_to_resident(probe_id);
        }
    }

    pub(crate) fn snapshot(&self) -> HybridGiRuntimeSnapshot {
        HybridGiRuntimeSnapshot {
            cache_entry_count: self.resident_slots.len(),
            resident_probe_count: self.resident_slots.len(),
            pending_update_count: self.pending_updates.len(),
            scheduled_trace_region_count: self.scheduled_trace_regions.len(),
        }
    }

    fn promote_to_resident(&mut self, probe_id: u32) {
        if self.resident_slots.contains_key(&probe_id) {
            return;
        }

        self.pending_probes.remove(&probe_id);
        self.pending_updates
            .retain(|update| update.probe_id != probe_id);

        let slot = self.take_free_slot().unwrap_or_else(|| {
            let slot = self.next_slot;
            self.next_slot += 1;
            slot
        });
        self.resident_slots.insert(probe_id, slot);
    }

    fn evict_one(&mut self, probe_ids: impl IntoIterator<Item = u32>) -> bool {
        for probe_id in probe_ids {
            if let Some(slot) = self.resident_slots.remove(&probe_id) {
                self.free_slots.insert(slot);
                self.evictable_probes
                    .retain(|candidate| *candidate != probe_id);
                return true;
            }
        }
        false
    }

    fn take_free_slot(&mut self) -> Option<u32> {
        let slot = self.free_slots.iter().next().copied()?;
        self.free_slots.remove(&slot);
        Some(slot)
    }

    fn complete_pending_probes(
        &mut self,
        probe_ids: impl IntoIterator<Item = u32>,
        evictable_probe_ids: &[u32],
    ) {
        if self.probe_budget == 0 {
            return;
        }

        let requested_probe_ids = probe_ids
            .into_iter()
            .filter(|probe_id| self.pending_probes.contains(probe_id))
            .take(self.probe_budget)
            .collect::<Vec<_>>();

        for probe_id in requested_probe_ids {
            while self.resident_slots.len() >= self.probe_budget {
                if !self.evict_one(evictable_probe_ids.iter().copied()) {
                    self.evictable_probes
                        .retain(|candidate| self.resident_slots.contains_key(candidate));
                    return;
                }
            }

            self.promote_to_resident(probe_id);
        }

        self.evictable_probes
            .retain(|candidate| self.resident_slots.contains_key(candidate));
    }
}

fn default_probe_irradiance_rgb(ray_budget: u32) -> [u8; 3] {
    let budget = ray_budget.max(1).min(255) as u8;
    [
        48u8.saturating_add(budget / 2),
        64u8.saturating_add(budget / 2),
        80u8.saturating_add(budget / 2),
    ]
}
