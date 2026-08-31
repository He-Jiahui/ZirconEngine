use zr_rhi::SubmissionTicket;

use crate::ui_surface::WgpuUiImageInFlightPins;

use super::super::buffer_upload_batch::WgpuBufferUploadBatch;
use super::super::resource_upload_batch::WgpuResourceUploadBatch;
use super::super::upload_batch::WgpuTextureUploadBatch;

pub(super) enum QueuedWgpuSubmission {
    Command {
        ticket: SubmissionTicket,
        command_buffers: Vec<wgpu::CommandBuffer>,
        ui_image_pins: Option<WgpuUiImageInFlightPins>,
    },
    BufferUpload {
        ticket: SubmissionTicket,
        batch: WgpuBufferUploadBatch,
    },
    TextureUpload {
        ticket: SubmissionTicket,
        batch: WgpuTextureUploadBatch,
    },
    ResourceUpload {
        ticket: SubmissionTicket,
        batch: WgpuResourceUploadBatch,
    },
}

impl QueuedWgpuSubmission {
    pub(super) const fn ticket(&self) -> SubmissionTicket {
        match self {
            Self::Command { ticket, .. }
            | Self::BufferUpload { ticket, .. }
            | Self::TextureUpload { ticket, .. }
            | Self::ResourceUpload { ticket, .. } => *ticket,
        }
    }

    pub(super) fn staging_bytes(&self) -> Option<u64> {
        match self {
            Self::BufferUpload { batch, .. } => Some(batch.payload_byte_len()),
            Self::TextureUpload { batch, .. } => Some(batch.payload_byte_len()),
            Self::ResourceUpload { batch, .. } => Some(batch.payload_byte_len()),
            Self::Command { .. } => None,
        }
    }
}

pub(super) fn queued_upload_stats(submissions: &[QueuedWgpuSubmission]) -> (usize, u64) {
    submissions.iter().fold(
        (0_usize, 0_u64),
        |(count, bytes), submission| match submission.staging_bytes() {
            Some(staging_bytes) => (count.saturating_add(1), bytes.saturating_add(staging_bytes)),
            None => (count, bytes),
        },
    )
}
