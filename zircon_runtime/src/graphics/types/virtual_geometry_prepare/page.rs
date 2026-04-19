#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VirtualGeometryPreparePage {
    pub(crate) page_id: u32,
    pub(crate) slot: u32,
    pub(crate) size_bytes: u64,
}
