#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct HostInvalidationDiagnostics {
    pub slow_path_rebuild_count: u64,
    pub render_rebuild_count: u64,
    pub paint_only_request_count: u64,
}
