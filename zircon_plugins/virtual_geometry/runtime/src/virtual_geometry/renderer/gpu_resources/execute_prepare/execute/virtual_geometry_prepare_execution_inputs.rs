use super::super::super::gpu_pending_request_input::GpuPendingRequestInput;

pub(super) struct VirtualGeometryPrepareExecutionInputs {
    pub(super) resident_entries: Vec<[u32; 2]>,
    pub(super) resident_slots: Vec<u32>,
    pub(super) pending_requests: Vec<GpuPendingRequestInput>,
    pub(super) available_slots: Vec<u32>,
    pub(super) evictable_slots: Vec<u32>,
    pub(super) page_table_words: Vec<u32>,
    pub(super) page_table_word_count: usize,
    pub(super) completed_word_count: usize,
}
