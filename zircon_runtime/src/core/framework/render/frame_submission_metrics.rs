/// One device-generation-local frame submission interval.
///
/// Logical packets describe work admitted by the frame owner. Flushed tickets and physical
/// submissions come from the backend timeline sampled after the terminal scene packet. A caller
/// receives no metrics when the backend owner changes or its monotonic counters regress.
///
/// 此 DTO 记录提交与上传计数、字节数，不含 GPU 执行耗时；耗时状态由帧剖析单独报告。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderFrameSubmissionMetrics {
    admitted_logical_packet_count: u64,
    flushed_logical_ticket_count: u64,
    physical_backend_submission_count: u64,
    buffer_upload_batch_count: u64,
    texture_upload_batch_count: u64,
    buffer_write_count: u64,
    texture_write_count: u64,
    upload_payload_bytes: u64,
}

impl RenderFrameSubmissionMetrics {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        admitted_logical_packet_count: u64,
        flushed_logical_ticket_count: u64,
        physical_backend_submission_count: u64,
        buffer_upload_batch_count: u64,
        texture_upload_batch_count: u64,
        buffer_write_count: u64,
        texture_write_count: u64,
        upload_payload_bytes: u64,
    ) -> Self {
        Self {
            admitted_logical_packet_count,
            flushed_logical_ticket_count,
            physical_backend_submission_count,
            buffer_upload_batch_count,
            texture_upload_batch_count,
            buffer_write_count,
            texture_write_count,
            upload_payload_bytes,
        }
    }

    pub const fn admitted_logical_packet_count(self) -> u64 {
        self.admitted_logical_packet_count
    }

    pub const fn flushed_logical_ticket_count(self) -> u64 {
        self.flushed_logical_ticket_count
    }

    pub const fn physical_backend_submission_count(self) -> u64 {
        self.physical_backend_submission_count
    }

    pub const fn buffer_upload_batch_count(self) -> u64 {
        self.buffer_upload_batch_count
    }

    pub const fn texture_upload_batch_count(self) -> u64 {
        self.texture_upload_batch_count
    }

    pub const fn buffer_write_count(self) -> u64 {
        self.buffer_write_count
    }

    pub const fn texture_write_count(self) -> u64 {
        self.texture_write_count
    }

    pub const fn upload_payload_bytes(self) -> u64 {
        self.upload_payload_bytes
    }
}

#[cfg(test)]
#[path = "tests/frame_submission_metrics.rs"]
mod tests;
