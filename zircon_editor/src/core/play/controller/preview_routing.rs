use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportCameraV1, ZrRuntimeViewportSizeV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
    ZR_RUNTIME_VIEWPORT_CAMERA_REQUEST_LIMIT_V1,
};

use crate::core::gateway::EditorRuntimeGatewayHandle;

use super::PlaySessionController;
use crate::core::play::{
    PlayKind, PlayMode, PlayPreviewCaptureError, PlayPreviewFrame, PlayPreviewInputError,
    PlaySimulateCameraError, WorldDomain,
};

impl PlaySessionController {
    pub fn capture_preview_frame(
        &self,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<Option<PlayPreviewFrame>, PlayPreviewCaptureError> {
        zircon_runtime::profile_scope!("editor", "play", "capture_preview_frame");
        let _transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mode = self.mode();
        if !mode.has_active_runtime() {
            return Ok(None);
        }
        let Some(WorldDomain::Play(instance)) = self.play_domain.attached_domain() else {
            return Err(PlayPreviewCaptureError::GatewayUnavailable { mode });
        };
        let Some(play_gateway) = self.play_domain.gateway(instance) else {
            return Err(PlayPreviewCaptureError::GatewayUnavailable { mode });
        };
        let gateway_identity = play_gateway.identity();
        let frame = play_gateway
            .capture_frame_at_identity(
                &gateway_identity,
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                size,
            )
            .map_err(PlayPreviewCaptureError::Capture)?;
        PlayPreviewFrame::copy_and_release(instance, gateway_identity, frame).map(Some)
    }

    /// Routes input only to an attached Play runtime. Simulate intentionally keeps editor input.
    pub fn route_preview_input(
        &self,
        event: ZrRuntimeEventV1,
    ) -> Result<bool, PlayPreviewInputError> {
        let _transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mode = self.mode_snapshot();
        if !matches!(
            &mode,
            PlayMode::Playing {
                kind: PlayKind::Play
            }
        ) {
            return Ok(false);
        }
        let mode = mode.kind();
        let Some(WorldDomain::Play(instance)) = self.play_domain.attached_domain() else {
            return Err(PlayPreviewInputError::GatewayUnavailable { mode });
        };
        let Some(play_gateway) = self.play_domain.gateway(instance) else {
            return Err(PlayPreviewInputError::GatewayUnavailable { mode });
        };
        let gateway_identity = play_gateway.identity();
        play_gateway
            .handle_event_at_identity(&gateway_identity, event)
            .map_err(PlayPreviewInputError::Dispatch)?;
        zircon_runtime::profile_counter!("editor", "play.preview.input_routed_count", 1);
        Ok(true)
    }

    pub fn preview_input_active(&self) -> bool {
        matches!(
            self.mode_snapshot(),
            PlayMode::Playing {
                kind: PlayKind::Play
            }
        )
    }

    /// Synchronizes the editor view into SIE without mutating the duplicated gameplay world.
    pub fn route_simulate_camera(
        &self,
        camera: ZrRuntimeViewportCameraV1,
    ) -> Result<bool, PlaySimulateCameraError> {
        let _transition = self
            .transition_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mode = self.mode_snapshot();
        if !matches!(
            &mode,
            PlayMode::Playing {
                kind: PlayKind::Simulate
            }
        ) {
            return Ok(false);
        }
        let mode = mode.kind();
        let Some(WorldDomain::Play(instance)) = self.play_domain.attached_domain() else {
            return Err(PlaySimulateCameraError::GatewayUnavailable { mode });
        };
        let Some(play_gateway) = self.play_domain.gateway(instance) else {
            return Err(PlaySimulateCameraError::GatewayUnavailable { mode });
        };
        let gateway_identity = play_gateway.identity();
        let payload = serde_json::to_vec(&camera).map_err(PlaySimulateCameraError::Encode)?;
        let limit = ZR_RUNTIME_VIEWPORT_CAMERA_REQUEST_LIMIT_V1.max_encoded_bytes;
        if payload.len() > limit {
            return Err(PlaySimulateCameraError::PayloadTooLarge {
                len: payload.len(),
                limit,
            });
        }
        play_gateway
            .handle_event_at_identity(
                &gateway_identity,
                ZrRuntimeEventV1::viewport_camera(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                    ZrByteSlice {
                        data: payload.as_ptr(),
                        len: payload.len(),
                    },
                ),
            )
            .map_err(PlaySimulateCameraError::Dispatch)?;
        zircon_runtime::profile_counter!("editor", "play.simulate.camera_routed_count", 1);
        Ok(true)
    }

    pub(crate) fn play_gateway_handle(&self) -> EditorRuntimeGatewayHandle {
        self.play_domain.gateway_handle()
    }
}
