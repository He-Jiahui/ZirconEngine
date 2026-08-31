use thiserror::Error;
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::framework::render::{
    RenderEnvironmentCaptureHandle, RenderEnvironmentCaptureSourcePayload,
    RenderEnvironmentCaptureStatus, RenderFramework, RenderFrameworkError, RenderSceneSnapshot,
};
use zircon_runtime::core::resource::ResourceId;

use super::{
    CapturedReflectionProbeConsumeError, CapturedReflectionProbePlacement,
    ReflectionProbeCaptureRequest, ReflectionProbeCaptureRequestError,
};

pub fn request_reflection_probe_capture(
    framework: &dyn RenderFramework,
    scene: &RenderSceneSnapshot,
    request: &ReflectionProbeCaptureRequest,
) -> Result<RenderEnvironmentCaptureHandle, ReflectionProbeCaptureError> {
    let render_request = request.render_request()?;
    framework
        .request_environment_capture(scene.clone(), render_request)
        .map_err(ReflectionProbeCaptureError::Framework)
}

pub fn request_reflection_probe_capture_with_placement(
    framework: &dyn RenderFramework,
    scene: &RenderSceneSnapshot,
    request: &ReflectionProbeCaptureRequest,
    placement: &CapturedReflectionProbePlacement,
) -> Result<RenderEnvironmentCaptureHandle, ReflectionProbeCaptureError> {
    request.validate()?;
    placement.validate()?;
    let pmrem_uri = AssetUri::parse(&placement.pmrem_uri)
        .map_err(|error| ReflectionProbeCaptureError::TargetResourceUri(error.to_string()))?;
    let cubemap = ResourceId::from_locator(&pmrem_uri);
    let render_request = request
        .render_request()?
        .with_reflection_probe_target(placement.probe_id, cubemap);
    framework
        .request_environment_capture(scene.clone(), render_request)
        .map_err(ReflectionProbeCaptureError::Framework)
}

pub fn poll_reflection_probe_capture(
    framework: &dyn RenderFramework,
    handle: RenderEnvironmentCaptureHandle,
) -> Result<RenderEnvironmentCaptureStatus, ReflectionProbeCaptureError> {
    framework
        .poll_environment_capture(handle)
        .map_err(ReflectionProbeCaptureError::Framework)
}

pub fn cancel_reflection_probe_capture(
    framework: &dyn RenderFramework,
    handle: RenderEnvironmentCaptureHandle,
) -> Result<(), ReflectionProbeCaptureError> {
    framework
        .cancel_environment_capture(handle)
        .map_err(ReflectionProbeCaptureError::Framework)
}

pub fn take_reflection_probe_capture_source(
    framework: &dyn RenderFramework,
    handle: RenderEnvironmentCaptureHandle,
) -> Result<Option<RenderEnvironmentCaptureSourcePayload>, ReflectionProbeCaptureError> {
    framework
        .take_environment_capture_source_payload(handle)
        .map_err(ReflectionProbeCaptureError::Framework)
}

#[derive(Debug, Error)]
pub enum ReflectionProbeCaptureError {
    #[error(transparent)]
    InvalidRequest(#[from] ReflectionProbeCaptureRequestError),
    #[error(transparent)]
    Framework(#[from] RenderFrameworkError),
    #[error(transparent)]
    Placement(#[from] CapturedReflectionProbeConsumeError),
    #[error("invalid captured reflection-probe target resource URI: {0}")]
    TargetResourceUri(String),
}

#[cfg(test)]
mod tests {
    #[test]
    fn capture_execution_is_a_nonblocking_framework_boundary() {
        let source = include_str!("execute.rs");
        assert!(source.contains("request_environment_capture"));
        assert!(source.contains("poll_environment_capture"));
        assert!(source.contains("cancel_environment_capture"));
        assert!(source.contains("take_environment_capture_source_payload"));
        assert!(!source.contains(&["Scene", "Renderer"].concat()));
        assert!(!source.contains(&["render_scene", "_color_hdr"].concat()));
        assert!(!source.contains(&["IblSourceCubemap", "StagingStore"].concat()));
        assert!(!source.contains("Vec::with_capacity"));
    }
}
