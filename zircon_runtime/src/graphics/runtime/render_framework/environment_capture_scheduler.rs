use std::collections::{HashMap, VecDeque};

use crate::core::framework::render::{
    RenderEnvironmentCaptureHandle, RenderEnvironmentCaptureOutputIdentity,
    RenderEnvironmentCapturePhase, RenderEnvironmentCaptureRequest,
    RenderEnvironmentCaptureSourcePayload, RenderEnvironmentCaptureStatus, RenderFrameworkError,
    SceneViewportRenderPacket, RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
};

mod completion;
mod control_plane;

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
    ready_source_payload: Option<RenderEnvironmentCaptureSourcePayload>,
    telemetry: EnvironmentCaptureSchedulerTelemetry,
}

struct QueuedEnvironmentCapture {
    handle: RenderEnvironmentCaptureHandle,
    scene: SceneViewportRenderPacket,
    request: RenderEnvironmentCaptureRequest,
}

struct ActiveEnvironmentCapture {
    handle: RenderEnvironmentCaptureHandle,
    request: RenderEnvironmentCaptureRequest,
    completed_work_items: u32,
    phase: RenderEnvironmentCapturePhase,
    terminal_intent: Option<RenderEnvironmentCapturePhase>,
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
            completed_work_items: 0,
            phase: RenderEnvironmentCapturePhase::Capturing,
            terminal_intent: None,
        });
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
mod tests {
    use super::*;
    use crate::core::framework::render::{
        EnvironmentExtract, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderOverlayExtract,
        RenderSceneGeometryExtract, ViewportCameraSnapshot,
    };
    use crate::core::math::Vec4;

    #[test]
    fn queue_retains_at_most_one_pending_scene_snapshot() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        scheduler.request(scene(), request("a", 1)).unwrap();

