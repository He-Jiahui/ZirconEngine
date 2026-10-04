//! 待提交工作按接受顺序保存命令与上传载荷，直到唯一队列完成原生提交。
use zr_rhi::SubmissionTicket;

use crate::ui_surface::WgpuUiImageInFlightPins;

use super::super::buffer_upload_batch::WgpuBufferUploadBatch;
use super::super::resource_upload_batch::WgpuResourceUploadBatch;
use super::super::upload_batch::WgpuTextureUploadBatch;

// 上传与命令均保留所有权载荷；队列按票据顺序批量刷新，不能让后来的写入越过先前命令。
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
