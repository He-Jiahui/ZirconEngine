use crate::core::framework::render::RenderVirtualGeometryVisBuffer64Entry;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VirtualGeometryGpuReadback {
    pub(crate) page_table_entries: Vec<(u32, u32)>,
    pub(crate) completed_page_ids: Vec<u32>,
    pub(crate) completed_page_assignments: Vec<(u32, u32)>,
    pub(crate) completed_page_replacements: Vec<(u32, u32)>,
    pub(crate) visbuffer64_clear_value: u64,
    pub(crate) visbuffer64_entries: Vec<RenderVirtualGeometryVisBuffer64Entry>,
}
