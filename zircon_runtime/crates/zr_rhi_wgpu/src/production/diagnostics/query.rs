//! Submission-bound native query allocation, resolve, and ordered delivery.

mod support;

use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::sync::mpsc::{self, Receiver, TryRecvError};

use zr_rhi::{
    aggregate_diagnostic_query_results, DeviceGeneration, DeviceId, DiagnosticFrameKey,
    DiagnosticPassResult, DiagnosticQueryPlan, DiagnosticReadbackAdmission,
    DiagnosticReadbackBudget, DiagnosticReadbackKind, DiagnosticReadbackTerminal,
    DiagnosticReadbackTracker, RhiError, SubmissionStatus, SubmissionTicket,
};

use super::readback::completion_order::{
    DiagnosticBatchCompletion, TicketOrderedDiagnosticCompletions,
};
use support::{
    bounded_query_count, bounded_timestamp_query_count, create_pipeline_statistics_query_set,
    create_timestamp_query_set, pipeline_statistics_query_bytes, prepare_query_resources,
    query_bytes, range_len, terminal_for_submission,
};

const QUERY_VALUE_BYTES: u64 = size_of::<u64>() as u64;
const PIPELINE_STATISTICS_TYPES: wgpu::PipelineStatisticsTypes =
    wgpu::PipelineStatisticsTypes::VERTEX_SHADER_INVOCATIONS
        .union(wgpu::PipelineStatisticsTypes::CLIPPER_INVOCATIONS)
        .union(wgpu::PipelineStatisticsTypes::CLIPPER_PRIMITIVES_OUT)
        .union(wgpu::PipelineStatisticsTypes::FRAGMENT_SHADER_INVOCATIONS)
        .union(wgpu::PipelineStatisticsTypes::COMPUTE_SHADER_INVOCATIONS);
const TIMESTAMP_REQUIRED_FEATURES: wgpu::Features =
    wgpu::Features::TIMESTAMP_QUERY.union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);

/// One submission-qualified numeric diagnostic result. Labels stay in the
/// graph compiler, so native readback does not allocate or compare strings.
#[derive(Clone, Debug, PartialEq)]
pub struct WgpuDiagnosticQueryDelivery {
    pub frame_index: u64,
    pub frame_key: Option<DiagnosticFrameKey>,
    pub terminal: DiagnosticReadbackTerminal,
    pub timestamp_period_ns: f32,
    pub pass_results: Option<Vec<DiagnosticPassResult>>,
}

/// Generation-qualified query sets reserved before transitional native passes record.
///
/// The recorder exposes query objects but no queue, submission, flush, or poll authority. Its
/// maximum ranges are admitted up front; the scene tail later supplies the actual neutral plan.
#[must_use = "a native diagnostic query recorder must be prepared or explicitly aborted"]
pub struct WgpuNativeDiagnosticQueryRecorder {
    device_id: DeviceId,
    generation: DeviceGeneration,
    frame_index: u64,
    timestamp_period_ns: f32,
    timestamp_query_set: Option<wgpu::QuerySet>,
    pipeline_statistics_query_set: Option<wgpu::QuerySet>,
    max_timestamp_query_count: u32,
    max_pipeline_statistics_query_count: u32,
}

impl WgpuNativeDiagnosticQueryRecorder {
    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    pub const fn frame_index(&self) -> u64 {
        self.frame_index
    }

    pub const fn max_timestamp_query_count(&self) -> u32 {
        self.max_timestamp_query_count
    }

    pub const fn timestamp_period_ns(&self) -> f32 {
        self.timestamp_period_ns
    }

    pub const fn max_pipeline_statistics_query_count(&self) -> u32 {
        self.max_pipeline_statistics_query_count
    }

    pub fn timestamp_query_set(&self) -> Option<&wgpu::QuerySet> {
        self.timestamp_query_set.as_ref()
    }

    pub fn pipeline_statistics_query_set(&self) -> Option<&wgpu::QuerySet> {
        self.pipeline_statistics_query_set.as_ref()
    }
}

/// Opaque query resolve frame encoded at the tail of a native scene packet.
#[must_use = "a prepared native diagnostic query frame must be submitted or explicitly aborted"]
pub struct WgpuNativeDiagnosticQueryFrame {
    device_id: DeviceId,
    generation: DeviceGeneration,
    resources: PreparedDiagnosticQueryResources,
}

