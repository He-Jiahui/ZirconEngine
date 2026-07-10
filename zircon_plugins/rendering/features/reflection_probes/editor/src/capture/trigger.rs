use std::path::{Path, PathBuf};

use zircon_plugin_rendering_reflection_probes_runtime::{
    capture_and_persist_reflection_probe, ReflectionProbeCaptureError,
    ReflectionProbeCaptureReport, ReflectionProbeCaptureRequest,
    ReflectionProbeCaptureRequestError,
};
use zircon_runtime::core::framework::render::RenderSceneSnapshot;
use zircon_runtime::graphics::SceneRenderer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectionProbeCaptureEditorCommand {
    request_json: String,
}

impl ReflectionProbeCaptureEditorCommand {
    pub fn from_request(
        request: &ReflectionProbeCaptureRequest,
    ) -> Result<Self, ReflectionProbeCaptureRequestError> {
        Ok(Self {
            request_json: request.encode_json()?,
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReflectionProbeCaptureEditorTrigger {
    library_root: PathBuf,
}

impl ReflectionProbeCaptureEditorTrigger {
    pub fn new(library_root: impl Into<PathBuf>) -> Self {
        Self {
            library_root: library_root.into(),
        }
    }

    pub fn library_root(&self) -> &Path {
        &self.library_root
    }

    pub fn execute(
        &self,
        renderer: &mut SceneRenderer,
        scene: &RenderSceneSnapshot,
        command: &ReflectionProbeCaptureEditorCommand,
    ) -> Result<ReflectionProbeCaptureReport, ReflectionProbeCaptureError> {
        let request = command.request()?;
        capture_and_persist_reflection_probe(renderer, scene, &self.library_root, &request)
    }
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
    }
}
