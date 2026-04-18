#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPrepareRequest {
    pub(crate) page_id: u32,
    pub(crate) size_bytes: u64,
    pub(crate) generation: u64,
    pub(crate) frontier_rank: u32,
    pub(crate) assigned_slot: Option<u32>,
    pub(crate) recycled_page_id: Option<u32>,
}
