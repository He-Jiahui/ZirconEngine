use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use zr_rhi::{
    DeviceGeneration, DeviceId, GpuMemoryBudget, GpuMemoryClass, RenderQueueClass, RhiError,
    SubmissionHistory, SubmissionLimits, SubmissionPollReceipt, SubmissionStatus, SubmissionTicket,
    TextureCopyRegion,
};

use crate::ui_surface::WgpuUiImageInFlightPins;

use super::buffer_upload_batch::{WgpuBufferUpload, WgpuBufferUploadBatch};
use super::resource_upload_batch::WgpuResourceUploadBatch;
use super::submission_metrics::{WgpuSubmissionMetrics, WgpuSubmissionMetricsSnapshot};
use super::translate::wgpu_texture_copy_aspect;
use super::upload_batch::{WgpuTextureUpload, WgpuTextureUploadBatch};

mod queued_work;
mod ui_image_retirement;

use queued_work::{queued_upload_stats, QueuedWgpuSubmission};
use ui_image_retirement::WgpuUiImageRetirementOwner;

/// Owns every native queue operation for one WGPU device generation.
///
/// WGPU command encoders and command buffers are one-shot objects, so this
/// service pools only the small command-context metadata that ties a ticket to
/// its submission. Native encoder reuse is deliberately not attempted.
pub(crate) struct WgpuSubmissionService {
    // Pending native work must be released before this service drops its queue handle.
    state: Arc<Mutex<WgpuSubmissionState>>,
    queue_access: Mutex<()>,
    ui_image_retirements: WgpuUiImageRetirementOwner,
    queue: wgpu::Queue,
    device_id: DeviceId,
    generation: DeviceGeneration,
}

struct WgpuSubmissionState {
    next_sequence: u64,
    next_poll_sequence: u64,
    history: SubmissionHistory,
    reserved: HashSet<SubmissionTicket>,
    pending: Vec<QueuedWgpuSubmission>,
    flushing_upload_bytes: u64,
    submitted_at: HashMap<SubmissionTicket, Instant>,
    metrics: WgpuSubmissionMetrics,
    memory_budget: GpuMemoryBudget,
    contexts: WgpuCommandContextPool,
}

#[derive(Default)]
struct WgpuCommandContextPool {
    allocated: usize,
    available: usize,
    accepted: HashSet<SubmissionTicket>,
    in_flight: HashSet<SubmissionTicket>,
}

impl WgpuCommandContextPool {
    fn checkout(&mut self, ticket: SubmissionTicket) {
        if self.available > 0 {
            self.available -= 1;
        } else {
            self.allocated = self.allocated.saturating_add(1);
        }
        self.accepted.insert(ticket);
    }

    fn mark_submitted(&mut self, ticket: SubmissionTicket) {
        self.accepted.remove(&ticket);
        self.in_flight.insert(ticket);
    }

    fn release_submitted(&mut self, ticket: SubmissionTicket) {
        if self.in_flight.remove(&ticket) {
            self.available = self.available.saturating_add(1);
        }
    }

    fn release_pending(&mut self, ticket: SubmissionTicket) {
        if self.accepted.remove(&ticket) {
            self.available = self.available.saturating_add(1);
        }
    }

    #[cfg(test)]
    fn counts(&self) -> (usize, usize) {
        (self.allocated, self.available)
    }
}

impl WgpuSubmissionService {
    pub(crate) fn new(
        queue: wgpu::Queue,
        device_id: DeviceId,
        generation: DeviceGeneration,
        memory_budget: GpuMemoryBudget,
        submission_limits: SubmissionLimits,
    ) -> Self {
        Self {
            device_id,
            generation,
            queue,
            queue_access: Mutex::new(()),
            ui_image_retirements: WgpuUiImageRetirementOwner::default(),
            state: Arc::new(Mutex::new(WgpuSubmissionState {
                next_sequence: 1,
                next_poll_sequence: 1,
                history: SubmissionHistory::new(submission_limits),
                reserved: HashSet::new(),
                pending: Vec::new(),
                flushing_upload_bytes: 0,
                submitted_at: HashMap::new(),
                metrics: WgpuSubmissionMetrics::default(),
                memory_budget,
                contexts: WgpuCommandContextPool::default(),
            })),
        }
    }

