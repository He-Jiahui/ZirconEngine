use std::collections::{HashMap, VecDeque};
use std::sync::mpsc::{self, TryRecvError};

use zr_rhi::{
    DeviceGeneration, DeviceId, DiagnosticFrameKey, DiagnosticReadbackAdmission,
    DiagnosticReadbackBudget, DiagnosticReadbackError, DiagnosticReadbackKind,
    DiagnosticReadbackRequestId, DiagnosticReadbackTerminal, DiagnosticReadbackTracker, RhiError,
    SubmissionStatus, SubmissionTicket, TextureCopyRegion, TextureHandle,
};

use super::super::query::{
    WgpuDiagnosticQueryDelivery, WgpuDiagnosticQueryFrame, WgpuDiagnosticQueryService,
    WgpuNativeDiagnosticQueryFrame, WgpuNativeDiagnosticQueryRecorder,
};
use super::batch::{
    ActiveDiagnosticReadbackBatch, DiagnosticReadbackBatch, DiagnosticReadbackBatchRequest,
    InFlightDiagnosticReadbackBatch,
};
use super::completion_order::{DiagnosticBatchCompletion, TicketOrderedDiagnosticCompletions};
use super::delivery::WgpuDiagnosticReadbackDelivery;
use super::layout::{
    align_up, texture_row_alignment, DiagnosticTextureMipChainReadbackLayout,
    DiagnosticTextureReadbackLayout,
};
use super::metrics::{WgpuDiagnosticReadbackMetrics, WgpuDiagnosticReadbackMetricsSnapshot};
use super::request::{
    DiagnosticBufferReadbackRequest, DiagnosticNativeBufferReadbackRequest,
    DiagnosticNativeTextureMipChainReadbackRequest, DiagnosticNativeTextureReadbackRequest,
    DiagnosticReadbackSource, DiagnosticTextureReadbackRequest,
};

const BUFFER_COPY_ALIGNMENT: u64 = wgpu::COPY_BUFFER_ALIGNMENT as u64;

/// Device-generation-local diagnostic copy/map lifecycle owner.
///
/// This service owns no queue and never polls a `wgpu::Device`. The enclosing
/// `WgpuRenderDevice` invokes it after the single submission-service poll, so
/// map callbacks, submission state, and terminal receipts share one owner.
pub(crate) struct WgpuDiagnosticReadbackService {
    tracker: DiagnosticReadbackTracker,
    metrics: WgpuDiagnosticReadbackMetrics,
    query_service: WgpuDiagnosticQueryService,
    active: Option<ActiveDiagnosticReadbackBatch>,
    in_flight: HashMap<SubmissionTicket, InFlightDiagnosticReadbackBatch>,
    completion_order: TicketOrderedDiagnosticCompletions<DiagnosticBatchCompletion>,
    deliveries: VecDeque<WgpuDiagnosticReadbackDelivery>,
    delivery_bytes: u64,
    dropped_delivery_count: u64,
}

