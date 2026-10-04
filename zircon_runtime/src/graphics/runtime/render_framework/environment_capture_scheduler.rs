use std::collections::{HashMap, VecDeque};

use crate::core::framework::render::{
    IblBakeKey, RenderEnvironmentCaptureHandle, RenderEnvironmentCaptureOutputIdentity,
    RenderEnvironmentCapturePhase, RenderEnvironmentCaptureRequest,
    RenderEnvironmentCaptureSourcePayload, RenderEnvironmentCaptureStatus, RenderFrameworkError,
    SceneViewportRenderPacket, RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
};

mod completion;
mod control_plane;
mod report;

pub(in crate::graphics::runtime::render_framework) use completion::EnvironmentCapturePublication;

const ENVIRONMENT_CAPTURE_PENDING_CAPACITY: usize = 1;
const ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY: usize = 64;
const ENVIRONMENT_CAPTURE_SOURCE_PAYLOAD_CAPACITY: usize = 1;

pub(in crate::graphics::runtime::render_framework) struct EnvironmentCaptureScheduler {
    next_handle: u64,
    pending: VecDeque<QueuedEnvironmentCapture>,
    active: Option<ActiveEnvironmentCapture>,
    statuses: HashMap<RenderEnvironmentCaptureHandle, RenderEnvironmentCaptureStatus>,
    terminal_order: VecDeque<RenderEnvironmentCaptureHandle>,
    latest_generations: HashMap<String, u64>,
    generation_order: VecDeque<String>,
    ready_source_payload: Option<ReadyEnvironmentCaptureSourcePayload>,
    telemetry: EnvironmentCaptureSchedulerTelemetry,
    observation_epoch: u64,
}

struct QueuedEnvironmentCapture {
    handle: RenderEnvironmentCaptureHandle,
    scene: SceneViewportRenderPacket,
    request: RenderEnvironmentCaptureRequest,
    bake_key: IblBakeKey,
}

struct ActiveEnvironmentCapture {
    handle: RenderEnvironmentCaptureHandle,
    request: RenderEnvironmentCaptureRequest,
    bake_key: IblBakeKey,
    completed_work_items: u32,
    phase: RenderEnvironmentCapturePhase,
    terminal_intent: Option<RenderEnvironmentCapturePhase>,
}

struct ReadyEnvironmentCaptureSourcePayload {
    payload: RenderEnvironmentCaptureSourcePayload,
    bake_key: IblBakeKey,
    bytes: u64,
}

impl ReadyEnvironmentCaptureSourcePayload {
    fn new(payload: RenderEnvironmentCaptureSourcePayload, bake_key: IblBakeKey) -> Self {
        let bytes = u64::try_from(payload.source_rgba16f_bytes().len()).unwrap_or(u64::MAX);
        Self {
            payload,
            bake_key,
            bytes,
        }
    }

    fn handle(&self) -> RenderEnvironmentCaptureHandle {
        self.payload.handle()
    }

    fn into_payload(self) -> RenderEnvironmentCaptureSourcePayload {
        self.payload
    }
}

pub(in crate::graphics) struct EnvironmentCaptureWorkItem {
    handle: RenderEnvironmentCaptureHandle,
    scene: SceneViewportRenderPacket,
    request: RenderEnvironmentCaptureRequest,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::graphics) struct EnvironmentCaptureSchedulerTelemetry {
    pub accepted_request_count: u64,
    pub duplicate_request_count: u64,
    pub capacity_rejection_count: u64,
    pub stale_generation_rejection_count: u64,
    pub superseded_capture_count: u64,
    pub cancellation_request_count: u64,
    pub succeeded_capture_count: u64,
    pub failed_capture_count: u64,
    pub terminal_status_eviction_count: u64,
    pub source_payload_backpressure_count: u64,
    pub source_payload_take_count: u64,
    pub source_payload_publish_count: u64,
    pub peak_ready_source_payload_bytes: u64,
    pub cumulative_source_payload_bytes: u64,
    pub pending_capture_count: usize,
    pub active_capture_count: usize,
    pub terminal_status_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::graphics) enum EnvironmentCaptureTransitionError {
    NoActiveCapture,
    HandleMismatch,
    InvalidPhase(RenderEnvironmentCapturePhase),
    PhaseRegression {
        previous: RenderEnvironmentCapturePhase,
        next: RenderEnvironmentCapturePhase,
    },
    ProgressRegression {
        previous: u32,
        next: u32,
    },
    ProgressOutOfRange(u32),
    IncompleteSuccess {
        phase: RenderEnvironmentCapturePhase,
        completed_work_items: u32,
    },
    PersistenceSourcePayloadRequired,
    SourcePayloadCapacityExceeded,
    SourcePayloadHandleMismatch,
    SourcePayloadOutputMismatch,
    SourcePayloadLayoutMismatch,
}

