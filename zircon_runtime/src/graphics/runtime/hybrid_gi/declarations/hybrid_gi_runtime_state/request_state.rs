use std::collections::{BTreeMap, BTreeSet};

use super::super::hybrid_gi_probe_update_request::HybridGiProbeUpdateRequest;
use super::runtime_state::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::graphics::runtime::hybrid_gi) fn current_requested_probe_ids(
        &self,
    ) -> &BTreeSet<u32> {
        &self.current_requested_probe_ids
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn current_requested_probe_ids_mut(
        &mut self,
    ) -> &mut BTreeSet<u32> {
        &mut self.current_requested_probe_ids
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn replace_current_requested_probe_ids(
        &mut self,
        current_requested_probe_ids: BTreeSet<u32>,
    ) {
        self.current_requested_probe_ids = current_requested_probe_ids;
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn scheduled_trace_region_ids(&self) -> &[u32] {
        &self.scheduled_trace_regions
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn clear_scheduled_trace_regions(&mut self) {
        self.scheduled_trace_regions.clear();
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn replace_scheduled_trace_regions(
        &mut self,
        scheduled_trace_regions: Vec<u32>,
    ) {
        self.scheduled_trace_regions = scheduled_trace_regions;
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn recent_lineage_trace_support_q8(
        &self,
    ) -> &BTreeMap<u32, u16> {
        &self.recent_lineage_trace_support_q8
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn recent_lineage_trace_support_q8_mut(
        &mut self,
    ) -> &mut BTreeMap<u32, u16> {
        &mut self.recent_lineage_trace_support_q8
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn recent_requested_lineage_support_q8(
        &self,
    ) -> &BTreeMap<u32, u16> {
        &self.recent_requested_lineage_support_q8
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn recent_requested_lineage_support_q8_mut(
        &mut self,
    ) -> &mut BTreeMap<u32, u16> {
        &mut self.recent_requested_lineage_support_q8
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn pending_update_count(&self) -> usize {
        self.pending_updates.len()
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn pending_update_requests(
        &self,
    ) -> &[HybridGiProbeUpdateRequest] {
        &self.pending_updates
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn push_pending_update_request(
        &mut self,
        update: HybridGiProbeUpdateRequest,
    ) {
        self.pending_updates.push(update);
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn retain_pending_update_requests(
        &mut self,
        mut retain: impl FnMut(&HybridGiProbeUpdateRequest) -> bool,
    ) {
        self.pending_updates.retain(|update| retain(update));
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn has_pending_probe(&self, probe_id: u32) -> bool {
        self.pending_probes.contains(&probe_id)
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn pending_probe_ids(
        &self,
    ) -> impl Iterator<Item = u32> + '_ {
        self.pending_probes.iter().copied()
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn insert_pending_probe(
        &mut self,
        probe_id: u32,
    ) -> bool {
        self.pending_probes.insert(probe_id)
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn remove_pending_probe(
        &mut self,
        probe_id: u32,
    ) -> bool {
        self.pending_probes.remove(&probe_id)
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn retain_pending_probes(
        &mut self,
        mut retain: impl FnMut(&u32) -> bool,
    ) {
        self.pending_probes.retain(|probe_id| retain(probe_id));
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn evictable_probe_ids(&self) -> &[u32] {
        &self.evictable_probes
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn clear_evictable_probes(&mut self) {
        self.evictable_probes.clear();
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn replace_evictable_probes(
        &mut self,
        evictable_probes: Vec<u32>,
    ) {
        self.evictable_probes = evictable_probes;
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn retain_evictable_probes(
        &mut self,
        mut retain: impl FnMut(&u32) -> bool,
    ) {
        self.evictable_probes.retain(|probe_id| retain(probe_id));
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn remove_evictable_probe(
        &mut self,
        probe_id: u32,
    ) {
        self.retain_evictable_probes(|candidate| *candidate != probe_id);
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn retain_resident_evictable_probes(&mut self) {
        let resident_probe_ids = self.resident_probe_ids().collect::<BTreeSet<_>>();
        self.retain_evictable_probes(|probe_id| resident_probe_ids.contains(probe_id));
    }
}
