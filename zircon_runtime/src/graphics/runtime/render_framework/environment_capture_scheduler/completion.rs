use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics::runtime::render_framework) enum EnvironmentCapturePublication {
    Publish,
    Discard,
}

impl EnvironmentCaptureScheduler {
    pub(in crate::graphics::runtime::render_framework) fn finish_active_success_with_publication<
        F,
    >(
        &mut self,
        handle: RenderEnvironmentCaptureHandle,
        source_payload: Option<RenderEnvironmentCaptureSourcePayload>,
        publication: F,
    ) -> Result<(), EnvironmentCaptureTransitionError>
    where
        F: FnOnce(EnvironmentCapturePublication, &EnvironmentCaptureScheduler),
    {
        self.validate_success(handle, source_payload.as_ref())?;

        let active = self.take_active(handle)?;
        if let Some(terminal_intent) = active.terminal_intent {
            publication(EnvironmentCapturePublication::Discard, self);
            self.publish_terminal(
                handle,
                terminal_intent,
                active.completed_work_items,
                None,
                Some(terminal_intent_diagnostic(terminal_intent)),
            );
            self.advance_observation();
            return Ok(());
        }

        publication(EnvironmentCapturePublication::Publish, self);
        if let Some(source_payload) = source_payload {
            self.publish_source_payload(source_payload, active.bake_key);
        }
        let output = RenderEnvironmentCaptureOutputIdentity::from_request(&active.request);
        self.publish_terminal(
            handle,
            RenderEnvironmentCapturePhase::Succeeded,
            RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            Some(output),
            None,
        );
        self.telemetry.succeeded_capture_count =
            self.telemetry.succeeded_capture_count.saturating_add(1);
        self.advance_observation();
        Ok(())
    }

    fn validate_success(
        &self,
        handle: RenderEnvironmentCaptureHandle,
        source_payload: Option<&RenderEnvironmentCaptureSourcePayload>,
    ) -> Result<(), EnvironmentCaptureTransitionError> {
        let active = self
            .active
            .as_ref()
            .ok_or(EnvironmentCaptureTransitionError::NoActiveCapture)?;
        if active.handle != handle {
            return Err(EnvironmentCaptureTransitionError::HandleMismatch);
        }
        if active.terminal_intent.is_some() {
            return Ok(());
        }
        if active.completed_work_items != RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT
            || !matches!(
                active.phase,
                RenderEnvironmentCapturePhase::Filtering
                    | RenderEnvironmentCapturePhase::Persisting
            )
        {
            return Err(EnvironmentCaptureTransitionError::IncompleteSuccess {
                phase: active.phase,
                completed_work_items: active.completed_work_items,
            });
        }

        let Some(payload) = source_payload else {
            return if active.request.persistence_output_uri().is_some() {
                Err(EnvironmentCaptureTransitionError::PersistenceSourcePayloadRequired)
            } else {
                Ok(())
            };
        };
        if active.phase != RenderEnvironmentCapturePhase::Persisting {
            return Err(EnvironmentCaptureTransitionError::InvalidPhase(
                active.phase,
            ));
        }
        if self.ready_source_payload.is_some() {
            return Err(EnvironmentCaptureTransitionError::SourcePayloadCapacityExceeded);
        }
        if payload.handle() != handle {
            return Err(EnvironmentCaptureTransitionError::SourcePayloadHandleMismatch);
        }
        let output = RenderEnvironmentCaptureOutputIdentity::from_request(&active.request);
        if payload.output() != &output {
            return Err(EnvironmentCaptureTransitionError::SourcePayloadOutputMismatch);
        }
        if payload.face_size() != active.request.face_size()
            || payload.mip_count()
                != crate::core::framework::render::source_cubemap_mip_count(
                    active.request.face_size(),
                )
        {
            return Err(EnvironmentCaptureTransitionError::SourcePayloadLayoutMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/completion.rs"]
mod tests;