impl Default for EnvironmentCaptureScheduler {
    fn default() -> Self {
        Self {
            next_handle: 1,
            pending: VecDeque::with_capacity(ENVIRONMENT_CAPTURE_PENDING_CAPACITY),
            active: None,
            statuses: HashMap::with_capacity(
                ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY
                    + ENVIRONMENT_CAPTURE_PENDING_CAPACITY
                    + 1,
            ),
            terminal_order: VecDeque::with_capacity(ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY),
            latest_generations: HashMap::with_capacity(
                ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY,
            ),
            generation_order: VecDeque::with_capacity(ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY),
            ready_source_payload: None,
            telemetry: EnvironmentCaptureSchedulerTelemetry::default(),
            observation_epoch: 0,
        }
    }
}

impl EnvironmentCaptureScheduler {
    pub(in crate::graphics) fn begin_next(&mut self) -> Option<EnvironmentCaptureWorkItem> {
        if self.active.is_some() {
            return None;
        }
        let queued = self.pending.pop_front()?;
        let status = RenderEnvironmentCaptureStatus::new(
            queued.handle,
            RenderEnvironmentCapturePhase::Capturing,
            0,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            None,
            None,
        )
        .expect("capture start status must be valid");
        self.statuses.insert(queued.handle, status);
        self.active = Some(ActiveEnvironmentCapture {
            handle: queued.handle,
            request: queued.request.clone(),
            bake_key: queued.bake_key,
            completed_work_items: 0,
            phase: RenderEnvironmentCapturePhase::Capturing,
            terminal_intent: None,
        });
        self.advance_observation();
        Some(EnvironmentCaptureWorkItem {
            handle: queued.handle,
            scene: queued.scene,
            request: queued.request,
        })
    }

    pub(in crate::graphics) fn advance_active(
        &mut self,
        handle: RenderEnvironmentCaptureHandle,
        phase: RenderEnvironmentCapturePhase,
        completed_work_items: u32,
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        if !matches!(
            phase,
            RenderEnvironmentCapturePhase::Capturing
                | RenderEnvironmentCapturePhase::Filtering
                | RenderEnvironmentCapturePhase::Persisting
        ) {
            return Err(EnvironmentCaptureTransitionError::InvalidPhase(phase));
        }
        let active = self
            .active
            .as_mut()
            .ok_or(EnvironmentCaptureTransitionError::NoActiveCapture)?;
        if active.handle != handle {
            return Err(EnvironmentCaptureTransitionError::HandleMismatch);
        }
        if completed_work_items > RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT {
            return Err(EnvironmentCaptureTransitionError::ProgressOutOfRange(
                completed_work_items,
            ));
        }
        if completed_work_items < active.completed_work_items {
            return Err(EnvironmentCaptureTransitionError::ProgressRegression {
                previous: active.completed_work_items,
                next: completed_work_items,
            });
        }
        if phase_rank(phase) < phase_rank(active.phase) {
            return Err(EnvironmentCaptureTransitionError::PhaseRegression {
                previous: active.phase,
                next: phase,
            });
        }
        active.phase = phase;
        active.completed_work_items = completed_work_items;
        let diagnostic = active.terminal_intent.map(terminal_intent_diagnostic);
        self.statuses.insert(
            handle,
            RenderEnvironmentCaptureStatus::new(
                handle,
                phase,
                completed_work_items,
                RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
                None,
                diagnostic,
            )
            .expect("active environment capture status must be valid"),
        );
        self.advance_observation();
        Ok(())
    }

