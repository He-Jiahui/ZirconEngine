#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPrepareRequest {
    pub(crate) page_id: u32,
    pub(crate) size_bytes: u64,
    pub(crate) generation: u64,
}
