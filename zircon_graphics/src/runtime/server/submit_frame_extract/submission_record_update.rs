use zircon_render_server::FrameHistoryHandle;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct HybridGiStatSnapshot {
    pub(super) cache_entry_count: usize,
    pub(super) resident_probe_count: usize,
    pub(super) pending_update_count: usize,
    pub(super) scheduled_trace_region_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct VirtualGeometryStatSnapshot {
    pub(super) page_table_entry_count: usize,
    pub(super) resident_page_count: usize,
    pub(super) pending_request_count: usize,
    pub(super) completed_page_count: usize,
    pub(super) indirect_draw_count: usize,
    pub(super) indirect_segment_count: usize,
}

pub(super) struct SubmissionRecordUpdate {
    pub(super) history_handle: FrameHistoryHandle,
    pub(super) previous_handle: Option<FrameHistoryHandle>,
    pub(super) hybrid_gi_stats: HybridGiStatSnapshot,
    pub(super) virtual_geometry_stats: VirtualGeometryStatSnapshot,
}