    /// Reserves one accepted ticket before a device owner records its native
    /// command buffer. The reservation cannot reach `flush` until committed.
    pub(crate) fn begin_packet(
        &self,
        queue_class: RenderQueueClass,
    ) -> Result<SubmissionTicket, RhiError> {
        let mut state = self.lock_state();
        if !state.history.can_accept() {
            return Err(RhiError::SubmissionBackpressure {
                unresolved_submissions: state.history.unresolved_count(),
                limit: state.history.limits().max_unresolved_submissions(),
            });
        }
        let sequence = state.next_sequence;
        state.next_sequence =
            sequence
                .checked_add(1)
                .ok_or(RhiError::SubmissionSequenceExhausted {
                    device_id: self.device_id,
                    generation: self.generation,
                })?;
        let ticket = SubmissionTicket::new(self.device_id, self.generation, queue_class, sequence);
        state.contexts.checkout(ticket);
        debug_assert!(state.history.record_accepted(ticket));
        state.reserved.insert(ticket);
        Ok(ticket)
    }

    /// Publishes the encoded packet for a previously accepted ticket.
    pub(crate) fn commit_packet(
        &self,
        ticket: SubmissionTicket,
        command_buffers: Vec<wgpu::CommandBuffer>,
    ) -> Result<(), RhiError> {
        self.commit_packet_with_ui_image_pins(ticket, command_buffers, None)
    }

    pub(crate) fn commit_packet_with_ui_image_pins(
        &self,
        ticket: SubmissionTicket,
        command_buffers: Vec<wgpu::CommandBuffer>,
        ui_image_pins: Option<WgpuUiImageInFlightPins>,
    ) -> Result<(), RhiError> {
        if ticket.device_id() != self.device_id || ticket.generation() != self.generation {
            return Err(RhiError::UnknownSubmissionTicket(ticket));
        }
        let _queue_access = self.lock_queue_access();
        let mut state = self.lock_state();
        let status = state
            .history
            .status(ticket)
            .ok_or(RhiError::UnknownSubmissionTicket(ticket))?;
        if status != SubmissionStatus::Accepted || !state.reserved.remove(&ticket) {
            return Err(RhiError::SubmissionNotAcceptingPacket { ticket, status });
        }
        state.pending.push(QueuedWgpuSubmission::Command {
            ticket,
            command_buffers,
            ui_image_pins,
        });
        Ok(())
    }

    /// Publishes a staged buffer upload for a previously accepted ticket.
    ///
    /// The service retains the native buffer reference and payload until a
    /// later flush, so logical destruction can invalidate the public handle
    /// without invalidating accepted GPU work.
    pub(crate) fn commit_upload(
        &self,
        ticket: SubmissionTicket,
        buffer: wgpu::Buffer,
        offset: u64,
        data: Vec<u8>,
    ) -> Result<(), RhiError> {
        let upload = WgpuBufferUpload::from_owned_bytes(buffer, offset, data);
        self.commit_buffer_upload_batch(ticket, WgpuBufferUploadBatch::from(upload))
    }

    /// Publishes every range for one logical buffer upload ticket.
    pub(crate) fn commit_buffer_upload_batch(
        &self,
        ticket: SubmissionTicket,
        batch: WgpuBufferUploadBatch,
    ) -> Result<(), RhiError> {
        let buffer_write_count = batch.upload_count();
        self.commit_staged_upload(
            ticket,
            QueuedWgpuSubmission::BufferUpload { ticket, batch },
            buffer_write_count,
            0,
        )
    }

    /// Publishes a staged texture upload for a previously accepted ticket.
    /// The payload and texture stay owned by this submission service until it
    /// reaches the native queue, preserving the same lifecycle as buffers.
    pub(crate) fn commit_texture_upload(
        &self,
        ticket: SubmissionTicket,
        texture: wgpu::Texture,
        region: TextureCopyRegion,
        bytes_per_row: u32,
        data: Vec<u8>,
    ) -> Result<(), RhiError> {
        let upload = WgpuTextureUpload::from_owned_bytes(
            texture,
            region,
            bytes_per_row,
            region.height,
            data,
        );
        self.commit_texture_upload_batch(ticket, WgpuTextureUploadBatch::from(upload))
    }

    /// Publishes every mip/layer write for one logical texture upload ticket.
    pub(crate) fn commit_texture_upload_batch(
        &self,
        ticket: SubmissionTicket,
        batch: WgpuTextureUploadBatch,
    ) -> Result<(), RhiError> {
        let texture_write_count = batch.upload_count();
        self.commit_staged_upload(
            ticket,
            QueuedWgpuSubmission::TextureUpload { ticket, batch },
            0,
            texture_write_count,
        )
    }