    pub(in crate::graphics) fn finish_active_failure(
        &mut self,
        handle: RenderEnvironmentCaptureHandle,
        diagnostic: impl Into<String>,
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        let active = self.take_active(handle)?;
        if let Some(terminal_intent) = active.terminal_intent {
            self.publish_terminal(
                handle,
                terminal_intent,
                active.completed_work_items,
                None,
                Some(terminal_intent_diagnostic(terminal_intent)),
            );
        } else {
            self.publish_terminal(
                handle,
                RenderEnvironmentCapturePhase::Failed,
                active.completed_work_items,
                None,
                Some(diagnostic.into()),
            );
            self.telemetry.failed_capture_count =
                self.telemetry.failed_capture_count.saturating_add(1);
        }
        self.advance_observation();
        Ok(())
    }

    pub(in crate::graphics) fn telemetry(&self) -> EnvironmentCaptureSchedulerTelemetry {
        EnvironmentCaptureSchedulerTelemetry {
            pending_capture_count: self.pending.len(),
            active_capture_count: usize::from(self.active.is_some()),
            terminal_status_count: self.terminal_order.len(),
            ..self.telemetry
        }
    }

    fn advance_observation(&mut self) {
        self.observation_epoch = self.observation_epoch.saturating_add(1);
    }

    fn publish_source_payload(
        &mut self,
        payload: RenderEnvironmentCaptureSourcePayload,
        bake_key: IblBakeKey,
    ) {
        let ready = ReadyEnvironmentCaptureSourcePayload::new(payload, bake_key);
        let payload_bytes = ready.bytes;
        self.telemetry.source_payload_publish_count = self
            .telemetry
            .source_payload_publish_count
            .saturating_add(1);
        self.telemetry.peak_ready_source_payload_bytes = self
            .telemetry
            .peak_ready_source_payload_bytes
            .max(payload_bytes);
        self.telemetry.cumulative_source_payload_bytes = self
            .telemetry
            .cumulative_source_payload_bytes
            .saturating_add(payload_bytes);
        self.ready_source_payload = Some(ready);
    }

    fn allocate_handle(&mut self) -> Result<RenderEnvironmentCaptureHandle, RenderFrameworkError> {
        let handle = RenderEnvironmentCaptureHandle::new(self.next_handle)
            .ok_or(RenderFrameworkError::EnvironmentCaptureHandleSpaceExhausted)?;
        self.next_handle = self.next_handle.checked_add(1).unwrap_or(0);
        Ok(handle)
    }

    fn duplicate_live_handle(
        &self,
        request: &RenderEnvironmentCaptureRequest,
    ) -> Option<RenderEnvironmentCaptureHandle> {
        self.active
            .as_ref()
            .filter(|active| active.terminal_intent.is_none() && active.request == *request)
            .map(|active| active.handle)
            .or_else(|| {
                self.pending
                    .iter()
                    .find(|job| job.request == *request)
                    .map(|job| job.handle)
            })
    }

    fn latest_generation(&self, capture_id: &str) -> Option<u64> {
        self.latest_generations
            .get(capture_id)
            .copied()
            .or_else(|| {
                self.active
                    .as_ref()
                    .filter(|active| active.request.capture_id() == capture_id)
                    .map(|active| active.request.output_generation())
                    .into_iter()
                    .chain(
                        self.pending
                            .iter()
                            .filter(|job| job.request.capture_id() == capture_id)
                            .map(|job| job.request.output_generation()),
                    )
                    .max()
            })
    }

    fn remember_generation(&mut self, request: &RenderEnvironmentCaptureRequest) {
        let capture_id = request.capture_id().to_string();
        self.latest_generations
            .insert(capture_id.clone(), request.output_generation());
        if let Some(index) = self
            .generation_order
            .iter()
            .position(|known| known == &capture_id)
        {
            self.generation_order.remove(index);
        }
        self.generation_order.push_back(capture_id);
        while self.generation_order.len() > ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY {
            if let Some(evicted) = self.generation_order.pop_front() {
                self.latest_generations.remove(&evicted);
            }
        }
    }

