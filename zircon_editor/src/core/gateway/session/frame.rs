use std::sync::Arc;

use zircon_runtime_host::foreign_output::RuntimeForeignOutputKind;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2,
};

use super::super::contract::EditorRuntimeFramePixels;
use super::super::{EditorRuntimeFrame, EditorRuntimeFrameDemand, GatewayError};
use super::gateway::SessionGateway;
use super::output::capture_owned_output;
use super::protocol::{
    ensure_frame_rgba_shape, ensure_output_abi, ensure_status, frame_demand_from_runtime,
};

struct SessionRuntimeFramePixels {
    _runtime_owner: Arc<dyn Send + Sync>,
    output: Option<super::output::GatewayOwnedOutput>,
}

impl EditorRuntimeFramePixels for SessionRuntimeFramePixels {
    fn rgba(&self) -> &[u8] {
        self.output
            .as_ref()
            .expect("session frame output remains present until release")
            .bytes("capture runtime frame")
            .expect("validated session frame storage remains owned until the frame is released")
    }

    fn release(mut self: Box<Self>) -> Result<(), GatewayError> {
        let output = self
            .output
            .take()
            .expect("session frame output can only be released once");
        output.release()
    }
}

impl Drop for SessionRuntimeFramePixels {
    fn drop(&mut self) {
        // Release while `_runtime_owner` still keeps the runtime session and DLL alive.
        drop(self.output.take());
    }
}

impl SessionGateway {
    pub(super) fn tick_frame(&self) -> Result<EditorRuntimeFrameDemand, GatewayError> {
        self.ensure_session_available("tick runtime frame")?;
        let tick = Self::required(self.api.tick_frame, "runtime.frame.tick")?;
        let mut demand = ZrRuntimeFrameDemandV1::idle();
        ensure_status(
            unsafe { tick(self.session, &mut demand) },
            "tick runtime frame",
        )?;
        match frame_demand_from_runtime(demand) {
            Ok(demand) => Ok(demand),
            Err(error) => self.reject_protocol(RuntimeForeignOutputKind::SessionProtocol, error),
        }
    }

    pub(super) fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError> {
        self.ensure_session_available("send runtime event")?;
        let handle_event = Self::required(self.api.handle_event, "runtime.event.handle")?;
        ensure_status(
            unsafe { handle_event(self.session, event) },
            "send runtime event",
        )
    }

    pub(super) fn capture_frame(
        &self,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Result<EditorRuntimeFrame, GatewayError> {
        self.ensure_output_available(RuntimeForeignOutputKind::SessionProtocol)?;
        let capture = Self::required(self.api.capture_frame, "runtime.frame.capture")?;
        let mut frame = ZrRuntimeFrameV2::empty(ZIRCON_RUNTIME_ABI_VERSION_V2);
        let status = unsafe {
            capture(
                self.session,
                ZrRuntimeFrameRequestV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport, size),
                &mut frame,
            )
        };
        let output = capture_owned_output(
            self.foreign_output.clone(),
            status,
            frame.rgba,
            self.output_releaser,
            "capture runtime frame",
        )?;
        let validation = ensure_output_abi(
            frame.abi_version,
            ZIRCON_RUNTIME_ABI_VERSION_V2,
            "runtime frame",
        )
        .and_then(|()| {
            output
                .bytes("capture runtime frame")
                .and_then(|rgba| ensure_frame_rgba_shape(frame.width, frame.height, rgba))
        });
        if let Err(error) = validation {
            return output.release_after_protocol_error(error);
        }
        Ok(EditorRuntimeFrame::from_pixels(
            frame.abi_version,
            frame.width,
            frame.height,
            frame.generation,
            Box::new(SessionRuntimeFramePixels {
                _runtime_owner: self._runtime_owner.clone(),
                output: Some(output),
            }),
        ))
    }
}