    /// Publishes buffer and texture setup writes under one previously accepted Copy ticket.
    pub(crate) fn commit_resource_upload_batch(
        &self,
        ticket: SubmissionTicket,
        batch: WgpuResourceUploadBatch,
    ) -> Result<(), RhiError> {
        let buffer_write_count = batch.buffer_upload_count();
        let texture_write_count = batch.texture_upload_count();
        self.commit_staged_upload(
            ticket,
            QueuedWgpuSubmission::ResourceUpload { ticket, batch },
            buffer_write_count,
            texture_write_count,
        )
    }

    fn commit_staged_upload(
        &self,
        ticket: SubmissionTicket,
        upload: QueuedWgpuSubmission,
        buffer_write_count: usize,
        texture_write_count: usize,
    ) -> Result<(), RhiError> {
        debug_assert!(upload.staging_bytes().is_some());
        debug_assert!(buffer_write_count > 0 || texture_write_count > 0);
        if ticket.device_id() != self.device_id || ticket.generation() != self.generation {
            return Err(RhiError::UnknownSubmissionTicket(ticket));
        }
        let _queue_access = self.lock_queue_access();
        let mut state = self.lock_state();
        let status = state
            .history
            .status(ticket)
            .ok_or(RhiError::UnknownSubmissionTicket(ticket))?;
        if status != SubmissionStatus::Accepted || !state.reserved.remove(&ticket) {
            return Err(RhiError::SubmissionNotAcceptingPacket { ticket, status });
        }
        let (pending_uploads, pending_upload_bytes) = queued_upload_stats(&state.pending);
        if pending_uploads >= state.memory_budget.max_pending_uploads() {
            state.reserved.insert(ticket);
            state
                .metrics
                .record_resource_upload_rejected(buffer_write_count > 0, texture_write_count > 0);
            return Err(RhiError::UploadBackpressure {
                pending_uploads,
                limit: state.memory_budget.max_pending_uploads(),
            });
        }
        let requested_bytes = upload.staging_bytes().unwrap_or_default();
        let current_bytes = pending_upload_bytes.saturating_add(state.flushing_upload_bytes);
        let limit_bytes = state.memory_budget.staging_bytes();
        if requested_bytes > limit_bytes.saturating_sub(current_bytes) {
            state.reserved.insert(ticket);
            state
                .metrics
                .record_resource_upload_rejected(buffer_write_count > 0, texture_write_count > 0);
            return Err(RhiError::MemoryBudgetExceeded {
                class: GpuMemoryClass::UploadStaging,
                current_bytes,
                requested_bytes,
                limit_bytes,
            });
        }
        state.pending.push(upload);
        state.metrics.record_resource_upload_admitted(
            buffer_write_count,
            texture_write_count,
            requested_bytes,
            current_bytes.saturating_add(requested_bytes),
        );
        Ok(())
    }