        assert_eq!(
            scheduler.request(scene(), request("b", 1)),
            Err(RenderFrameworkError::EnvironmentCaptureQueueCapacityExceeded { limit: 1 })
        );
        assert_eq!(scheduler.telemetry().pending_capture_count, 1);
        assert_eq!(scheduler.telemetry().capacity_rejection_count, 1);
    }

    #[test]
    fn duplicate_reuses_handle_and_new_generation_supersedes_queued_work() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        let first = scheduler.request(scene(), request("probe", 1)).unwrap();
        assert_eq!(
            scheduler.request(scene(), request("probe", 1)).unwrap(),
            first
        );

        let second = scheduler.request(scene(), request("probe", 2)).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            scheduler.poll(first).unwrap().phase(),
            RenderEnvironmentCapturePhase::Superseded
        );
        assert_eq!(
            scheduler.poll(second).unwrap().phase(),
            RenderEnvironmentCapturePhase::Queued
        );
        assert_eq!(scheduler.telemetry().duplicate_request_count, 1);
        assert_eq!(scheduler.telemetry().superseded_capture_count, 1);
    }

    #[test]
    fn superseded_active_gpu_work_cannot_publish_output() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        let first = scheduler.request(scene(), request("probe", 1)).unwrap();
        let work = scheduler.begin_next().unwrap();
        assert_eq!(work.handle(), first);
        assert_eq!(work.request().output_generation(), 1);
        assert!(work.scene().scene.meshes.is_empty());
        scheduler
            .advance_active(first, RenderEnvironmentCapturePhase::Filtering, 6)
            .unwrap();

        let second = scheduler.request(scene(), request("probe", 2)).unwrap();
        assert_eq!(
            finish_success(&mut scheduler, first, None).unwrap(),
            EnvironmentCapturePublication::Discard
        );
        assert_eq!(
            scheduler.poll(first).unwrap().phase(),
            RenderEnvironmentCapturePhase::Superseded
        );
        assert!(scheduler.poll(first).unwrap().output().is_none());

        scheduler.begin_next().unwrap();
        scheduler
            .advance_active(second, RenderEnvironmentCapturePhase::Filtering, 6)
            .unwrap();
        assert_eq!(
            finish_success(&mut scheduler, second, None).unwrap(),
            EnvironmentCapturePublication::Publish
        );
        let status = scheduler.poll(second).unwrap();
        assert_eq!(status.phase(), RenderEnvironmentCapturePhase::Succeeded);
        assert_eq!(status.output().unwrap().output_generation(), 2);
    }

    #[test]
    fn work_item_transfer_boundary_moves_scene_and_request_without_clone() {
        let source = include_str!("environment_capture_scheduler.rs");
        let method = source
            .split("fn into_parts(")
            .nth(1)
            .and_then(|source| source.split("pub(in crate::graphics) fn scene").next())
            .expect("consuming work-item transfer method");

        assert!(method.contains("self.handle"));
        assert!(method.contains("self.scene"));
        assert!(method.contains("self.request"));
        assert!(!method.contains("clone()"));
    }

    #[test]
    fn cancellation_is_idempotent_and_suppresses_active_success() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        let handle = scheduler.request(scene(), request("probe", 1)).unwrap();
        scheduler.begin_next().unwrap();
        scheduler.cancel(handle).unwrap();
        scheduler.cancel(handle).unwrap();
        assert_eq!(
            finish_success(&mut scheduler, handle, None).unwrap(),
            EnvironmentCapturePublication::Discard
        );
        scheduler.cancel(handle).unwrap();

        let status = scheduler.poll(handle).unwrap();
        assert_eq!(status.phase(), RenderEnvironmentCapturePhase::Cancelled);
        assert!(status.output().is_none());
        assert_eq!(scheduler.telemetry().cancellation_request_count, 1);
    }

    #[test]
    fn live_generation_rejects_stale_requests_and_handle_overflow_is_typed() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        scheduler.request(scene(), request("probe", 2)).unwrap();
        assert_eq!(
            scheduler.request(scene(), request("probe", 1)),
            Err(RenderFrameworkError::EnvironmentCaptureGenerationNotNewer {
                capture_id: "probe".to_string(),
                requested_generation: 1,
                live_generation: 2,
            })
        );

        let mut exhausted = EnvironmentCaptureScheduler::default();
        exhausted.set_next_handle_for_tests(0);
        assert_eq!(
            exhausted.request(scene(), request("probe", 1)),
            Err(RenderFrameworkError::EnvironmentCaptureHandleSpaceExhausted)
        );
    }

    #[test]
    fn terminal_generation_rejects_replayed_older_request() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        let handle = scheduler.request(scene(), request("probe", 2)).unwrap();
        scheduler.begin_next().unwrap();
        assert_eq!(
            finish_success(&mut scheduler, handle, None),
            Err(EnvironmentCaptureTransitionError::IncompleteSuccess {
                phase: RenderEnvironmentCapturePhase::Capturing,
                completed_work_items: 0,
            })
        );
        scheduler
            .advance_active(handle, RenderEnvironmentCapturePhase::Filtering, 6)
            .unwrap();
        assert_eq!(
            finish_success(&mut scheduler, handle, None).unwrap(),
            EnvironmentCapturePublication::Publish
        );

        assert_eq!(
            scheduler.request(scene(), request("probe", 1)),
            Err(RenderFrameworkError::EnvironmentCaptureGenerationNotNewer {
                capture_id: "probe".to_string(),
                requested_generation: 1,
                live_generation: 2,
            })
        );
    }

    #[test]
    fn terminal_history_is_bounded_and_progress_cannot_regress() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        let first = scheduler.request(scene(), request("probe-0", 1)).unwrap();
        scheduler.begin_next().unwrap();
        scheduler
            .advance_active(first, RenderEnvironmentCapturePhase::Capturing, 2)
            .unwrap();
        assert_eq!(
            scheduler.advance_active(first, RenderEnvironmentCapturePhase::Capturing, 1),
            Err(EnvironmentCaptureTransitionError::ProgressRegression {
                previous: 2,
                next: 1,
            })
        );
        assert_eq!(
            scheduler.advance_active(first, RenderEnvironmentCapturePhase::Filtering, 2),
            Ok(())
        );
        assert_eq!(
            scheduler.advance_active(first, RenderEnvironmentCapturePhase::Capturing, 2),
            Err(EnvironmentCaptureTransitionError::PhaseRegression {
                previous: RenderEnvironmentCapturePhase::Filtering,
                next: RenderEnvironmentCapturePhase::Capturing,
            })
        );
        scheduler.finish_active_failure(first, "test").unwrap();

        let mut latest = first;
        for index in 1..=ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY {
            latest = scheduler
                .request(scene(), request(&format!("probe-{index}"), 1))
                .unwrap();
            scheduler.begin_next().unwrap();
            scheduler.finish_active_failure(latest, "test").unwrap();
        }

        assert_eq!(
            scheduler.poll(first),
            Err(RenderFrameworkError::UnknownEnvironmentCaptureHandle {
                handle: first.get(),
            })
        );
        assert_eq!(
            scheduler.poll(latest).unwrap().phase(),
            RenderEnvironmentCapturePhase::Failed
        );
        assert_eq!(
            scheduler.telemetry().terminal_status_count,
            ENVIRONMENT_CAPTURE_TERMINAL_STATUS_CAPACITY
        );
        assert_eq!(scheduler.telemetry().terminal_status_eviction_count, 1);
    }

    #[test]
    fn framework_control_plane_does_not_drain_frames_or_lock_renderer_state() {
        let source = include_str!("render_framework_trait_binding/wgpu_framework.rs");
        let control_plane = source
            .split("fn request_environment_capture")
            .nth(1)
            .and_then(|source| source.split("fn capture_frame_if_newer").next())
            .expect("environment capture request/poll/cancel trait binding");

        assert!(control_plane.contains(".environment_captures"));
        assert!(!control_plane.contains("finish_submission"));
        assert!(!control_plane.contains("lock_operation"));
        assert!(!control_plane.contains("lock_state"));
    }

    #[test]
    fn persisted_source_payload_is_taken_once_and_backpressures_only_persistence_requests() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        let capture_request = persistence_request("atrium", 1);
        let handle = scheduler.request(scene(), capture_request.clone()).unwrap();
        scheduler.begin_next().unwrap();
        scheduler
            .advance_active(
                handle,
                RenderEnvironmentCapturePhase::Persisting,
                RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            )
            .unwrap();

        assert_eq!(
            finish_success(&mut scheduler, handle, None),
            Err(EnvironmentCaptureTransitionError::PersistenceSourcePayloadRequired)
        );
        assert_eq!(
            finish_success(
                &mut scheduler,
                handle,
                Some(source_payload(handle, &capture_request)),
            )
            .unwrap(),
            EnvironmentCapturePublication::Publish
        );
        assert_eq!(
            scheduler.request(scene(), persistence_request("lobby", 1)),
            Err(
                RenderFrameworkError::EnvironmentCapturePersistenceResultCapacityExceeded {
                    limit: 1,
                }
            )
        );
        assert!(scheduler
            .request(scene(), request("runtime-only", 1))
            .is_ok());

        let payload = scheduler.take_source_payload(handle).unwrap().unwrap();
        assert_eq!(payload.handle(), handle);
        assert!(scheduler.take_source_payload(handle).unwrap().is_none());
        assert_eq!(scheduler.telemetry().source_payload_backpressure_count, 1);
        assert_eq!(scheduler.telemetry().source_payload_take_count, 1);
    }

    #[test]
    fn runtime_only_success_preserves_an_unconsumed_persistence_payload() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        let persistence_request = persistence_request("atrium", 1);
        let persistence_handle = scheduler
            .request(scene(), persistence_request.clone())
            .unwrap();
        scheduler.begin_next().unwrap();
        scheduler
            .advance_active(
                persistence_handle,
                RenderEnvironmentCapturePhase::Persisting,
                RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            )
            .unwrap();
        assert_eq!(
            finish_success(
                &mut scheduler,
                persistence_handle,
                Some(source_payload(persistence_handle, &persistence_request)),
            )
            .unwrap(),
            EnvironmentCapturePublication::Publish
        );

        let runtime_handle = scheduler
            .request(scene(), request("runtime-only", 1))
            .unwrap();
        scheduler.begin_next().unwrap();
        scheduler
            .advance_active(
                runtime_handle,
                RenderEnvironmentCapturePhase::Filtering,
                RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            )
            .unwrap();
        assert_eq!(
            finish_success(&mut scheduler, runtime_handle, None).unwrap(),
            EnvironmentCapturePublication::Publish
        );

        let payload = scheduler
            .take_source_payload(persistence_handle)
            .unwrap()
            .expect("runtime-only success must not clear the persistence mailbox");
        assert_eq!(payload.handle(), persistence_handle);
    }

    #[test]
    fn cancellation_discards_a_completed_source_payload_before_publication() {
        let mut scheduler = EnvironmentCaptureScheduler::default();
        let capture_request = persistence_request("atrium", 1);
        let handle = scheduler.request(scene(), capture_request.clone()).unwrap();
        scheduler.begin_next().unwrap();
        scheduler
            .advance_active(
                handle,
                RenderEnvironmentCapturePhase::Persisting,
                RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            )
            .unwrap();
        scheduler.cancel(handle).unwrap();

        assert_eq!(
            finish_success(
                &mut scheduler,
                handle,
                Some(source_payload(handle, &capture_request)),
            )
            .unwrap(),
            EnvironmentCapturePublication::Discard
        );
        assert_eq!(
            scheduler.poll(handle).unwrap().phase(),
            RenderEnvironmentCapturePhase::Cancelled
        );
        assert!(scheduler.take_source_payload(handle).unwrap().is_none());
    }

    #[test]
    fn gpu_framework_work_item_entrypoint_only_takes_scheduler_lock() {
        let source = include_str!("wgpu_render_framework/wgpu_render_framework.rs");
        let framework_impl = source
            .split("impl WgpuRenderFramework {")
            .nth(1)
            .expect("WGPU framework implementation");
        let method = framework_impl
            .split("fn begin_environment_capture_work_item(")
            .nth(1)
            .and_then(|source| source.split("/// Publishes capture progress").next())
            .expect("framework work-item entrypoint");

        assert!(method.contains("environment_captures"));
        assert!(method.contains("begin_next()"));
        assert!(!method.contains("lock_operation"));
        assert!(!method.contains("lock_state"));
        assert!(!method.contains("finish_submission"));
    }

    #[test]
    fn gpu_framework_progress_entrypoints_only_touch_scheduler_state() {
        let source = include_str!("wgpu_render_framework/wgpu_render_framework.rs");
        let framework_impl = source
            .split("impl WgpuRenderFramework {")
            .nth(1)
            .expect("WGPU framework implementation");
        let method = framework_impl
            .split("fn advance_environment_capture_work_item(")
            .nth(1)
            .and_then(|source| source.split("/// Publishes physical output").next())
            .expect("framework progress entrypoint");
        assert!(method.contains("environment_captures"));
        assert!(method.contains("advance_active"));
        assert!(!method.contains("lock_operation"));
        assert!(!method.contains("lock_state"));
        assert!(!method.contains("finish_submission"));

        let success = framework_impl
            .split("fn settle_environment_capture_work_item_success(")
            .nth(1)
            .and_then(|source| source.split("/// Records a recorder").next())
            .expect("framework success entrypoint");
        assert!(success.contains("environment_captures"));
        assert!(success.contains("finish_active_success_with_publication"));
        assert!(!success.contains("lock_operation"));
        assert!(!success.contains("lock_state"));

        let failure = framework_impl
            .split("fn finish_environment_capture_work_item_failure(")
            .nth(1)
            .and_then(|source| source.split("/// Compiles an existing Plan08").next())
            .expect("framework failure entrypoint");
        assert!(failure.contains("finish_active_failure"));
        assert!(!failure.contains("lock_operation"));
        assert!(!failure.contains("lock_state"));
    }

    fn request(capture_id: &str, output_generation: u64) -> RenderEnvironmentCaptureRequest {
        RenderEnvironmentCaptureRequest::with_revisions(
            capture_id,
            [0.0; 3],
            0.1,
            200.0,
            128,
            crate::core::framework::render::SourceCubemapPrefilterQuality::Normal,
            output_generation,
            output_generation,
            output_generation,
        )
        .unwrap()
    }

    fn persistence_request(
        capture_id: &str,
        output_generation: u64,
    ) -> RenderEnvironmentCaptureRequest {
        RenderEnvironmentCaptureRequest::with_revisions(
            capture_id,
            [0.0; 3],
            0.1,
            200.0,
            16,
            crate::core::framework::render::SourceCubemapPrefilterQuality::Normal,
            output_generation,
            output_generation,
            output_generation,
        )
        .unwrap()
        .with_persistence_output_uri(format!("res://probes/{capture_id}.zcube"))
        .unwrap()
    }

    fn source_payload(
        handle: RenderEnvironmentCaptureHandle,
        request: &RenderEnvironmentCaptureRequest,
    ) -> RenderEnvironmentCaptureSourcePayload {
        let mip_count =
            crate::core::framework::render::source_cubemap_mip_count(request.face_size());
        let byte_len = crate::core::framework::render::source_cubemap_sample_count(
            request.face_size(),
            mip_count,
        ) * crate::core::framework::render::RGBA16F_TEXEL_SIZE_BYTES;
        RenderEnvironmentCaptureSourcePayload::new(
            handle,
            RenderEnvironmentCaptureOutputIdentity::from_request(request),
            request.face_size(),
            mip_count,
            vec![0; byte_len],
        )
        .unwrap()
    }

    fn finish_success(
        scheduler: &mut EnvironmentCaptureScheduler,
        handle: RenderEnvironmentCaptureHandle,
        source_payload: Option<RenderEnvironmentCaptureSourcePayload>,
    ) -> Result<EnvironmentCapturePublication, EnvironmentCaptureTransitionError> {
        let mut publication = None;
        scheduler.finish_active_success_with_publication(
            handle,
            source_payload,
            |disposition, scheduler_before_publication| {
                let status = scheduler_before_publication.poll(handle).unwrap();
                assert!(!status.phase().is_terminal());
                assert!(!scheduler_before_publication
                    .ready_source_payload
                    .as_ref()
                    .is_some_and(|payload| payload.handle() == handle));
                publication = Some(disposition);
            },
        )?;
        Ok(publication.expect("success settlement must publish or discard exactly once"))
    }

    fn scene() -> SceneViewportRenderPacket {
        SceneViewportRenderPacket {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            environment: EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        }
    }
}
