use zircon_plugin_rendering_reflection_probes_runtime::{
    cancel_reflection_probe_capture, poll_reflection_probe_capture,
    request_reflection_probe_capture, request_reflection_probe_capture_with_placement,
    take_reflection_probe_capture_source, CapturedReflectionProbeConsumeError,
    CapturedReflectionProbePlacement, ReflectionProbeCaptureError, ReflectionProbeCaptureRequest,
    ReflectionProbeCaptureRequestError,
};
use zircon_runtime::core::framework::render::{
    RenderEnvironmentCaptureHandle, RenderEnvironmentCaptureSourcePayload,
    RenderEnvironmentCaptureStatus, RenderFramework, RenderSceneSnapshot,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectionProbeCaptureEditorCommand {
    request_json: String,
    placement_json: Option<String>,
}

impl ReflectionProbeCaptureEditorCommand {
    pub fn from_request(
        request: &ReflectionProbeCaptureRequest,
    ) -> Result<Self, ReflectionProbeCaptureRequestError> {
        Ok(Self {
            request_json: request.encode_json()?,
            placement_json: None,
        })
    }

    pub fn from_request_and_placement(
        request: &ReflectionProbeCaptureRequest,
        placement: &CapturedReflectionProbePlacement,
    ) -> Result<Self, ReflectionProbeCaptureEditorCommandError> {
        Ok(Self {
            request_json: request.encode_json()?,
            placement_json: Some(placement.encode_json()?),
        })
    }

    pub fn request(
        &self,
    ) -> Result<ReflectionProbeCaptureRequest, ReflectionProbeCaptureRequestError> {
        ReflectionProbeCaptureRequest::decode_json(&self.request_json)
    }

    pub fn request_json(&self) -> &str {
        &self.request_json
    }

    pub fn placement(
        &self,
    ) -> Result<Option<CapturedReflectionProbePlacement>, CapturedReflectionProbeConsumeError> {
        self.placement_json
            .as_deref()
            .map(CapturedReflectionProbePlacement::decode_json)
            .transpose()
    }

    pub fn placement_json(&self) -> Option<&str> {
        self.placement_json.as_deref()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReflectionProbeCaptureEditorTrigger;

impl ReflectionProbeCaptureEditorTrigger {
    pub const fn new() -> Self {
        Self
    }

    pub fn submit(
        &self,
        framework: &dyn RenderFramework,
        scene: &RenderSceneSnapshot,
        command: &ReflectionProbeCaptureEditorCommand,
    ) -> Result<RenderEnvironmentCaptureHandle, ReflectionProbeCaptureEditorExecutionError> {
        let request = command.request()?;
        request_reflection_probe_capture(framework, scene, &request).map_err(Into::into)
    }

    pub fn submit_with_placement(
        &self,
        framework: &dyn RenderFramework,
        scene: &RenderSceneSnapshot,
        command: &ReflectionProbeCaptureEditorCommand,
    ) -> Result<ReflectionProbeCaptureEditorResult, ReflectionProbeCaptureEditorExecutionError>
    {
        let placement = command
            .placement()?
            .ok_or(ReflectionProbeCaptureEditorExecutionError::MissingPlacement)?;
        let request = command.request()?;
        let handle = request_reflection_probe_capture_with_placement(
            framework, scene, &request, &placement,
        )?;
        Ok(ReflectionProbeCaptureEditorResult { handle, placement })
    }

    pub fn poll(
        &self,
        framework: &dyn RenderFramework,
        handle: RenderEnvironmentCaptureHandle,
    ) -> Result<RenderEnvironmentCaptureStatus, ReflectionProbeCaptureEditorExecutionError> {
        poll_reflection_probe_capture(framework, handle).map_err(Into::into)
    }

    pub fn cancel(
        &self,
        framework: &dyn RenderFramework,
        handle: RenderEnvironmentCaptureHandle,
    ) -> Result<(), ReflectionProbeCaptureEditorExecutionError> {
        cancel_reflection_probe_capture(framework, handle).map_err(Into::into)
    }

    pub fn take_source_payload(
        &self,
        framework: &dyn RenderFramework,
        handle: RenderEnvironmentCaptureHandle,
    ) -> Result<
        Option<RenderEnvironmentCaptureSourcePayload>,
        ReflectionProbeCaptureEditorExecutionError,
    > {
        take_reflection_probe_capture_source(framework, handle).map_err(Into::into)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReflectionProbeCaptureEditorResult {
    pub handle: RenderEnvironmentCaptureHandle,
    pub placement: CapturedReflectionProbePlacement,
}

#[derive(Debug, thiserror::Error)]
pub enum ReflectionProbeCaptureEditorCommandError {
    #[error(transparent)]
    Request(#[from] ReflectionProbeCaptureRequestError),
    #[error(transparent)]
    Placement(#[from] CapturedReflectionProbeConsumeError),
}

#[derive(Debug, thiserror::Error)]
pub enum ReflectionProbeCaptureEditorExecutionError {
    #[error(transparent)]
    Request(#[from] ReflectionProbeCaptureRequestError),
    #[error(transparent)]
    Capture(#[from] ReflectionProbeCaptureError),
    #[error(transparent)]
    Placement(#[from] CapturedReflectionProbeConsumeError),
    #[error("reflection-probe capture editor command has no placement metadata")]
    MissingPlacement,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_trigger_is_nonblocking_and_does_not_own_runtime_resources() {
        let source = include_str!("trigger.rs");
        assert!(source.contains("pub fn submit"));
        assert!(source.contains("pub fn poll"));
        assert!(source.contains("pub fn cancel"));
        assert!(source.contains("pub fn take_source_payload"));
        assert!(!source.contains(&["Scene", "Renderer"].concat()));
        assert!(!source.contains(&["cache", "_root"].concat()));
        assert!(!source.contains(&["ProjectAsset", "Manager"].concat()));
        assert!(!source.contains(&["capture_and_persist", "_reflection_probe"].concat()));
    }

    #[test]
    fn editor_command_keeps_runtime_capture_request_serialized() {
        let request = ReflectionProbeCaptureRequest::new(
            "atrium",
            "lib://probes/atrium.zcube",
            [3.0, 1.5, -2.0],
            9,
        );
        let command = ReflectionProbeCaptureEditorCommand::from_request(&request).unwrap();

        assert_eq!(command.request().unwrap(), request);
        assert!(command.request_json().contains("\"schema_version\": 2"));
        assert!(command.placement().unwrap().is_none());
    }
}
