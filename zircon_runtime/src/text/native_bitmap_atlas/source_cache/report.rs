use crate::text::parallel::raster_pool::TextRasterWorkerPoolDiagnostics;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NativeBitmapAtlasSourceCacheFrameReport {
    pub(crate) capacity: usize,
    pub(crate) max_byte_count: usize,
    pub(crate) resident_byte_count: usize,
    pub(crate) hit_count: usize,
    pub(crate) approximate_hit_count: usize,
    pub(crate) approximate_probe_count: usize,
    pub(crate) miss_count: usize,
    pub(crate) insert_count: usize,
    pub(crate) worker_request_submitted_count: usize,
    pub(crate) worker_request_pending_count: usize,
    pub(crate) worker_request_deferred_count: usize,
    pub(crate) worker_request_failed_count: usize,
    pub(crate) worker_request_backpressured_count: usize,
    pub(crate) worker_request_font_missing_count: usize,
    pub(crate) worker_request_font_copied_byte_count: usize,
    /// Font bytes retained by the worker-facing font snapshot for the active face epoch.
    pub(crate) worker_raster_font_resident_byte_count: usize,
    /// Distinct backend faces retained by the worker-facing font snapshot for the active epoch.
    pub(crate) worker_raster_font_entry_count: usize,
    pub(crate) worker_request_unavailable_count: usize,
    pub(crate) worker_request_cancelled_count: usize,
    pub(crate) worker_completion_insert_count: usize,
    pub(crate) worker_completion_failed_count: usize,
    pub(crate) worker_completion_unknown_count: usize,
    pub(crate) worker_completion_invalid_bitmap_count: usize,
    pub(crate) worker_completion_face_invalidated_count: usize,
    pub(crate) worker_completion_applied_byte_count: usize,
    pub(crate) worker_completion_drained_byte_count: usize,
    pub(crate) worker_completion_byte_budget_deferred_count: usize,
    pub(crate) worker_completion_oversized_accepted_count: usize,
    pub(crate) worker_pool_budgeted_thread_count: usize,
    pub(crate) worker_pool_in_flight_count: usize,
    pub(crate) worker_pool_queued_count: usize,
    pub(crate) worker_pool_queued_input_byte_count: usize,
    pub(crate) worker_pool_running_count: usize,
    pub(crate) worker_pool_completed_total: u64,
    pub(crate) worker_pool_failed_total: u64,
    pub(crate) worker_pool_queue_peak_count: usize,
    pub(crate) worker_pool_completion_backlog_count: usize,
    pub(crate) worker_pool_completion_backlog_byte_count: usize,
    pub(crate) worker_pool_completion_backpressured_total: u64,
    pub(crate) worker_pool_completion_budget_rejected_total: u64,
    pub(crate) worker_pool_completion_rejected_byte_total: u64,
    pub(crate) worker_pool_request_backpressured_total: u64,
    pub(crate) worker_pool_cancelled_total: u64,
    pub(crate) lru_repair_count: usize,
    pub(crate) lru_touch_count: usize,
    pub(crate) evicted_count: usize,
    pub(crate) evicted_byte_count: usize,
    pub(crate) budget_linked_eviction_count: usize,
    pub(crate) linked_raster_invalidation_count: usize,
    pub(crate) rejected_byte_budget_count: usize,
    pub(crate) invalidated_count: usize,
    pub(crate) entry_count: usize,
    /// Unique persistent bitmap raster keys currently bound to live source-cache entries.
    pub(crate) persistent_raster_key_count: usize,
    pub(crate) pending_worker_count: usize,
}

impl NativeBitmapAtlasSourceCacheFrameReport {
    pub(crate) fn record_worker_pool_diagnostics(
        &mut self,
        diagnostics: TextRasterWorkerPoolDiagnostics,
    ) {
        self.worker_pool_budgeted_thread_count = diagnostics.budgeted_threads;
        self.worker_pool_in_flight_count = diagnostics.in_flight;
        self.worker_pool_queued_count = diagnostics.queued;
        self.worker_pool_queued_input_byte_count = diagnostics.queued_input_bytes;
        self.worker_pool_running_count = diagnostics.running;
        self.worker_pool_completed_total = diagnostics.completed;
        self.worker_pool_failed_total = diagnostics.failed;
        self.worker_pool_queue_peak_count = diagnostics.queue_peak;
        self.worker_pool_completion_backlog_count = diagnostics.completion_backlog;
        self.worker_pool_completion_backlog_byte_count = diagnostics.completion_backlog_bytes;
        self.worker_pool_completion_backpressured_total = diagnostics.completion_backpressured;
        self.worker_pool_completion_budget_rejected_total = diagnostics.completion_budget_rejected;
        self.worker_pool_completion_rejected_byte_total = diagnostics.completion_rejected_bytes;
        self.worker_pool_request_backpressured_total = diagnostics.request_backpressured;
        self.worker_pool_cancelled_total = diagnostics.cancelled;
    }
}