impl WgpuNativeDiagnosticQueryFrame {
    pub const fn device_id(&self) -> DeviceId {
        self.device_id
    }

    pub const fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    pub fn frame_index(&self) -> u64 {
        self.resources.frame_index()
    }
}

/// Native query objects and staging allocation retained until their one packet
/// reaches a completion boundary. It owns no queue and performs no polling.
pub(crate) struct WgpuDiagnosticQueryFrame {
    frame_key: DiagnosticFrameKey,
    resources: PreparedDiagnosticQueryResources,
}

struct PreparedDiagnosticQueryResources {
    plan: DiagnosticQueryPlan,
    timestamp_period_ns: f32,
    timestamp_query_set: Option<wgpu::QuerySet>,
    pipeline_statistics_query_set: Option<wgpu::QuerySet>,
    timestamp_resolve: Option<wgpu::Buffer>,
    pipeline_statistics_resolve: Option<wgpu::Buffer>,
    staging: wgpu::Buffer,
    timestamp_staging_range: Range<u64>,
    pipeline_statistics_staging_range: Range<u64>,
}

impl WgpuDiagnosticQueryFrame {
    pub(crate) fn timestamp_query_set(&self) -> Option<&wgpu::QuerySet> {
        self.resources.timestamp_query_set.as_ref()
    }

    pub(crate) fn pipeline_statistics_query_set(&self) -> Option<&wgpu::QuerySet> {
        self.resources.pipeline_statistics_query_set.as_ref()
    }

    pub(crate) fn encode_resolve(&self, device: &wgpu::Device) -> wgpu::CommandBuffer {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-diagnostic-query-resolve"),
        });
        self.resources.encode_resolve_into(&mut encoder);
        encoder.finish()
    }

    fn into_in_flight(self) -> InFlightDiagnosticQueryFrame {
        self.resources.into_in_flight(self.frame_key)
    }
}

impl PreparedDiagnosticQueryResources {
    fn frame_index(&self) -> u64 {
        self.plan
            .frame_index()
            .expect("prepared diagnostic query resources require a frame index")
    }

    fn encode_resolve_into(&self, encoder: &mut wgpu::CommandEncoder) {
        if let (Some(query_set), Some(resolve)) = (
            self.timestamp_query_set.as_ref(),
            self.timestamp_resolve.as_ref(),
        ) {
            let byte_len = range_len(&self.timestamp_staging_range);
            encoder.resolve_query_set(query_set, 0..self.plan.timestamp_query_count(), resolve, 0);
            encoder.copy_buffer_to_buffer(
                resolve,
                0,
                &self.staging,
                self.timestamp_staging_range.start,
                byte_len,
            );
        }
        if let (Some(query_set), Some(resolve)) = (
            self.pipeline_statistics_query_set.as_ref(),
            self.pipeline_statistics_resolve.as_ref(),
        ) {
            let byte_len = range_len(&self.pipeline_statistics_staging_range);
            encoder.resolve_query_set(
                query_set,
                0..self.plan.pipeline_statistics_query_count(),
                resolve,
                0,
            );
            encoder.copy_buffer_to_buffer(
                resolve,
                0,
                &self.staging,
                self.pipeline_statistics_staging_range.start,
                byte_len,
            );
        }
    }

    fn into_in_flight(self, frame_key: DiagnosticFrameKey) -> InFlightDiagnosticQueryFrame {
        InFlightDiagnosticQueryFrame {
            frame_index: self.frame_index(),
            frame_key,
            plan: self.plan,
            timestamp_period_ns: self.timestamp_period_ns,
            staging: self.staging,
            timestamp_staging_range: self.timestamp_staging_range,
            pipeline_statistics_staging_range: self.pipeline_statistics_staging_range,
            map_receiver: None,
        }
    }
}

pub(crate) struct WgpuDiagnosticQueryService {
    device_id: DeviceId,
    generation: DeviceGeneration,
    tracker: DiagnosticReadbackTracker,
    delivery_limit: usize,
    active_native: Option<ActiveNativeDiagnosticQueryFrame>,
    in_flight: HashMap<SubmissionTicket, InFlightDiagnosticQueryFrame>,
    completion_order: TicketOrderedDiagnosticCompletions<DiagnosticBatchCompletion>,
    deliveries: VecDeque<WgpuDiagnosticQueryDelivery>,
    dropped_delivery_count: u64,
}