    pub(crate) fn flush(&self) -> Result<usize, RhiError> {
        let _queue_access = self.lock_queue_access();
        let submissions = {
            let mut state = self.lock_state();
            let submissions = std::mem::take(&mut state.pending);
            state.flushing_upload_bytes = state.flushing_upload_bytes.saturating_add(
                submissions
                    .iter()
                    .filter_map(QueuedWgpuSubmission::staging_bytes)
                    .fold(0_u64, u64::saturating_add),
            );
            submissions
        };
        let count = submissions.len();
        if count == 0 {
            return Ok(0);
        }

        let mut tickets = Vec::with_capacity(count);
        let mut command_buffers = Vec::with_capacity(count);
        let mut ui_image_retirements = Vec::new();
        for submission in submissions {
            match submission {
                QueuedWgpuSubmission::Command {
                    ticket,
                    command_buffers: packet_command_buffers,
                    ui_image_pins,
                } => {
                    tickets.push(ticket);
                    command_buffers.extend(packet_command_buffers);
                    if let Some(pins) = ui_image_pins {
                        ui_image_retirements.push((ticket, pins));
                    }
                }
                QueuedWgpuSubmission::BufferUpload { ticket, batch } => {
                    // A new upload following encoded work creates a real
                    // ordering boundary. Consecutive uploads remain batched
                    // with their following command buffers in one submit.
                    if !command_buffers.is_empty() {
                        self.submit_native_batch(
                            &mut tickets,
                            &mut command_buffers,
                            &mut ui_image_retirements,
                        );
                    }
                    let batch_bytes = batch.payload_byte_len();
                    for upload in batch.into_uploads() {
                        self.queue
                            .write_buffer(upload.buffer(), upload.offset(), upload.payload());
                    }
                    self.release_flushing_upload_bytes(batch_bytes);
                    tickets.push(ticket);
                }
                QueuedWgpuSubmission::TextureUpload { ticket, batch } => {
                    if !command_buffers.is_empty() {
                        self.submit_native_batch(
                            &mut tickets,
                            &mut command_buffers,
                            &mut ui_image_retirements,
                        );
                    }
                    let batch_bytes = batch.payload_byte_len();
                    for upload in batch.into_uploads() {
                        let region = upload.region();
                        self.queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: upload.texture(),
                                mip_level: region.mip_level,
                                origin: wgpu::Origin3d {
                                    x: region.origin_x,
                                    y: region.origin_y,
                                    z: region.origin_z,
                                },
                                aspect: wgpu_texture_copy_aspect(region.aspect),
                            },
                            upload.payload(),
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(upload.bytes_per_row()),
                                rows_per_image: Some(upload.rows_per_image()),
                            },
                            wgpu::Extent3d {
                                width: region.width,
                                height: region.height,
                                depth_or_array_layers: region.depth_or_array_layers,
                            },
                        );
                    }
                    self.release_flushing_upload_bytes(batch_bytes);
                    tickets.push(ticket);
                }
                QueuedWgpuSubmission::ResourceUpload { ticket, batch } => {
                    if !command_buffers.is_empty() {
                        self.submit_native_batch(
                            &mut tickets,
                            &mut command_buffers,
                            &mut ui_image_retirements,
                        );
                    }
                    let batch_bytes = batch.payload_byte_len();
                    let (buffer_uploads, texture_uploads) = batch.into_batches();
                    for upload in buffer_uploads.into_uploads() {
                        self.queue
                            .write_buffer(upload.buffer(), upload.offset(), upload.payload());
                    }
                    for upload in texture_uploads.into_uploads() {
                        let region = upload.region();
                        self.queue.write_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: upload.texture(),
                                mip_level: region.mip_level,
                                origin: wgpu::Origin3d {
                                    x: region.origin_x,
                                    y: region.origin_y,
                                    z: region.origin_z,
                                },
                                aspect: wgpu_texture_copy_aspect(region.aspect),
                            },
                            upload.payload(),
                            wgpu::TexelCopyBufferLayout {
                                offset: 0,
                                bytes_per_row: Some(upload.bytes_per_row()),
                                rows_per_image: Some(upload.rows_per_image()),
                            },
                            wgpu::Extent3d {
                                width: region.width,
                                height: region.height,
                                depth_or_array_layers: region.depth_or_array_layers,
                            },
                        );
                    }
                    self.release_flushing_upload_bytes(batch_bytes);
                    tickets.push(ticket);
                }
            }
        }
        self.submit_native_batch(
            &mut tickets,
            &mut command_buffers,
            &mut ui_image_retirements,
        );
        Ok(count)
    }

    pub(crate) fn status(&self, ticket: SubmissionTicket) -> Result<SubmissionStatus, RhiError> {
        if ticket.device_id() != self.device_id || ticket.generation() != self.generation {
            return Err(RhiError::UnknownSubmissionTicket(ticket));
        }
        self.lock_state()
            .history
            .status(ticket)
            .ok_or(RhiError::UnknownSubmissionTicket(ticket))
    }

    pub(crate) fn append_statuses(
        &self,
        tickets: &[SubmissionTicket],
        statuses: &mut Vec<Result<SubmissionStatus, RhiError>>,
    ) {
        let state = self.lock_state();
        statuses.reserve(tickets.len());
        statuses.extend(tickets.iter().copied().map(|ticket| {
            if ticket.device_id() != self.device_id || ticket.generation() != self.generation {
                return Err(RhiError::UnknownSubmissionTicket(ticket));
            }
            state
                .history
                .status(ticket)
                .ok_or(RhiError::UnknownSubmissionTicket(ticket))
        }));
    }

    pub(crate) fn is_ticket_terminal(&self, ticket: SubmissionTicket) -> bool {
        ticket.device_id() == self.device_id
            && ticket.generation() == self.generation
            && self.lock_state().history.is_terminal(ticket)
    }

    pub(crate) fn cancel(&self, ticket: SubmissionTicket) -> Result<SubmissionStatus, RhiError> {
        if ticket.device_id() != self.device_id || ticket.generation() != self.generation {
            return Err(RhiError::UnknownSubmissionTicket(ticket));
        }
        // Keep cancellation from observing a packet after flush removed it but
        // before the native queue accepted and published its submitted state.
        let _queue_access = self.lock_queue_access();
        let mut state = self.lock_state();
        let status = state
            .history
            .status(ticket)
            .ok_or(RhiError::UnknownSubmissionTicket(ticket))?;
        let cancelled_submission = match status {
            SubmissionStatus::Accepted => {
                let cancelled_submission = if state.reserved.remove(&ticket) {
                    state.contexts.release_pending(ticket);
                    None
                } else {
                    let pending_index = state
                        .pending
                        .iter()
                        .position(|submission| submission.ticket() == ticket)
                        .ok_or(RhiError::UnknownSubmissionTicket(ticket))?;
                    state.contexts.release_pending(ticket);
                    Some(state.pending.remove(pending_index))
                };
                state
                    .history
                    .transition(ticket, SubmissionStatus::Cancelled);
                cancelled_submission
            }
            SubmissionStatus::Submitted => {
                return Err(RhiError::SubmissionCannotCancel { ticket, status });
            }
            terminal => return Ok(terminal),
        };
        drop(state);
        drop(cancelled_submission);
        Ok(SubmissionStatus::Cancelled)
    }

    /// Settles an ordered abandoned-frame ticket batch under one queue/state lock.
    ///
    /// Accepted packets are cancelled. Submitted and terminal packets keep their
    /// observable state so the frame owner can publish an exact failure receipt.
    pub(crate) fn settle_abandoned_submissions(
        &self,
        tickets: &[SubmissionTicket],
    ) -> Result<Vec<SubmissionStatus>, RhiError> {
        let _queue_access = self.lock_queue_access();
        let mut state = self.lock_state();
        let mut statuses = Vec::with_capacity(tickets.len());
        let mut accepted = HashSet::with_capacity(tickets.len());

        for &ticket in tickets {
            if ticket.device_id() != self.device_id || ticket.generation() != self.generation {
                return Err(RhiError::UnknownSubmissionTicket(ticket));
            }
            let status = state
                .history
                .status(ticket)
                .ok_or(RhiError::UnknownSubmissionTicket(ticket))?;
            if status == SubmissionStatus::Accepted {
                accepted.insert(ticket);
            }
            statuses.push(status);
        }

        if accepted.is_empty() {
            return Ok(statuses);
        }

        for &ticket in tickets {
            if accepted.remove(&ticket) && state.reserved.remove(&ticket) {
                state.contexts.release_pending(ticket);
                state
                    .history
                    .transition(ticket, SubmissionStatus::Cancelled);
            } else if state.history.status(ticket) == Some(SubmissionStatus::Accepted) {
                accepted.insert(ticket);
            }
        }

        let pending = std::mem::take(&mut state.pending);
        let mut cancelled_submissions = Vec::new();
        for submission in pending {
            let ticket = submission.ticket();
            if accepted.remove(&ticket) {
                state.contexts.release_pending(ticket);
                state
                    .history
                    .transition(ticket, SubmissionStatus::Cancelled);
                cancelled_submissions.push(submission);
            } else {
                state.pending.push(submission);
            }
        }

        if let Some(ticket) = accepted.into_iter().next() {
            return Err(RhiError::UnknownSubmissionTicket(ticket));
        }
        for status in &mut statuses {
            if *status == SubmissionStatus::Accepted {
                *status = SubmissionStatus::Cancelled;
            }
        }
        drop(state);
        drop(cancelled_submissions);
        Ok(statuses)
    }

    pub(crate) fn poll(&self, device: &wgpu::Device) -> Result<SubmissionPollReceipt, RhiError> {
        device
            .poll(wgpu::PollType::Poll)
            .map_err(|error| RhiError::NativeDevicePoll {
                reason: error.to_string(),
            })?;
        self.issue_poll_receipt()
    }

    pub(crate) fn issue_poll_receipt(&self) -> Result<SubmissionPollReceipt, RhiError> {
        let mut state = self.lock_state();
        let sequence = state.next_poll_sequence;
        state.next_poll_sequence =
            sequence
                .checked_add(1)
                .ok_or(RhiError::SubmissionPollSequenceExhausted {
                    device_id: self.device_id,
                    generation: self.generation,
                })?;
        Ok(SubmissionPollReceipt::new(
            self.device_id,
            self.generation,
            sequence,
        ))
    }

    pub(crate) fn terminalize_unresolved(&self, status: SubmissionStatus) {
        debug_assert!(status.is_terminal());
        // Fault handling shares the queue transition lock so it cannot turn a
        // packet terminal while flush is between native submit and state publish.
        let _queue_access = self.lock_queue_access();
        let mut state = self.lock_state();
        let reserved = std::mem::take(&mut state.reserved);
        for ticket in reserved {
            state.history.transition(ticket, status);
            state.contexts.release_pending(ticket);
        }
        let pending = std::mem::take(&mut state.pending);
        state.flushing_upload_bytes = 0;
        for submission in &pending {
            let ticket = submission.ticket();
            state.history.transition(ticket, status);
            state.contexts.release_pending(ticket);
        }

        let unresolved = state.history.unresolved_tickets();
        for ticket in unresolved {
            state.history.transition(ticket, status);
            state.submitted_at.remove(&ticket);
            state.contexts.release_submitted(ticket);
        }
        drop(state);
        drop(pending);
        self.ui_image_retirements.terminalize_all();
    }

    #[cfg(test)]
    pub(crate) fn command_context_pool_counts(&self) -> (usize, usize) {
        self.lock_state().contexts.counts()
    }

    pub(crate) fn pending_upload_bytes(&self) -> u64 {
        let state = self.lock_state();
        state
            .flushing_upload_bytes
            .saturating_add(queued_upload_stats(&state.pending).1)
    }

    pub(crate) fn metrics_snapshot(&self) -> WgpuSubmissionMetricsSnapshot {
        let state = self.lock_state();
        let (_, pending_upload_bytes) = queued_upload_stats(&state.pending);
        state.metrics.snapshot(
            self.device_id,
            self.generation,
            pending_upload_bytes.saturating_add(state.flushing_upload_bytes),
        )
    }

    fn lock_state(&self) -> MutexGuard<'_, WgpuSubmissionState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_queue_access(&self) -> MutexGuard<'_, ()> {
        self.queue_access
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn release_flushing_upload_bytes(&self, bytes: u64) {
        let mut state = self.lock_state();
        state.flushing_upload_bytes = state.flushing_upload_bytes.saturating_sub(bytes);
    }

    fn submit_native_batch(
        &self,
        tickets: &mut Vec<SubmissionTicket>,
        command_buffers: &mut Vec<wgpu::CommandBuffer>,
        ui_image_retirements: &mut Vec<(SubmissionTicket, WgpuUiImageInFlightPins)>,
    ) {
        if tickets.is_empty() {
            return;
        }

        let submitted_at = Instant::now();
        let ticket_count = tickets.len();
        self.queue.submit(std::mem::take(command_buffers));
        let completed_tickets = std::mem::take(tickets);
        self.ui_image_retirements
            .retain_batch(std::mem::take(ui_image_retirements));
        {
            let mut state = self.lock_state();
            state.metrics.record_native_submission(ticket_count);
            for ticket in &completed_tickets {
                state
                    .history
                    .transition(*ticket, SubmissionStatus::Submitted);
                state.contexts.mark_submitted(*ticket);
                state.submitted_at.insert(*ticket, submitted_at);
            }
        }

        let completion_state = Arc::clone(&self.state);
        let completion_retirements = self.ui_image_retirements.clone();
        self.queue.on_submitted_work_done(move || {
            let completed_at = Instant::now();
            let mut state = completion_state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            for &ticket in &completed_tickets {
                if state.history.status(ticket) == Some(SubmissionStatus::Submitted) {
                    state
                        .history
                        .transition(ticket, SubmissionStatus::Completed);
                    if let Some(submitted_at) = state.submitted_at.remove(&ticket) {
                        state.metrics.record_completion(
                            completed_at.saturating_duration_since(submitted_at),
                        );
                    }
                    state.contexts.release_submitted(ticket);
                }
            }
            drop(state);
            completion_retirements.complete(&completed_tickets);
        });
    }
}
