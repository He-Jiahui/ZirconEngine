use super::*;

impl EnvironmentCaptureScheduler {
    pub(in crate::graphics::runtime::render_framework) fn request(
        &mut self,
        scene: SceneViewportRenderPacket,
        request: RenderEnvironmentCaptureRequest,
    ) -> Result<RenderEnvironmentCaptureHandle, RenderFrameworkError> {
        if let Some(handle) = self.duplicate_live_handle(&request) {
            self.telemetry.duplicate_request_count =
                self.telemetry.duplicate_request_count.saturating_add(1);
            return Ok(handle);
        }
        if request.persistence_output_uri().is_some() && self.ready_source_payload.is_some() {
            self.telemetry.source_payload_backpressure_count = self
                .telemetry
                .source_payload_backpressure_count
                .saturating_add(1);
            return Err(
                RenderFrameworkError::EnvironmentCapturePersistenceResultCapacityExceeded {
                    limit: ENVIRONMENT_CAPTURE_SOURCE_PAYLOAD_CAPACITY,
                },
            );
        }
        if let Some(latest_generation) = self.latest_generation(request.capture_id()) {
            if request.output_generation() <= latest_generation {
                self.telemetry.stale_generation_rejection_count = self
                    .telemetry
                    .stale_generation_rejection_count
                    .saturating_add(1);
                return Err(RenderFrameworkError::EnvironmentCaptureGenerationNotNewer {
                    capture_id: request.capture_id().to_string(),
                    requested_generation: request.output_generation(),
                    live_generation: latest_generation,
                });
            }
        }

        let pending_match = self
            .pending
            .iter()
            .position(|job| job.request.capture_id() == request.capture_id());
        if pending_match.is_none() && self.pending.len() >= ENVIRONMENT_CAPTURE_PENDING_CAPACITY {
            self.telemetry.capacity_rejection_count =
                self.telemetry.capacity_rejection_count.saturating_add(1);
            return Err(
                RenderFrameworkError::EnvironmentCaptureQueueCapacityExceeded {
                    limit: ENVIRONMENT_CAPTURE_PENDING_CAPACITY,
                },
            );
        }

        let handle = self.allocate_handle()?;
        if let Some(index) = pending_match {
            let superseded = self
                .pending
                .remove(index)
                .expect("matched environment capture must still be queued");
            self.publish_terminal(
                superseded.handle,
                RenderEnvironmentCapturePhase::Superseded,
                0,
                None,
                Some("superseded by a newer queued generation".to_string()),
            );
        }
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.request.capture_id() == request.capture_id())
        {
            self.set_active_terminal_intent(
                RenderEnvironmentCapturePhase::Superseded,
                "superseded by a newer capture generation",
            );
        }

        self.statuses.insert(
            handle,
            RenderEnvironmentCaptureStatus::queued(handle)
                .expect("queued environment capture status must be valid"),
        );
        self.remember_generation(&request);
        self.pending.push_back(QueuedEnvironmentCapture {
            handle,
            scene,
            request,
        });
        self.telemetry.accepted_request_count =
            self.telemetry.accepted_request_count.saturating_add(1);
        Ok(handle)
    }

    pub(in crate::graphics::runtime::render_framework) fn poll(
        &self,
        handle: RenderEnvironmentCaptureHandle,
    ) -> Result<RenderEnvironmentCaptureStatus, RenderFrameworkError> {
        self.statuses.get(&handle).cloned().ok_or(
            RenderFrameworkError::UnknownEnvironmentCaptureHandle {
                handle: handle.get(),
            },
        )
    }

    pub(in crate::graphics::runtime::render_framework) fn take_source_payload(
        &mut self,
        handle: RenderEnvironmentCaptureHandle,
    ) -> Result<Option<RenderEnvironmentCaptureSourcePayload>, RenderFrameworkError> {
        if self
            .ready_source_payload
            .as_ref()
            .is_some_and(|payload| payload.handle() == handle)
        {
            self.telemetry.source_payload_take_count =
                self.telemetry.source_payload_take_count.saturating_add(1);
            return Ok(self.ready_source_payload.take());
        }
        if self.statuses.contains_key(&handle)
            || self.pending.iter().any(|queued| queued.handle == handle)
            || self
                .active
                .as_ref()
                .is_some_and(|active| active.handle == handle)
        {
            return Ok(None);
        }
        Err(RenderFrameworkError::UnknownEnvironmentCaptureHandle {
            handle: handle.get(),
        })
    }

    pub(in crate::graphics::runtime::render_framework) fn cancel(
        &mut self,
        handle: RenderEnvironmentCaptureHandle,
    ) -> Result<(), RenderFrameworkError> {
        if let Some(index) = self.pending.iter().position(|job| job.handle == handle) {
            let cancelled = self
                .pending
                .remove(index)
                .expect("matched environment capture must still be queued");
            self.telemetry.cancellation_request_count =
                self.telemetry.cancellation_request_count.saturating_add(1);
            self.publish_terminal(
                cancelled.handle,
                RenderEnvironmentCapturePhase::Cancelled,
                0,
                None,
                Some("cancelled before GPU recording".to_string()),
            );
            return Ok(());
        }
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.handle == handle)
        {
            if self.active.as_ref().is_some_and(|active| {
                active.terminal_intent == Some(RenderEnvironmentCapturePhase::Cancelled)
            }) {
                return Ok(());
            }
            self.telemetry.cancellation_request_count =
                self.telemetry.cancellation_request_count.saturating_add(1);
            self.set_active_terminal_intent(
                RenderEnvironmentCapturePhase::Cancelled,
                "cancellation requested while GPU work is active",
            );
            return Ok(());
        }
        if self
            .statuses
            .get(&handle)
            .is_some_and(|status| status.phase().is_terminal())
        {
            return Ok(());
        }
        Err(RenderFrameworkError::UnknownEnvironmentCaptureHandle {
            handle: handle.get(),
        })
    }
}
