use zircon_runtime_interface::{
    ZrRuntimeViewportCameraV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1,
    ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_PERSPECTIVE_V1,
};

use crate::core::editing::authoring_world::AuthoringWorldAccessError;
use crate::core::play::{PlayInstanceId, PlayKind, PlayMode, PlaySimulateCameraError, WorldDomain};
use crate::scene::viewport::ProjectionMode;

use super::EditorHostEventController;

impl EditorHostEventController {
    pub(crate) fn simulate_preview_camera(
        &self,
    ) -> Result<Option<(PlayInstanceId, ZrRuntimeViewportCameraV1)>, AuthoringWorldAccessError>
    {
        if !matches!(
            self.play_sessions.mode_snapshot(),
            PlayMode::Playing {
                kind: PlayKind::Simulate
            }
        ) {
            return Ok(None);
        }
        let Some(WorldDomain::Play(instance)) = self.play_sessions.attached_world_domain() else {
            return Ok(None);
        };
        let shell = self.shell.lock();
        let Some(camera) = shell.state.viewport_camera_snapshot()? else {
            return Ok(None);
        };
        let projection_kind = match camera.projection_mode {
            ProjectionMode::Perspective => ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_PERSPECTIVE_V1,
            ProjectionMode::Orthographic => ZR_RUNTIME_VIEWPORT_CAMERA_PROJECTION_ORTHOGRAPHIC_V1,
        };
        Ok(Some((
            instance,
            ZrRuntimeViewportCameraV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                camera.transform,
                projection_kind,
                camera.fov_y_radians,
                camera.ortho_size,
                camera.z_near,
                camera.z_far,
            ),
        )))
    }

    pub(crate) fn route_simulate_preview_camera(
        &self,
        camera: ZrRuntimeViewportCameraV1,
    ) -> Result<bool, PlaySimulateCameraError> {
        self.play_sessions.route_simulate_camera(camera)
    }
}
