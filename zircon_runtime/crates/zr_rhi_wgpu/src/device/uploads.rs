use std::sync::Arc;

use zr_rhi::{
    BufferUploadBatch, BufferUsage, GpuMemoryClass, RenderQueueClass, RhiError, SubmissionTicket,
    TextureUpload, TextureUploadBatch, TextureUsage,
};

use super::{
    resources, DeterministicRhiContractDevice, DeterministicRhiContractDeviceState,
    QueuedDeterministicSubmission,
};
use crate::resource_validation::{ensure_buffer_usage, ensure_texture_usage};
use crate::texture_copy::{
    texture_upload_byte_len, texture_upload_layout, texture_write_out_of_range,
    validate_texture_copy_destination_aspect,
};

pub(super) fn enqueue_buffer_upload_batch(
    device: &DeterministicRhiContractDevice,
    batch: BufferUploadBatch,
) -> Result<SubmissionTicket, RhiError> {
    if batch.is_empty() {
        return Err(RhiError::EmptyUploadBatch);
    }
    let staging_bytes = batch
        .payload_byte_len()
        .ok_or(RhiError::UploadByteCountOverflow)?;
    let mut state = device.lock_state();
    for upload in batch.uploads() {
        let handle = upload.buffer();
        let buffer_desc = state
            .buffers
            .get(&handle)
            .map(|buffer| buffer.desc.clone())
            .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))?;
        ensure_buffer_usage(handle.diagnostic_id(), &buffer_desc, BufferUsage::COPY_DST)?;
        let size = upload.payload_byte_len();
        if upload.destination_offset().saturating_add(size) > buffer_desc.size_bytes {
            return Err(RhiError::WriteOutOfRange {
                buffer: handle.diagnostic_id(),
                offset: upload.destination_offset(),
                size,
            });
        }
    }
    ensure_upload_admission(device, &state, staging_bytes)?;
    let ticket = state.allocate_submission_ticket(
        device.device_id,
        device.generation,
        RenderQueueClass::Copy,
    )?;
    state
        .pending_submissions
        .push(QueuedDeterministicSubmission::Upload { ticket, batch });
    Ok(ticket)
}

pub(super) fn enqueue_texture_upload_batch(
    device: &DeterministicRhiContractDevice,
    batch: TextureUploadBatch,
) -> Result<SubmissionTicket, RhiError> {
    if batch.is_empty() {
        return Err(RhiError::EmptyUploadBatch);
    }
    let mut state = device.lock_state();
    let mut canonical_batch = TextureUploadBatch::new();
    for upload in batch.uploads() {
        let handle = upload.texture();
        let region = upload.region();
        let bytes_per_row = upload.bytes_per_row();
        let texture_desc = state
            .textures
            .get(&handle)
            .map(|texture| texture.desc.clone())
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))?;
        ensure_texture_usage(
            handle.diagnostic_id(),
            &texture_desc,
            TextureUsage::COPY_DST,
        )?;
        validate_texture_copy_destination_aspect(handle, &texture_desc, region)?;
        let source_bytes = upload.payload_byte_len();
        let layout = texture_upload_layout(&texture_desc, region, bytes_per_row, source_bytes)
            .ok_or_else(|| {
                texture_write_out_of_range(handle, source_bytes, bytes_per_row, region)
            })?;
        let upload_bytes = texture_upload_byte_len(region, bytes_per_row, layout.copy_row_bytes)
            .ok_or_else(|| {
                texture_write_out_of_range(handle, source_bytes, bytes_per_row, region)
            })?;
        let upload_len = usize::try_from(upload_bytes)
            .map_err(|_| texture_write_out_of_range(handle, source_bytes, bytes_per_row, region))?;
        let source_range = upload.source_range();
        let source_end = source_range
            .start
            .checked_add(upload_len)
            .ok_or(RhiError::UploadByteCountOverflow)?;
        let canonical_range = source_range.start..source_end;
        let canonical = TextureUpload::new(
            handle,
            region,
            bytes_per_row,
            Arc::clone(upload.payload_owner()),
            canonical_range.clone(),
        )
        .ok_or(RhiError::InvalidUploadSourceRange {
            start: canonical_range.start,
            end: canonical_range.end,
            payload_bytes: upload.payload_owner().len(),
        })?;
        canonical_batch.push(canonical);
    }
    let staging_bytes = canonical_batch
        .payload_byte_len()
        .ok_or(RhiError::UploadByteCountOverflow)?;
    ensure_upload_admission(device, &state, staging_bytes)?;
    let ticket = state.allocate_submission_ticket(
        device.device_id,
        device.generation,
        RenderQueueClass::Copy,
    )?;
    state
        .pending_submissions
        .push(QueuedDeterministicSubmission::TextureUpload {
            ticket,
            batch: canonical_batch,
        });
    Ok(ticket)
}

pub(super) fn execute_buffer_upload_batch(
    state: &mut DeterministicRhiContractDeviceState,
    batch: BufferUploadBatch,
) -> Result<(), RhiError> {
    for upload in batch.into_uploads() {
        resources::execute_buffer_upload(
            state,
            upload.buffer(),
            upload.destination_offset(),
            upload.payload(),
        )?;
    }
    Ok(())
}

pub(super) fn execute_texture_upload_batch(
    state: &mut DeterministicRhiContractDeviceState,
    batch: TextureUploadBatch,
) -> Result<(), RhiError> {
    for upload in batch.into_uploads() {
        resources::execute_texture_upload(
            state,
            upload.texture(),
            upload.region(),
            upload.bytes_per_row(),
            upload.payload(),
        )?;
    }
    Ok(())
}

fn ensure_upload_admission(
    device: &DeterministicRhiContractDevice,
    state: &DeterministicRhiContractDeviceState,
    staging_bytes: u64,
) -> Result<(), RhiError> {
    let (pending_uploads, pending_upload_bytes) = state.pending_upload_stats();
    if pending_uploads >= device.memory_budget.max_pending_uploads() {
        return Err(RhiError::UploadBackpressure {
            pending_uploads,
            limit: device.memory_budget.max_pending_uploads(),
        });
    }
    resources::ensure_memory_capacity(
        GpuMemoryClass::UploadStaging,
        pending_upload_bytes,
        staging_bytes,
        device.memory_budget.staging_bytes(),
    )
}
