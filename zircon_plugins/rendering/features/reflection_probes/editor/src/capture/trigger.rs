use std::path::{Path, PathBuf};

use zircon_plugin_rendering_reflection_probes_runtime::{
    capture_and_persist_reflection_probe, register_captured_reflection_probe,
    CapturedReflectionProbeAsset, CapturedReflectionProbeConsumeError,
    CapturedReflectionProbePlacement, ReflectionProbeCaptureError, ReflectionProbeCaptureReport,
    ReflectionProbeCaptureRequest, ReflectionProbeCaptureRequestError,
};
use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::RenderSceneSnapshot;
use zircon_runtime::graphics::SceneRenderer;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectionProbeCaptureEditorTrigger {
    cache_root: PathBuf,
}

impl ReflectionProbeCaptureEditorTrigger {
    pub fn new(cache_root: impl Into<PathBuf>) -> Self {
        Self {
            cache_root: cache_root.into(),
        }
    }

    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    pub fn execute(
        &self,
        renderer: &mut SceneRenderer,
        scene: &RenderSceneSnapshot,
        command: &ReflectionProbeCaptureEditorCommand,
    ) -> Result<ReflectionProbeCaptureReport, ReflectionProbeCaptureError> {
        let request = command.request()?;
        capture_and_persist_reflection_probe(renderer, scene, &self.cache_root, &request)
    }

    pub fn execute_and_register(
        &self,
        renderer: &mut SceneRenderer,
        asset_manager: &ProjectAssetManager,
        scene: &RenderSceneSnapshot,
        command: &ReflectionProbeCaptureEditorCommand,
    ) -> Result<ReflectionProbeCaptureEditorResult, ReflectionProbeCaptureEditorExecutionError>
    {
        let request = command.request()?;
        let placement = command
            .placement()?
            .ok_or(ReflectionProbeCaptureEditorExecutionError::MissingPlacement)?;
        let capture =
            capture_and_persist_reflection_probe(renderer, scene, &self.cache_root, &request)?;
        let asset =
            register_captured_reflection_probe(asset_manager, &request, &capture, &placement)?;
        Ok(ReflectionProbeCaptureEditorResult { capture, asset })
    }
}

#[derive(Debug)]
pub struct ReflectionProbeCaptureEditorResult {
    pub capture: ReflectionProbeCaptureReport,
    pub asset: CapturedReflectionProbeAsset,
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
    Consume(#[from] CapturedReflectionProbeConsumeError),
    #[error("reflection-probe capture editor command has no runtime placement")]
    MissingPlacement,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(command.request_json().contains("\"schema_version\": 1"));
        assert!(command.placement().unwrap().is_none());
    }

    #[test]
    fn editor_command_keeps_capture_and_runtime_placement_serialized() {
        let request = ReflectionProbeCaptureRequest::new(
            "atrium",
            "lib://probes/atrium.zcube",
            [3.0, 1.5, -2.0],
            9,
        );
        let placement = CapturedReflectionProbePlacement::box_probe(
            9,
            "lib://probes/atrium.pmrem",
            [8.0, 4.0, 6.0],
            1.0,
        );

        let command =
            ReflectionProbeCaptureEditorCommand::from_request_and_placement(&request, &placement)
                .unwrap();

        assert_eq!(command.request().unwrap(), request);
        assert_eq!(command.placement().unwrap(), Some(placement));
        assert!(command.placement_json().unwrap().contains("pmrem_uri"));
    }
}