impl WgpuDiagnosticQueryService {
    pub(crate) fn new(
        device_id: DeviceId,
        generation: DeviceGeneration,
        budget: DiagnosticReadbackBudget,
    ) -> Self {
        Self {
            device_id,
            generation,
            tracker: DiagnosticReadbackTracker::new(device_id, generation, budget),
            delivery_limit: budget.max_completed_receipts(),
            active_native: None,
            in_flight: HashMap::new(),
            completion_order: TicketOrderedDiagnosticCompletions::default(),
            deliveries: VecDeque::new(),
            dropped_delivery_count: 0,
        }
    }

    pub(crate) fn begin_native_frame(
        &mut self,
        device: &wgpu::Device,
        timestamp_period_ns: f32,
        frame_index: u64,
        timestamps_enabled: bool,
        pipeline_statistics_enabled: bool,
    ) -> Result<Option<WgpuNativeDiagnosticQueryRecorder>, RhiError> {
        if !timestamps_enabled && !pipeline_statistics_enabled {
            return Ok(None);
        }
        let budget = self.tracker.budget();
        let max_timestamp_query_count = if timestamps_enabled {
            bounded_timestamp_query_count(budget.max_timestamp_scopes())?
        } else {
            0
        };
        let max_pipeline_statistics_query_count = if pipeline_statistics_enabled {
            bounded_query_count(
                budget.max_pipeline_statistics_scopes(),
                "pipeline statistics",
            )?
        } else {
            0
        };
        if max_timestamp_query_count == 0 && max_pipeline_statistics_query_count == 0 {
            self.push_delivery(WgpuDiagnosticQueryDelivery {
                frame_index,
                frame_key: None,
                terminal: DiagnosticReadbackTerminal::OverBudget,
                timestamp_period_ns: 0.0,
                pass_results: None,
            });
            return Ok(None);
        }
        let features = device.features();
        if (max_timestamp_query_count > 0 && !features.contains(TIMESTAMP_REQUIRED_FEATURES))
            || (max_pipeline_statistics_query_count > 0
                && !features.contains(wgpu::Features::PIPELINE_STATISTICS_QUERY))
        {
            self.push_delivery(WgpuDiagnosticQueryDelivery {
                frame_index,
                frame_key: None,
                terminal: DiagnosticReadbackTerminal::Unavailable,
                timestamp_period_ns: 0.0,
                pass_results: None,
            });
            return Ok(None);
        }

        let timestamp_bytes = query_bytes(u64::from(max_timestamp_query_count))?;
        let pipeline_statistics_bytes =
            pipeline_statistics_query_bytes(u64::from(max_pipeline_statistics_query_count))?;
        self.tracker.begin_frame(frame_index)?;
        self.active_native = Some(ActiveNativeDiagnosticQueryFrame {
            frame_index,
            timestamp_period_ns,
        });
        let admitted =
            match self.admit_queries(timestamp_bytes, pipeline_statistics_bytes, frame_index) {
                Ok(admitted) => admitted,
                Err(error) => {
                    self.abandon_active_native_frame(DiagnosticReadbackTerminal::MapFailed);
                    return Err(error);
                }
            };
        if !admitted {
            return Ok(None);
        }

        Ok(Some(WgpuNativeDiagnosticQueryRecorder {
            device_id: self.device_id,
            generation: self.generation,
            frame_index,
            timestamp_period_ns,
            timestamp_query_set: create_timestamp_query_set(device, max_timestamp_query_count),
            pipeline_statistics_query_set: create_pipeline_statistics_query_set(
                device,
                max_pipeline_statistics_query_count,
            ),
            max_timestamp_query_count,
            max_pipeline_statistics_query_count,
        }))
    }

