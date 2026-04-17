use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct VirtualGeometryUploaderParams {
    pub(super) pending_count: u32,
    pub(super) available_slot_count: u32,
    pub(super) evictable_count: u32,
    pub(super) page_budget: u32,
    pub(super) streaming_budget_bytes: u32,
    pub(super) reclaimable_bytes: u32,
    pub(super) resident_count: u32,
    pub(super) _padding1: u32,
}
