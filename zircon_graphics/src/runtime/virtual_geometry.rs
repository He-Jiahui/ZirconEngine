use std::collections::{BTreeMap, BTreeSet};

use zircon_scene::RenderVirtualGeometryExtract;

use crate::types::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState, VirtualGeometryPrepareFrame,
    VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
};
use crate::{
    VisibilityVirtualGeometryCluster, VisibilityVirtualGeometryFeedback,
    VisibilityVirtualGeometryPageUploadPlan,
};

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VirtualGeometryPageResidencyState {
    Resident,
    PendingUpload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPageRequest {
    pub(crate) page_id: u32,
    pub(crate) size_bytes: u64,
    pub(crate) generation: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryRuntimeSnapshot {
    pub(crate) page_table_entry_count: usize,
    pub(crate) resident_page_count: usize,
    pub(crate) pending_request_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryRuntimeState {
    page_budget: usize,
    page_sizes: BTreeMap<u32, u64>,
    resident_slots: BTreeMap<u32, u32>,
    pending_requests: Vec<VirtualGeometryPageRequest>,
    pending_pages: BTreeSet<u32>,
    evictable_pages: Vec<u32>,
    free_slots: BTreeSet<u32>,
    next_slot: u32,
}

impl VirtualGeometryRuntimeState {
    pub(crate) fn register_extract(&mut self, extract: Option<&RenderVirtualGeometryExtract>) {
        self.evictable_pages.clear();

        let Some(extract) = extract else {
            self.page_budget = 0;
            return;
        };

        self.page_budget = (extract.page_budget as usize)
            .max(extract.pages.iter().filter(|page| page.resident).count());

        for page in &extract.pages {
            self.page_sizes.insert(page.page_id, page.size_bytes);
            if page.resident {
                self.promote_to_resident(page.page_id);
            }
        }
    }

    pub(crate) fn ingest_plan(
        &mut self,
        generation: u64,
        plan: &VisibilityVirtualGeometryPageUploadPlan,
    ) {
        for &page_id in &plan.resident_pages {
            self.promote_to_resident(page_id);
        }

        for &page_id in &plan.dirty_requested_pages {
            if self.resident_slots.contains_key(&page_id) || self.pending_pages.contains(&page_id) {
                continue;
            }

            self.pending_pages.insert(page_id);
            self.pending_requests.push(VirtualGeometryPageRequest {
                page_id,
                size_bytes: self.page_sizes.get(&page_id).copied().unwrap_or_default(),
                generation,
            });
        }

        self.evictable_pages = plan
            .evictable_pages
            .iter()
            .copied()
            .filter(|page_id| self.resident_slots.contains_key(page_id))
            .collect();
    }

    pub(crate) fn consume_feedback(&mut self, feedback: &VisibilityVirtualGeometryFeedback) {
        self.complete_pending_pages(
            feedback.requested_pages.iter().copied(),
            &feedback.evictable_pages,
        );
    }

    pub(crate) fn complete_gpu_uploads(
        &mut self,
        page_ids: impl IntoIterator<Item = u32>,
        evictable_pages: &[u32],
    ) {
        self.complete_pending_pages(page_ids, evictable_pages);
    }

    pub(crate) fn build_prepare_frame(
        &self,
        visible_clusters: &[VisibilityVirtualGeometryCluster],
    ) -> VirtualGeometryPrepareFrame {
        let resident_pages = self
            .resident_slots
            .iter()
            .map(|(&page_id, &slot)| VirtualGeometryPreparePage {
                page_id,
                slot,
                size_bytes: self.page_sizes.get(&page_id).copied().unwrap_or_default(),
            })
            .collect::<Vec<_>>();
        let pending_page_requests = self
            .pending_requests
            .iter()
            .map(|request| VirtualGeometryPrepareRequest {
                page_id: request.page_id,
                size_bytes: request.size_bytes,
                generation: request.generation,
            })
            .collect::<Vec<_>>();
        let evictable_pages =
            self.evictable_pages
                .iter()
                .filter_map(|page_id| {
                    self.resident_slots.get(page_id).copied().map(|slot| {
                        VirtualGeometryPreparePage {
                            page_id: *page_id,
                            slot,
                            size_bytes: self.page_sizes.get(page_id).copied().unwrap_or_default(),
                        }
                    })
                })
                .collect::<Vec<_>>();
        let mut visible_entities = BTreeSet::new();
        let visible_clusters = visible_clusters
            .iter()
            .map(|cluster| {
                let resident_slot = self.resident_slots.get(&cluster.page_id).copied();
                let state = if resident_slot.is_some() {
                    VirtualGeometryPrepareClusterState::Resident
                } else if self.pending_pages.contains(&cluster.page_id) {
                    VirtualGeometryPrepareClusterState::PendingUpload
                } else {
                    VirtualGeometryPrepareClusterState::Missing
                };
                if !matches!(state, VirtualGeometryPrepareClusterState::Missing) {
                    visible_entities.insert(cluster.entity);
                }
                VirtualGeometryPrepareCluster {
                    entity: cluster.entity,
                    cluster_id: cluster.cluster_id,
                    page_id: cluster.page_id,
                    lod_level: cluster.lod_level,
                    resident_slot,
                    state,
                }
            })
            .collect::<Vec<_>>();

        VirtualGeometryPrepareFrame {
            visible_entities: visible_entities.into_iter().collect(),
            visible_clusters,
            resident_pages,
            pending_page_requests,
            evictable_pages,
        }
    }

    #[cfg(test)]
    pub(crate) fn page_slot(&self, page_id: u32) -> Option<u32> {
        self.resident_slots.get(&page_id).copied()
    }

    #[cfg(test)]
    pub(crate) fn page_residency(&self, page_id: u32) -> Option<VirtualGeometryPageResidencyState> {
        if self.resident_slots.contains_key(&page_id) {
            return Some(VirtualGeometryPageResidencyState::Resident);
        }
        if self.pending_pages.contains(&page_id) {
            return Some(VirtualGeometryPageResidencyState::PendingUpload);
        }
        None
    }

    #[cfg(test)]
    pub(crate) fn pending_requests(&self) -> Vec<VirtualGeometryPageRequest> {
        self.pending_requests.clone()
    }

    #[cfg(test)]
    pub(crate) fn evictable_pages(&self) -> Vec<u32> {
        self.evictable_pages.clone()
    }

    #[cfg(test)]
    pub(crate) fn apply_evictions(&mut self, page_ids: impl IntoIterator<Item = u32>) {
        for page_id in page_ids {
            if let Some(slot) = self.resident_slots.remove(&page_id) {
                self.free_slots.insert(slot);
            }
        }
        self.evictable_pages
            .retain(|page_id| self.resident_slots.contains_key(page_id));
    }

    #[cfg(test)]
    pub(crate) fn fulfill_requests(&mut self, page_ids: impl IntoIterator<Item = u32>) {
        for page_id in page_ids {
            if !self.pending_pages.remove(&page_id) {
                continue;
            }

            self.pending_requests
                .retain(|request| request.page_id != page_id);
            self.promote_to_resident(page_id);
        }
    }

    pub(crate) fn snapshot(&self) -> VirtualGeometryRuntimeSnapshot {
        VirtualGeometryRuntimeSnapshot {
            page_table_entry_count: self.resident_slots.len(),
            resident_page_count: self.resident_slots.len(),
            pending_request_count: self.pending_requests.len(),
        }
    }

    fn promote_to_resident(&mut self, page_id: u32) {
        if self.resident_slots.contains_key(&page_id) {
            return;
        }

        self.pending_pages.remove(&page_id);
        self.pending_requests
            .retain(|request| request.page_id != page_id);

        let slot = self.take_free_slot().unwrap_or_else(|| {
            let slot = self.next_slot;
            self.next_slot += 1;
            slot
        });
        self.resident_slots.insert(page_id, slot);
    }

    fn evict_one(&mut self, page_ids: impl IntoIterator<Item = u32>) -> bool {
        for page_id in page_ids {
            if let Some(slot) = self.resident_slots.remove(&page_id) {
                self.free_slots.insert(slot);
                self.evictable_pages
                    .retain(|candidate| *candidate != page_id);
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

    fn complete_pending_pages(
        &mut self,
        page_ids: impl IntoIterator<Item = u32>,
        evictable_pages: &[u32],
    ) {
        if self.page_budget == 0 {
            return;
        }

        let requested_pages = page_ids
            .into_iter()
            .filter(|page_id| self.pending_pages.contains(page_id))
            .take(self.page_budget)
            .collect::<Vec<_>>();

        for page_id in requested_pages {
            while self.resident_slots.len() >= self.page_budget {
                if !self.evict_one(evictable_pages.iter().copied()) {
                    self.evictable_pages
                        .retain(|candidate| self.resident_slots.contains_key(candidate));
                    return;
                }
            }

            self.promote_to_resident(page_id);
        }

        self.evictable_pages
            .retain(|candidate| self.resident_slots.contains_key(candidate));
    }
}