    pub(crate) fn prepare_native_frame(
        &mut self,
        device: &wgpu::Device,
        recorder: WgpuNativeDiagnosticQueryRecorder,
        plan: DiagnosticQueryPlan,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Option<WgpuNativeDiagnosticQueryFrame>, RhiError> {
        let frame_index = recorder.frame_index;
        if plan.is_empty() {
            self.abandon_active_native_frame(DiagnosticReadbackTerminal::Cancelled);
            return Ok(None);
        }
        if plan.frame_index() != Some(frame_index) {
            self.abandon_active_native_frame(DiagnosticReadbackTerminal::MapFailed);
            return Err(RhiError::ReadbackUnavailable {
                reason: "native diagnostic query plan frame does not match its recorder"
                    .to_string(),
            });
        }
        if plan.timestamp_query_count() > recorder.max_timestamp_query_count
            || plan.pipeline_statistics_query_count() > recorder.max_pipeline_statistics_query_count
        {
            self.abandon_active_native_frame(DiagnosticReadbackTerminal::OverBudget);
            return Err(RhiError::ReadbackUnavailable {
                reason: "native diagnostic query plan exceeds its admitted recorder ranges"
                    .to_string(),
            });
        }
        if (plan.timestamp_query_count() > 0 && recorder.timestamp_query_set.is_none())
            || (plan.pipeline_statistics_query_count() > 0
                && recorder.pipeline_statistics_query_set.is_none())
        {
            self.abandon_active_native_frame(DiagnosticReadbackTerminal::Unavailable);
            return Err(RhiError::DiagnosticQueryPlanRequired);
        }

        let resources = match prepare_query_resources(
            device,
            plan,
            recorder.timestamp_period_ns,
            recorder.timestamp_query_set,
            recorder.pipeline_statistics_query_set,
        ) {
            Ok(resources) => resources,
            Err(error) => {
                self.abandon_active_native_frame(DiagnosticReadbackTerminal::MapFailed);
                return Err(error);
            }
        };
        resources.encode_resolve_into(encoder);
        Ok(Some(WgpuNativeDiagnosticQueryFrame {
            device_id: recorder.device_id,
            generation: recorder.generation,
            resources,
        }))
    }

    pub(crate) fn bind_native_frame(
        &mut self,
        ticket: SubmissionTicket,
        frame: WgpuNativeDiagnosticQueryFrame,
    ) -> Result<DiagnosticFrameKey, RhiError> {
        let frame_key = match self.tracker.bind_active_frame(ticket) {
            Ok(frame_key) => frame_key,
            Err(error) => {
                self.abandon_active_native_frame(DiagnosticReadbackTerminal::MapFailed);
                return Err(error.into());
            }
        };
        self.active_native = None;
        debug_assert!(!self.in_flight.contains_key(&ticket));
        self.in_flight
            .insert(ticket, frame.resources.into_in_flight(frame_key));
        self.completion_order.register(ticket);
        Ok(frame_key)
    }

    pub(crate) fn abandon_native_recorder(
        &mut self,
        recorder: WgpuNativeDiagnosticQueryRecorder,
        terminal: DiagnosticReadbackTerminal,
    ) {
        debug_assert_eq!(
            self.active_native.map(|active| active.frame_index),
            Some(recorder.frame_index)
        );
        self.abandon_active_native_frame(terminal);
    }

    pub(crate) fn abandon_prepared_native_frame(
        &mut self,
        frame: WgpuNativeDiagnosticQueryFrame,
        terminal: DiagnosticReadbackTerminal,
    ) {
        debug_assert_eq!(
            self.active_native.map(|active| active.frame_index),
            Some(frame.frame_index())
        );
        self.abandon_active_native_frame(terminal);
    }

    pub(crate) fn prepare_frame(
        &mut self,
        device: &wgpu::Device,
        timestamp_period_ns: f32,
        ticket: SubmissionTicket,
        plan: &DiagnosticQueryPlan,
    ) -> Result<Option<WgpuDiagnosticQueryFrame>, RhiError> {
        if plan.is_empty() {
            return Ok(None);
        }
        let frame_index = plan
            .frame_index()
            .ok_or(RhiError::DiagnosticQueryFrameIndexRequired)?;
        let timestamp_bytes = query_bytes(u64::from(plan.timestamp_query_count()))?;
        let pipeline_statistics_bytes = query_bytes(
            u64::try_from(plan.pipeline_statistics_result_value_count()).map_err(|_| {
                RhiError::ReadbackUnavailable {
                    reason: "pipeline-statistics result value count overflowed".to_string(),
                }
            })?,
        )?;
        let features = device.features();
        if timestamp_bytes > 0 && !features.contains(TIMESTAMP_REQUIRED_FEATURES) {
            self.push_delivery(WgpuDiagnosticQueryDelivery {
                frame_index,
                frame_key: None,
                terminal: DiagnosticReadbackTerminal::Unavailable,
                timestamp_period_ns: 0.0,
                pass_results: None,
            });
            return Ok(None);
        }
        if pipeline_statistics_bytes > 0
            && !features.contains(wgpu::Features::PIPELINE_STATISTICS_QUERY)
        {
            self.push_delivery(WgpuDiagnosticQueryDelivery {
                frame_index,
                frame_key: None,
                terminal: DiagnosticReadbackTerminal::Unavailable,
                timestamp_period_ns: 0.0,
                pass_results: None,
            });
            return Ok(None);
        }

        self.tracker.begin_frame(frame_index)?;
        let admitted =
            match self.admit_queries(timestamp_bytes, pipeline_statistics_bytes, frame_index) {
                Ok(admitted) => admitted,
                Err(error) => {
                    self.tracker
                        .terminalize_active_frame(DiagnosticReadbackTerminal::MapFailed);
                    self.drain_tracker_receipts();
                    return Err(error);
                }
            };
        if !admitted {
            return Ok(None);
        }

        let resources = match prepare_query_resources(
            device,
            plan.clone(),
            timestamp_period_ns,
            create_timestamp_query_set(device, plan.timestamp_query_count()),
            create_pipeline_statistics_query_set(device, plan.pipeline_statistics_query_count()),
        ) {
            Ok(resources) => resources,
            Err(error) => {
                self.tracker
                    .terminalize_active_frame(DiagnosticReadbackTerminal::MapFailed);
                self.drain_tracker_receipts();
                return Err(error);
            }
        };
        let frame_key = match self.tracker.bind_active_frame(ticket) {
            Ok(frame_key) => frame_key,
            Err(error) => {
                self.tracker
                    .terminalize_active_frame(DiagnosticReadbackTerminal::MapFailed);
                self.drain_tracker_receipts();
                return Err(error.into());
            }
        };
        Ok(Some(WgpuDiagnosticQueryFrame {
            frame_key,
            resources,
        }))
    }

    pub(crate) fn commit_frame(
        &mut self,
        ticket: SubmissionTicket,
        frame: WgpuDiagnosticQueryFrame,
    ) {
        debug_assert!(!self.in_flight.contains_key(&ticket));
        self.in_flight.insert(ticket, frame.into_in_flight());
        self.completion_order.register(ticket);
    }

    pub(crate) fn abandon_prepared_frame(
        &mut self,
        frame: WgpuDiagnosticQueryFrame,
        terminal: DiagnosticReadbackTerminal,
    ) {
        self.tracker.terminalize_frame(frame.frame_key, terminal);
        self.drain_tracker_receipts();
        self.push_delivery(WgpuDiagnosticQueryDelivery {
            frame_index: frame.resources.frame_index(),
            frame_key: Some(frame.frame_key),
            terminal,
            timestamp_period_ns: frame.resources.timestamp_period_ns,
            pass_results: None,
        });
    }

    pub(crate) fn collect_completed(
        &mut self,
        mut status_for: impl FnMut(SubmissionTicket) -> Result<SubmissionStatus, RhiError>,
    ) -> Result<(), RhiError> {
        self.start_maps(&mut status_for)?;
        let mut completed = Vec::new();
        for (ticket, frame) in &mut self.in_flight {
            if self.completion_order.is_completed(*ticket) {
                continue;
            }
            let Some(receiver) = frame.map_receiver.as_ref() else {
                continue;
            };
            match receiver.try_recv() {
                Ok(Ok(())) => completed.push((*ticket, DiagnosticBatchCompletion::Mapped)),
                Ok(Err(_)) | Err(TryRecvError::Disconnected) => {
                    completed.push((*ticket, DiagnosticBatchCompletion::MapFailed))
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        for (ticket, completion) in completed {
            if let Some(frame) = self.in_flight.get_mut(&ticket) {
                frame.map_receiver = None;
            }
            self.completion_order.complete(ticket, completion);
        }
        self.drain_completed();
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
            self.drain_completed();
        }
    }

    pub(crate) fn terminalize_all(&mut self, terminal: DiagnosticReadbackTerminal) {
        self.completion_order
            .replace_all(DiagnosticBatchCompletion::Terminal(terminal));
        self.drain_completed();
        if let Some(active) = self.active_native.take() {
            self.push_delivery(WgpuDiagnosticQueryDelivery {
                frame_index: active.frame_index,
                frame_key: None,
                terminal,
                timestamp_period_ns: active.timestamp_period_ns,
                pass_results: None,
            });
        }
        self.tracker.terminalize_all(terminal);
        self.drain_tracker_receipts();
    }

    pub(crate) fn take_delivery(&mut self) -> Option<WgpuDiagnosticQueryDelivery> {
        self.deliveries.pop_front()
    }

    pub(crate) fn append_deliveries(
        &mut self,
        output: &mut Vec<WgpuDiagnosticQueryDelivery>,
    ) -> usize {
        let appended = self.deliveries.len();
        output.reserve(appended);
        output.extend(self.deliveries.drain(..));
        appended
    }

    pub(crate) const fn dropped_delivery_count(&self) -> u64 {
        self.dropped_delivery_count
    }

    fn admit_queries(
        &mut self,
        timestamp_bytes: u64,
        pipeline_statistics_bytes: u64,
        frame_index: u64,
    ) -> Result<bool, RhiError> {
        let timestamp_admission = if timestamp_bytes > 0 {
            Some(
                self.tracker
                    .admit_or_reject(zr_rhi::DiagnosticReadbackKind::Timestamp, timestamp_bytes)?,
            )
        } else {
            None
        };
        let statistics_admission = if pipeline_statistics_bytes > 0 {
            Some(self.tracker.admit_or_reject(
                zr_rhi::DiagnosticReadbackKind::PipelineStatistics,
                pipeline_statistics_bytes,
            )?)
        } else {
            None
        };
        let admitted = matches!(
            timestamp_admission,
            None | Some(DiagnosticReadbackAdmission::Admitted(_))
        ) && matches!(
            statistics_admission,
            None | Some(DiagnosticReadbackAdmission::Admitted(_))
        );
        if admitted {
            return Ok(true);
        }
        self.tracker
            .terminalize_active_frame(DiagnosticReadbackTerminal::OverBudget);
        self.drain_tracker_receipts();
        let timestamp_period_ns = self
            .active_native
            .take()
            .map_or(0.0, |active| active.timestamp_period_ns);
        self.push_delivery(WgpuDiagnosticQueryDelivery {
            frame_index,
            frame_key: None,
            terminal: DiagnosticReadbackTerminal::OverBudget,
            timestamp_period_ns,
            pass_results: None,
        });
        Ok(false)
    }

    fn abandon_active_native_frame(&mut self, terminal: DiagnosticReadbackTerminal) {
        self.tracker.terminalize_active_frame(terminal);
        self.drain_tracker_receipts();
        let Some(active) = self.active_native.take() else {
            return;
        };
        self.push_delivery(WgpuDiagnosticQueryDelivery {
            frame_index: active.frame_index,
            frame_key: None,
            terminal,
            timestamp_period_ns: active.timestamp_period_ns,
            pass_results: None,
        });
    }

    fn start_maps(
        &mut self,
        status_for: &mut impl FnMut(SubmissionTicket) -> Result<SubmissionStatus, RhiError>,
    ) -> Result<(), RhiError> {
        let mut terminal = Vec::new();
        for (ticket, frame) in &mut self.in_flight {
            if frame.map_receiver.is_some() || self.completion_order.is_completed(*ticket) {
                continue;
            }
            match status_for(*ticket)? {
                SubmissionStatus::Submitted | SubmissionStatus::Completed => {
                    let (sender, receiver) = mpsc::channel();
                    frame.staging.map_async(
                        wgpu::MapMode::Read,
                        0..frame.staging.size(),
                        move |result| {
                            let _ = sender.send(result);
                        },
                    );
                    frame.map_receiver = Some(receiver);
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

    fn drain_completed(&mut self) {
        while let Some((ticket, completion)) = self.completion_order.take_next_ready() {
            let Some(frame) = self.in_flight.remove(&ticket) else {
                continue;
            };
            match completion {
                DiagnosticBatchCompletion::Mapped => self.complete_mapped(frame),
                DiagnosticBatchCompletion::MapFailed => {
                    self.tracker
                        .terminalize_frame(frame.frame_key, DiagnosticReadbackTerminal::MapFailed);
                    self.drain_tracker_receipts();
                    self.push_terminal_delivery(frame, DiagnosticReadbackTerminal::MapFailed);
                }
                DiagnosticBatchCompletion::Terminal(terminal) => {
                    self.tracker.terminalize_frame(frame.frame_key, terminal);
                    self.drain_tracker_receipts();
                    self.push_terminal_delivery(frame, terminal);
                }
            }
        }
    }

    fn complete_mapped(&mut self, frame: InFlightDiagnosticQueryFrame) {
        let decoded = {
            let mapped = frame.staging.get_mapped_range(0..frame.staging.size());
            let timestamp_bytes = mapped[frame.timestamp_staging_range.start as usize
                ..frame.timestamp_staging_range.end as usize]
                .to_vec();
            let pipeline_statistics_bytes = mapped[frame.pipeline_statistics_staging_range.start
                as usize
                ..frame.pipeline_statistics_staging_range.end as usize]
                .to_vec();
            aggregate_diagnostic_query_results(
                &frame.plan,
                &timestamp_bytes,
                &pipeline_statistics_bytes,
            )
        };
        frame.staging.unmap();
        match decoded {
            Ok(pass_results) => {
                self.tracker
                    .terminalize_frame(frame.frame_key, DiagnosticReadbackTerminal::Succeeded);
                self.drain_tracker_receipts();
                self.push_delivery(WgpuDiagnosticQueryDelivery {
                    frame_index: frame.frame_index,
                    frame_key: Some(frame.frame_key),
                    terminal: DiagnosticReadbackTerminal::Succeeded,
                    timestamp_period_ns: frame.timestamp_period_ns,
                    pass_results: Some(pass_results),
                });
            }
            Err(_) => {
                self.tracker
                    .terminalize_frame(frame.frame_key, DiagnosticReadbackTerminal::MapFailed);
                self.drain_tracker_receipts();
                self.push_terminal_delivery(frame, DiagnosticReadbackTerminal::MapFailed);
            }
        }
    }

    fn push_terminal_delivery(
        &mut self,
        frame: InFlightDiagnosticQueryFrame,
        terminal: DiagnosticReadbackTerminal,
    ) {
        self.push_delivery(WgpuDiagnosticQueryDelivery {
            frame_index: frame.frame_index,
            frame_key: Some(frame.frame_key),
            terminal,
            timestamp_period_ns: frame.timestamp_period_ns,
            pass_results: None,
        });
    }

    fn push_delivery(&mut self, delivery: WgpuDiagnosticQueryDelivery) {
        let limit = self.delivery_limit;
        if limit == 0 {
            self.dropped_delivery_count = self.dropped_delivery_count.saturating_add(1);
            return;
        }
        if self.deliveries.len() >= limit {
            self.deliveries.pop_front();
            self.dropped_delivery_count = self.dropped_delivery_count.saturating_add(1);
        }
        self.deliveries.push_back(delivery);
    }

    fn drain_tracker_receipts(&mut self) {
        while self.tracker.take_completed_receipt().is_some() {}
    }

    pub(crate) const fn is_query_kind(kind: DiagnosticReadbackKind) -> bool {
        matches!(
            kind,
            DiagnosticReadbackKind::Timestamp | DiagnosticReadbackKind::PipelineStatistics
        )
    }
}

struct InFlightDiagnosticQueryFrame {
    frame_index: u64,
    frame_key: DiagnosticFrameKey,
    plan: DiagnosticQueryPlan,
    timestamp_period_ns: f32,
    staging: wgpu::Buffer,
    timestamp_staging_range: Range<u64>,
    pipeline_statistics_staging_range: Range<u64>,
    map_receiver: Option<Receiver<Result<(), wgpu::BufferAsyncError>>>,
}

#[derive(Clone, Copy)]
struct ActiveNativeDiagnosticQueryFrame {
    frame_index: u64,
    timestamp_period_ns: f32,
}
