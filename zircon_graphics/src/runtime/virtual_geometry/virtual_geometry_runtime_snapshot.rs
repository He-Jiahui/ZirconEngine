#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryRuntimeSnapshot {
    pub(crate) page_table_entry_count: usize,
    pub(crate) resident_page_count: usize,
    pub(crate) pending_request_count: usize,
}
