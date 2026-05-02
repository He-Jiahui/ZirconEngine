use std::collections::BTreeMap;

use zircon_runtime::core::math::Vec3;

use super::{
    representation::HybridGiCardDescriptor, surface_cache_state::HybridGiSurfaceCacheState,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::hybrid_gi::scene_representation) struct HybridGiScreenProbeDescriptor {
    probe_id: u32,
    card_id: u32,
    surface_page_id: Option<u32>,
    bounds_center: Vec3,
    bounds_radius: f32,
    ray_budget: u32,
}

impl HybridGiScreenProbeDescriptor {
    pub(in crate::hybrid_gi::scene_representation) fn probe_id(&self) -> u32 {
        self.probe_id
    }

    pub(in crate::hybrid_gi::scene_representation) fn card_id(&self) -> u32 {
        self.card_id
    }

    pub(in crate::hybrid_gi::scene_representation) fn surface_page_id(&self) -> Option<u32> {
        self.surface_page_id
    }

    pub(in crate::hybrid_gi::scene_representation) fn bounds_center(&self) -> Vec3 {
        self.bounds_center
    }

    pub(in crate::hybrid_gi::scene_representation) fn bounds_radius(&self) -> f32 {
        self.bounds_radius
    }

    pub(in crate::hybrid_gi::scene_representation) fn ray_budget(&self) -> u32 {
        self.ray_budget
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::hybrid_gi::scene_representation) struct HybridGiScreenProbeState {
    probes: Vec<HybridGiScreenProbeDescriptor>,
}

impl HybridGiScreenProbeState {
    pub(in crate::hybrid_gi::scene_representation) fn synchronize(
        &mut self,
        cards: &[HybridGiCardDescriptor],
        surface_cache: &HybridGiSurfaceCacheState,
        trace_budget: usize,
    ) {
        if trace_budget == 0 || cards.is_empty() {
            self.probes.clear();
            return;
        }

        let surface_page_ids_by_card_id = surface_cache
            .owner_card_ids_by_page_id_snapshot()
            .into_iter()
            .map(|(page_id, card_id)| (card_id, page_id))
            .collect::<BTreeMap<_, _>>();
        let probe_count = trace_budget.min(cards.len());
        let ray_budget = ray_budget_per_probe(trace_budget, probe_count);

        self.probes = cards
            .iter()
            .take(probe_count)
            .enumerate()
            .map(|(probe_index, card)| HybridGiScreenProbeDescriptor {
                probe_id: probe_index as u32,
                card_id: card.card_id(),
                surface_page_id: surface_page_ids_by_card_id.get(&card.card_id()).copied(),
                bounds_center: card.bounds_center(),
                bounds_radius: card.bounds_radius(),
                ray_budget,
            })
            .collect();
    }

    pub(in crate::hybrid_gi::scene_representation) fn probe_count(&self) -> usize {
        self.probes.len()
    }

    pub(in crate::hybrid_gi::scene_representation) fn descriptors(
        &self,
    ) -> &[HybridGiScreenProbeDescriptor] {
        &self.probes
    }
}

fn ray_budget_per_probe(trace_budget: usize, probe_count: usize) -> u32 {
    if probe_count == 0 {
        return 0;
    }
    ((trace_budget as u32) / (probe_count as u32)).max(1)
}