impl WgpuDiagnosticReadbackService {
    pub(crate) fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        budget: DiagnosticReadbackBudget,
    ) -> Self {
        Self {
            tracker: DiagnosticReadbackTracker::new(device_id, generation, budget),
            metrics: WgpuDiagnosticReadbackMetrics::new(device_id, generation),
            query_service: WgpuDiagnosticQueryService::new(device_id, generation, budget),
            active: None,
            in_flight: HashMap::new(),
            completion_order: TicketOrderedDiagnosticCompletions::default(),
            deliveries: VecDeque::new(),
            delivery_bytes: 0,
            dropped_delivery_count: 0,
        }
    }

    pub(crate) fn begin_frame(&mut self, frame_index: u64) -> Result<(), DiagnosticReadbackError> {
        self.tracker.begin_frame(frame_index)?;
        self.metrics.begin_frame();
        self.active = Some(ActiveDiagnosticReadbackBatch {
            requests: Vec::new(),
            byte_len: 0,
        });
        Ok(())
    }

    pub(crate) fn admit_buffer(
        &mut self,
        source: zr_rhi::BufferHandle,
        source_offset: u64,
        byte_len: u64,
    ) -> Result<DiagnosticReadbackAdmission, DiagnosticReadbackError> {
        let staging = self.reserve_active_staging(BUFFER_COPY_ALIGNMENT, byte_len)?;
        let admission = self
            .tracker
            .admit_or_reject(DiagnosticReadbackKind::Buffer, staging.accounted_byte_len)?;
        self.metrics
            .record_admission(admission, staging.accounted_byte_len);
        if let DiagnosticReadbackAdmission::Admitted(request) = admission {
            let active = self
                .active
                .as_mut()
                .ok_or(DiagnosticReadbackError::NoActiveFrame)?;
            active.requests.push(DiagnosticReadbackBatchRequest {
                source: DiagnosticReadbackSource::Buffer(DiagnosticBufferReadbackRequest {
                    request,
                    source,
                    source_offset,
                    byte_len,
                }),
                staging_offset: staging.offset,
            });
            active.byte_len = staging.end;
        }
        self.drain_tracker_receipts();
        Ok(admission)
    }

    pub(crate) fn admit_texture(
        &mut self,
        source: TextureHandle,
        region: TextureCopyRegion,
        layout: DiagnosticTextureReadbackLayout,
    ) -> Result<DiagnosticReadbackAdmission, DiagnosticReadbackError> {
        let staging =
            self.reserve_active_staging(texture_row_alignment(), layout.staging_byte_len())?;
        let admission = self
            .tracker
            .admit_or_reject(DiagnosticReadbackKind::Texture, staging.accounted_byte_len)?;
        self.metrics
            .record_admission(admission, staging.accounted_byte_len);
        if let DiagnosticReadbackAdmission::Admitted(request) = admission {
            let active = self
                .active
                .as_mut()
                .ok_or(DiagnosticReadbackError::NoActiveFrame)?;
            active.requests.push(DiagnosticReadbackBatchRequest {
                source: DiagnosticReadbackSource::Texture(DiagnosticTextureReadbackRequest {
                    request,
                    source,
                    region,
                    layout,
                }),
                staging_offset: staging.offset,
            });
            active.byte_len = staging.end;
        }
        self.drain_tracker_receipts();
        Ok(admission)
    }

    pub(crate) fn admit_native_buffer(
        &mut self,
        source: wgpu::Buffer,
        source_offset: u64,
        byte_len: u64,
    ) -> Result<DiagnosticReadbackAdmission, DiagnosticReadbackError> {
        let staging = self.reserve_active_staging(BUFFER_COPY_ALIGNMENT, byte_len)?;
        let admission = self
            .tracker
            .admit_or_reject(DiagnosticReadbackKind::Buffer, staging.accounted_byte_len)?;
        self.metrics
            .record_admission(admission, staging.accounted_byte_len);
        if let DiagnosticReadbackAdmission::Admitted(request) = admission {
            let active = self
                .active
                .as_mut()
                .ok_or(DiagnosticReadbackError::NoActiveFrame)?;
            active.requests.push(DiagnosticReadbackBatchRequest {
                source: DiagnosticReadbackSource::NativeBuffer(
                    DiagnosticNativeBufferReadbackRequest {
                        request,
                        source,
                        source_offset,
                        byte_len,
                    },
                ),
                staging_offset: staging.offset,
            });
            active.byte_len = staging.end;
        }
        self.drain_tracker_receipts();
        Ok(admission)
    }

    pub(crate) fn admit_native_texture(
        &mut self,
        source: wgpu::Texture,
        region: TextureCopyRegion,
        layout: DiagnosticTextureReadbackLayout,
    ) -> Result<DiagnosticReadbackAdmission, DiagnosticReadbackError> {
        let staging =
            self.reserve_active_staging(texture_row_alignment(), layout.staging_byte_len())?;
        let admission = self
            .tracker
            .admit_or_reject(DiagnosticReadbackKind::Texture, staging.accounted_byte_len)?;
        self.metrics
            .record_admission(admission, staging.accounted_byte_len);
        if let DiagnosticReadbackAdmission::Admitted(request) = admission {
            let active = self
                .active
                .as_mut()
                .ok_or(DiagnosticReadbackError::NoActiveFrame)?;
            active.requests.push(DiagnosticReadbackBatchRequest {
                source: DiagnosticReadbackSource::NativeTexture(
                    DiagnosticNativeTextureReadbackRequest {
                        request,
                        source,
                        region,
                        layout,
                    },
                ),
                staging_offset: staging.offset,
            });
            active.byte_len = staging.end;
        }
        self.drain_tracker_receipts();
        Ok(admission)
    }

    pub(crate) fn admit_native_texture_mip_chain(
        &mut self,
        source: wgpu::Texture,
        layout: DiagnosticTextureMipChainReadbackLayout,
    ) -> Result<DiagnosticReadbackAdmission, DiagnosticReadbackError> {
        let staging =
            self.reserve_active_staging(texture_row_alignment(), layout.staging_byte_len())?;
        let admission = self
            .tracker
            .admit_or_reject(DiagnosticReadbackKind::Texture, staging.accounted_byte_len)?;
        self.metrics
            .record_admission(admission, staging.accounted_byte_len);
        if let DiagnosticReadbackAdmission::Admitted(request) = admission {
            let active = self
                .active
                .as_mut()
                .ok_or(DiagnosticReadbackError::NoActiveFrame)?;
            active.requests.push(DiagnosticReadbackBatchRequest {
                source: DiagnosticReadbackSource::NativeTextureMipChain(
                    DiagnosticNativeTextureMipChainReadbackRequest {
                        request,
                        source,
                        layout,
                    },
                ),
                staging_offset: staging.offset,
            });
            active.byte_len = staging.end;
        }
        self.drain_tracker_receipts();
        Ok(admission)
    }

    pub(crate) fn take_active_batch(
        &mut self,
    ) -> Result<Option<DiagnosticReadbackBatch>, DiagnosticReadbackError> {
        let active = self
            .active
            .take()
            .ok_or(DiagnosticReadbackError::NoActiveFrame)?;
        if active.requests.is_empty() {
            self.metrics.seal_active_frame();
            self.tracker
                .terminalize_active_frame(DiagnosticReadbackTerminal::Shutdown);
            self.drain_tracker_receipts();
            return Ok(None);
        }
        Ok(Some(DiagnosticReadbackBatch {
            requests: active.requests,
            byte_len: active.byte_len,
        }))
    }

    pub(crate) fn abandon_active_batch(&mut self, terminal: DiagnosticReadbackTerminal) {
        self.active = None;
        self.metrics.seal_active_frame();
        self.tracker.terminalize_active_frame(terminal);
        self.drain_tracker_receipts();
    }

    pub(crate) fn bind_batch(
        &mut self,
        ticket: SubmissionTicket,
        batch: DiagnosticReadbackBatch,
        staging: wgpu::Buffer,
    ) -> Result<DiagnosticFrameKey, DiagnosticReadbackError> {
        let frame_key = self.tracker.bind_active_frame(ticket)?;
        let request_count = batch.requests.len();
        let byte_len = batch.byte_len;
        self.in_flight.insert(
            ticket,
            InFlightDiagnosticReadbackBatch {
                frame_key,
                staging,
                byte_len: batch.byte_len,
                requests: batch.requests,
                map_receiver: None,
            },
        );
        self.completion_order.register(ticket);
        self.metrics.record_submitted_batch(request_count, byte_len);
        Ok(frame_key)
    }

    pub(crate) fn collect_completed_maps(
        &mut self,
        mut status_for: impl FnMut(SubmissionTicket) -> Result<SubmissionStatus, RhiError>,
    ) -> Result<(), RhiError> {
        self.start_maps(&mut status_for)?;
        let mut completed = Vec::new();
        for (ticket, batch) in &mut self.in_flight {
            if self.completion_order.is_completed(*ticket) {
                continue;
            }
            let Some(receiver) = batch.map_receiver.as_ref() else {
                continue;
            };
            match receiver.try_recv() {
                Ok(Ok(())) => {
                    self.metrics.record_map_completed();
                    completed.push((*ticket, DiagnosticBatchCompletion::Mapped));
                }
                Ok(Err(_)) | Err(TryRecvError::Disconnected) => {
                    self.metrics.record_map_completed();
                    completed.push((*ticket, DiagnosticBatchCompletion::MapFailed))
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        for (ticket, completion) in completed {
            if let Some(batch) = self.in_flight.get_mut(&ticket) {
                batch.map_receiver = None;
            }
            self.completion_order.complete(ticket, completion);
        }
        self.drain_completed_batches();
        self.query_service.collect_completed(status_for)?;
        self.drain_tracker_receipts();
        Ok(())
    }

    pub(crate) fn terminalize_submission(
        &mut self,
        ticket: SubmissionTicket,
        terminal: DiagnosticReadbackTerminal,
    ) {
        if self.in_flight.contains_key(&ticket) {
            self.completion_order
                .complete(ticket, DiagnosticBatchCompletion::Terminal(terminal));
            self.drain_completed_batches();
        }
        self.query_service.terminalize_submission(ticket, terminal);
        self.drain_tracker_receipts();
    }

    pub(crate) fn terminalize_all(&mut self, terminal: DiagnosticReadbackTerminal) {
        self.active = None;
        self.in_flight.clear();
        self.metrics.seal_active_frame();
        self.metrics.clear_in_flight();
        self.completion_order.clear();
        self.query_service.terminalize_all(terminal);
        self.tracker.terminalize_all(terminal);
        self.drain_tracker_receipts();
    }

    pub(crate) fn take_delivery(&mut self) -> Option<WgpuDiagnosticReadbackDelivery> {
        let delivery = self.deliveries.pop_front()?;
        self.metrics
            .record_delivery_drained(delivery.byte_len_for_budget());
        self.delivery_bytes = self
            .delivery_bytes
            .saturating_sub(delivery.byte_len_for_budget());
        Some(delivery)
    }

    pub(crate) fn append_deliveries(
        &mut self,
        output: &mut Vec<WgpuDiagnosticReadbackDelivery>,
    ) -> usize {
        let appended = self.deliveries.len();
        output.reserve(appended);
        while let Some(delivery) = self.take_delivery() {
            output.push(delivery);
        }
        appended
    }

    /// Retains an unrelated oldest delivery for the request owner that created it.
    pub(crate) fn take_delivery_for(
        &mut self,
        request: DiagnosticReadbackRequestId,
    ) -> Option<WgpuDiagnosticReadbackDelivery> {
        if self.deliveries.front()?.receipt().request() == request {
            self.take_delivery()
        } else {
            None
        }
    }

    pub(crate) fn dropped_delivery_count(&self) -> u64 {
        self.dropped_delivery_count
    }

    pub(crate) const fn retained_delivery_bytes(&self) -> u64 {
        self.delivery_bytes
    }

    pub(crate) fn metrics_snapshot(&self) -> WgpuDiagnosticReadbackMetricsSnapshot {
        self.metrics.snapshot(
            self.deliveries.len(),
            self.delivery_bytes,
            self.dropped_delivery_count,
        )
    }

    pub(crate) fn prepare_query_frame(
        &mut self,
        device: &wgpu::Device,
        timestamp_period_ns: f32,
        ticket: SubmissionTicket,
        plan: &zr_rhi::DiagnosticQueryPlan,
    ) -> Result<Option<WgpuDiagnosticQueryFrame>, RhiError> {
        self.query_service
            .prepare_frame(device, timestamp_period_ns, ticket, plan)
    }

    pub(crate) fn begin_native_query_frame(
        &mut self,
        device: &wgpu::Device,
        timestamp_period_ns: f32,
        frame_index: u64,
        timestamps_enabled: bool,
        pipeline_statistics_enabled: bool,
    ) -> Result<Option<WgpuNativeDiagnosticQueryRecorder>, RhiError> {
        self.query_service.begin_native_frame(
            device,
            timestamp_period_ns,
            frame_index,
            timestamps_enabled,
            pipeline_statistics_enabled,
        )
    }

    pub(crate) fn prepare_native_query_frame(
        &mut self,
        device: &wgpu::Device,
        recorder: WgpuNativeDiagnosticQueryRecorder,
        plan: zr_rhi::DiagnosticQueryPlan,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Option<WgpuNativeDiagnosticQueryFrame>, RhiError> {
        self.query_service
            .prepare_native_frame(device, recorder, plan, encoder)
    }

    pub(crate) fn bind_native_query_frame(
        &mut self,
        ticket: SubmissionTicket,
        frame: WgpuNativeDiagnosticQueryFrame,
    ) -> Result<DiagnosticFrameKey, RhiError> {
        self.query_service.bind_native_frame(ticket, frame)
    }

    pub(crate) fn abandon_native_query_recorder(
        &mut self,
        recorder: WgpuNativeDiagnosticQueryRecorder,
        terminal: DiagnosticReadbackTerminal,
    ) {
        self.query_service
            .abandon_native_recorder(recorder, terminal);
    }

    pub(crate) fn abandon_prepared_native_query_frame(
        &mut self,
        frame: WgpuNativeDiagnosticQueryFrame,
        terminal: DiagnosticReadbackTerminal,
    ) {
        self.query_service
            .abandon_prepared_native_frame(frame, terminal);
    }

    pub(crate) fn commit_query_frame(
        &mut self,
        ticket: SubmissionTicket,
        frame: WgpuDiagnosticQueryFrame,
    ) {
        self.query_service.commit_frame(ticket, frame);
    }

    pub(crate) fn abandon_prepared_query_frame(
        &mut self,
        frame: WgpuDiagnosticQueryFrame,
        terminal: DiagnosticReadbackTerminal,
    ) {
        self.query_service.abandon_prepared_frame(frame, terminal);
    }

    pub(crate) fn take_query_delivery(&mut self) -> Option<WgpuDiagnosticQueryDelivery> {
        self.query_service.take_delivery()
    }

    pub(crate) fn append_query_deliveries(
        &mut self,
        output: &mut Vec<WgpuDiagnosticQueryDelivery>,
    ) -> usize {
        self.query_service.append_deliveries(output)
    }

    pub(crate) fn dropped_query_delivery_count(&self) -> u64 {
        self.query_service.dropped_delivery_count()
    }

    fn start_maps(
        &mut self,
        status_for: &mut impl FnMut(SubmissionTicket) -> Result<SubmissionStatus, RhiError>,
    ) -> Result<(), RhiError> {
        let mut terminal = Vec::new();
        for (ticket, batch) in &mut self.in_flight {
            if batch.map_receiver.is_some() || self.completion_order.is_completed(*ticket) {
                continue;
            }
            match status_for(*ticket)? {
                SubmissionStatus::Submitted | SubmissionStatus::Completed => {
                    let (sender, receiver) = mpsc::channel();
                    batch.staging.map_async(
                        wgpu::MapMode::Read,
                        0..batch.byte_len,
                        move |result| {
                            let _ = sender.send(result);
                        },
                    );
                    batch.map_receiver = Some(receiver);
                    self.metrics.record_map_started();
                }
                status => {
                    if let Some(terminal_status) = terminal_for_submission(status) {
                        terminal.push((*ticket, terminal_status));
                    }
                }
            }
        }
        for (ticket, terminal_status) in terminal {
            self.terminalize_submission(ticket, terminal_status);
        }
        Ok(())
    }

    fn drain_completed_batches(&mut self) {
        while let Some((ticket, completion)) = self.completion_order.take_next_ready() {
            let Some(batch) = self.in_flight.remove(&ticket) else {
                continue;
            };
            self.metrics
                .release_in_flight_batch(batch.requests.len(), batch.byte_len);
            match completion {
                DiagnosticBatchCompletion::Mapped => self.complete_mapped_batch(batch),
                DiagnosticBatchCompletion::MapFailed => {
                    self.terminalize_frame(batch.frame_key, DiagnosticReadbackTerminal::MapFailed)
                }
                DiagnosticBatchCompletion::Terminal(terminal) => {
                    self.terminalize_frame(batch.frame_key, terminal)
                }
            }
        }
    }

    fn complete_mapped_batch(&mut self, batch: InFlightDiagnosticReadbackBatch) {
        let payloads = {
            let mapped = batch.staging.get_mapped_range(0..batch.byte_len);
            let mut payloads = Vec::with_capacity(batch.requests.len());
            let mut valid = true;
            for request in &batch.requests {
                let Some(bytes) = request
                    .source()
                    .copy_payload(&mapped, request.staging_offset())
                else {
                    valid = false;
                    break;
                };
                payloads.push((request.source().request(), bytes));
            }
            valid.then_some(payloads)
        };
        batch.staging.unmap();
        let Some(payloads) = payloads else {
            self.terminalize_frame(batch.frame_key, DiagnosticReadbackTerminal::MapFailed);
            return;
        };
        for (request, bytes) in payloads {
            self.terminalize_request(request, DiagnosticReadbackTerminal::Succeeded, Some(bytes));
        }
    }

    fn terminalize_frame(
        &mut self,
        frame_key: DiagnosticFrameKey,
        terminal: DiagnosticReadbackTerminal,
    ) {
        self.tracker.terminalize_frame(frame_key, terminal);
        self.drain_tracker_receipts();
    }

    fn terminalize_request(
        &mut self,
        request: DiagnosticReadbackRequestId,
        terminal: DiagnosticReadbackTerminal,
        bytes: Option<Vec<u8>>,
    ) {
        let receipt = self.tracker.terminalize(request, terminal);
        if let (Some(receipt), Some(bytes)) = (receipt, bytes) {
            self.push_delivery(WgpuDiagnosticReadbackDelivery {
                receipt,
                bytes: Some(bytes),
            });
        }
        self.drain_tracker_receipts();
    }

    fn drain_tracker_receipts(&mut self) {
        while let Some(receipt) = self.tracker.take_completed_receipt() {
            if WgpuDiagnosticQueryService::is_query_kind(receipt.kind()) {
                continue;
            }
            if receipt.terminal() != DiagnosticReadbackTerminal::Succeeded {
                self.push_delivery(WgpuDiagnosticReadbackDelivery {
                    receipt,
                    bytes: None,
                });
            }
        }
    }

    pub(super) fn push_delivery(&mut self, delivery: WgpuDiagnosticReadbackDelivery) {
        self.metrics.record_terminal(delivery.receipt());
        let receipt_limit = self.tracker.budget().max_completed_receipts();
        let byte_limit = self.tracker.budget().max_pending_bytes();
        let delivery_bytes = delivery.byte_len_for_budget();
        if receipt_limit == 0 || delivery_bytes > byte_limit {
            self.dropped_delivery_count = self.dropped_delivery_count.saturating_add(1);
            return;
        }
        while self.deliveries.len() >= receipt_limit
            || delivery_bytes > byte_limit.saturating_sub(self.delivery_bytes)
        {
            let Some(evicted) = self.deliveries.pop_front() else {
                self.dropped_delivery_count = self.dropped_delivery_count.saturating_add(1);
                return;
            };
            self.delivery_bytes = self
                .delivery_bytes
                .saturating_sub(evicted.byte_len_for_budget());
            self.dropped_delivery_count = self.dropped_delivery_count.saturating_add(1);
        }
        self.delivery_bytes = self.delivery_bytes.saturating_add(delivery_bytes);
        self.deliveries.push_back(delivery);
        self.metrics
            .observe_retained_deliveries(self.deliveries.len(), self.delivery_bytes);
    }

    fn reserve_active_staging(
        &self,
        alignment: u64,
        byte_len: u64,
    ) -> Result<StagingReservation, DiagnosticReadbackError> {
        let active = self
            .active
            .as_ref()
            .ok_or(DiagnosticReadbackError::NoActiveFrame)?;
        let offset = align_up(active.byte_len, alignment)
            .ok_or(DiagnosticReadbackError::StagingLayoutOverflow)?;
        let end = offset
            .checked_add(byte_len)
            .ok_or(DiagnosticReadbackError::StagingLayoutOverflow)?;
        Ok(StagingReservation {
            offset,
            end,
            accounted_byte_len: end.saturating_sub(active.byte_len),
        })
    }
}

#[derive(Clone, Copy)]
struct StagingReservation {
    offset: u64,
    end: u64,
    accounted_byte_len: u64,
}

fn terminal_for_submission(status: SubmissionStatus) -> Option<DiagnosticReadbackTerminal> {
    match status {
        SubmissionStatus::Accepted | SubmissionStatus::Submitted | SubmissionStatus::Completed => {
            None
        }
        SubmissionStatus::Cancelled => Some(DiagnosticReadbackTerminal::Cancelled),
        SubmissionStatus::DeviceLost => Some(DiagnosticReadbackTerminal::DeviceLost),
        SubmissionStatus::Failed => Some(DiagnosticReadbackTerminal::MapFailed),
    }
}
