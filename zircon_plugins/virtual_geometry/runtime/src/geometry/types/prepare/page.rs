#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualGeometryPreparePage {
    pub page_id: u32,
    pub slot: u32,
    pub size_bytes: u64,
}