    fn set_active_terminal_intent(
        &mut self,
        phase: RenderEnvironmentCapturePhase,
        diagnostic: &'static str,
    ) {
        let (handle, active_phase, completed_work_items) = {
            let Some(active) = self.active.as_mut() else {
                return;
            };
            if active.terminal_intent == Some(RenderEnvironmentCapturePhase::Cancelled) {
                return;
            }
            active.terminal_intent = Some(phase);
            (active.handle, active.phase, active.completed_work_items)
        };
        let status = RenderEnvironmentCaptureStatus::new(
            handle,
            active_phase,
            completed_work_items,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            None,
            Some(diagnostic.to_string()),
        )
        .expect("terminal-intent environment capture status must be valid");
        self.statuses.insert(handle, status);
    }

    fn take_active(
        &mut self,
        handle: RenderEnvironmentCaptureHandle,
    ) -> Result<ActiveEnvironmentCapture, EnvironmentCaptureTransitionError> {
        let active = self
            .active
            .take()
            .ok_or(EnvironmentCaptureTransitionError::NoActiveCapture)?;
        if active.handle == handle {
            Ok(active)
        } else {
            self.active = Some(active);
            Err(EnvironmentCaptureTransitionError::HandleMismatch)
        }
    }

    fn publish_terminal(
        &mut self,
        handle: RenderEnvironmentCaptureHandle,
        phase: RenderEnvironmentCapturePhase,
        completed_work_items: u32,
        output: Option<RenderEnvironmentCaptureOutputIdentity>,
        diagnostic: Option<String>,
    ) {
        let status = RenderEnvironmentCaptureStatus::new(
            handle,
            phase,
            completed_work_items,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            output,
            diagnostic,
        )
        .expect("terminal environment capture status must be valid");
        self.statuses.insert(handle, status);
        self.terminal_order.push_back(handle);
        match phase {
            RenderEnvironmentCapturePhase::Superseded => {
                self.telemetry.superseded_capture_count =
                    self.telemetry.superseded_capture_count.saturating_add(1);
            }
            RenderEnvironmentCapturePhase::Cancelled => {}
            _ => {}
        }
        while self.terminal_order.len() > ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY {
            if let Some(evicted) = self.terminal_order.pop_front() {
                self.statuses.remove(&evicted);
                self.telemetry.terminal_status_eviction_count = self
                    .telemetry
                    .terminal_status_eviction_count
                    .saturating_add(1);
            }
        }
    }

    #[cfg(test)]
    fn set_next_handle_for_tests(&mut self, next_handle: u64) {
        self.next_handle = next_handle;
    }
}

impl EnvironmentCaptureWorkItem {
    pub(in crate::graphics) const fn handle(&self) -> RenderEnvironmentCaptureHandle {
        self.handle
    }

    /// Transfers the queued scene snapshot and request to the GPU recorder.
    ///
    /// The scheduler must relinquish ownership before resource preparation or
    /// command recording starts. Keeping this as a consuming boundary prevents
    /// a recorder from retaining the scheduler mutex while it moves the scene
    /// into `EnvironmentCaptureSceneBatch`.
    pub(in crate::graphics) fn into_parts(
        self,
    ) -> (
        RenderEnvironmentCaptureHandle,
        SceneViewportRenderPacket,
        RenderEnvironmentCaptureRequest,
    ) {
        (self.handle, self.scene, self.request)
    }

    pub(in crate::graphics) fn scene(&self) -> &SceneViewportRenderPacket {
        &self.scene
    }

    pub(in crate::graphics) fn request(&self) -> &RenderEnvironmentCaptureRequest {
        &self.request
    }
}

fn terminal_intent_diagnostic(phase: RenderEnvironmentCapturePhase) -> String {
    match phase {
        RenderEnvironmentCapturePhase::Cancelled => {
            "capture completed after cancellation; output was not published".to_string()
        }
        RenderEnvironmentCapturePhase::Superseded => {
            "capture completed after supersession; output was not published".to_string()
        }
        _ => "capture output publication was suppressed".to_string(),
    }
}

const fn phase_rank(phase: RenderEnvironmentCapturePhase) -> u8 {
    match phase {
        RenderEnvironmentCapturePhase::Capturing => 0,
        RenderEnvironmentCapturePhase::Filtering => 1,
        RenderEnvironmentCapturePhase::Persisting => 2,
        _ => u8::MAX,
    }
}

#[cfg(test)]
#[path = "tests/environment_capture_scheduler.rs"]
mod tests;
