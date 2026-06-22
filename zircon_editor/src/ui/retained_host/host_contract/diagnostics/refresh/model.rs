mod counters;

use std::time::Instant;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostRefreshDiagnostics {
    pub present_count: u64,
    pub full_paint_count: u64,
    pub region_paint_count: u64,
    pub painted_pixel_count: u64,
    pub slow_path_rebuild_count: u64,
    pub render_rebuild_count: u64,
    pub paint_only_request_count: u64,
    pub(super) first_present_at: Option<Instant>,
    pub(super) last_present_at: Option<Instant>,
}

impl Default for HostRefreshDiagnostics {
    fn default() -> Self {
        Self {
            present_count: 0,
            full_paint_count: 0,
            region_paint_count: 0,
            painted_pixel_count: 0,
            slow_path_rebuild_count: 0,
            render_rebuild_count: 0,
            paint_only_request_count: 0,
            first_present_at: None,
            last_present_at: None,
        }
    }
}
