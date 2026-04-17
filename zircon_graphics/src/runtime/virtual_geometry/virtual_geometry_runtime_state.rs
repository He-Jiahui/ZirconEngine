use std::collections::{BTreeMap, BTreeSet};

use super::virtual_geometry_page_request::VirtualGeometryPageRequest;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryRuntimeState {
    pub(super) page_budget: usize,
    pub(super) page_sizes: BTreeMap<u32, u64>,
    pub(super) resident_slots: BTreeMap<u32, u32>,
    pub(super) pending_requests: Vec<VirtualGeometryPageRequest>,
    pub(super) pending_pages: BTreeSet<u32>,
    pub(super) evictable_pages: Vec<u32>,
    pub(super) free_slots: BTreeSet<u32>,
    pub(super) next_slot: u32,
}
