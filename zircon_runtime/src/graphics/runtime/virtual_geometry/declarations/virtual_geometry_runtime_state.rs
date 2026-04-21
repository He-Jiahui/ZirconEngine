use std::collections::{BTreeMap, BTreeSet};

use super::virtual_geometry_page_request::VirtualGeometryPageRequest;

pub(crate) const HOT_FRONTIER_COOLING_FRAME_COUNT: u8 = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry) page_budget: usize,
    pub(in crate::graphics::runtime::virtual_geometry) page_sizes: BTreeMap<u32, u64>,
    pub(in crate::graphics::runtime::virtual_geometry) page_parent_pages: BTreeMap<u32, u32>,
    pub(in crate::graphics::runtime::virtual_geometry) current_requested_page_order:
        BTreeMap<u32, usize>,
    pub(in crate::graphics::runtime::virtual_geometry) current_hot_resident_pages: BTreeSet<u32>,
    pub(in crate::graphics::runtime::virtual_geometry) recent_hot_resident_pages: BTreeMap<u32, u8>,
    pub(in crate::graphics::runtime::virtual_geometry) resident_slots: BTreeMap<u32, u32>,
    pub(in crate::graphics::runtime::virtual_geometry) pending_requests:
        Vec<VirtualGeometryPageRequest>,
    pub(in crate::graphics::runtime::virtual_geometry) pending_pages: BTreeSet<u32>,
    pub(in crate::graphics::runtime::virtual_geometry) evictable_pages: Vec<u32>,
    pub(in crate::graphics::runtime::virtual_geometry) free_slots: BTreeSet<u32>,
    pub(in crate::graphics::runtime::virtual_geometry) next_slot: u32,
}
