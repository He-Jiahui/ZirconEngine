use zircon_runtime_host::viewport_surface::ViewportSurfaceOperationInFlight;
use zircon_runtime_interface::{
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeFrameRequestV1, ZrRuntimeViewportHandle,
};

use super::super::GatewayError;
use super::gateway::SessionGateway;
use super::protocol::ensure_status;

impl SessionGateway {
    pub(super) fn bind_viewport_surface(
        &self,
        request: ZrRuntimeBindViewportSurfaceRequestV1,
    ) -> Result<(), GatewayError> {
        self.ensure_session_available("bind runtime viewport surface")?;
        let bind = Self::required(
            self.api.bind_viewport_surface,
            "runtime.viewport.surface.bind",
        )?;
        let operation = self
            .viewport_surface_bindings
            .begin_binding(request.viewport)
            .map_err(viewport_surface_transition_in_flight)?;
        let result = ensure_status(
            unsafe { bind(self.session, request) },
            "bind runtime viewport surface",
        );
        operation.finish(result.is_ok());
        result
    }

    pub(super) fn unbind_viewport_surface(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<(), GatewayError> {
        self.ensure_session_available("unbind runtime viewport surface")?;
        let Some(operation) = self
            .viewport_surface_bindings
            .begin_release(viewport)
            .map_err(viewport_surface_transition_in_flight)?
        else {
            return Ok(());
        };
        let unbind = Self::required(
            self.api.unbind_viewport_surface,
            "runtime.viewport.surface.unbind",
        )?;
        let result = ensure_status(
            unsafe { unbind(self.session, viewport) },
            "unbind runtime viewport surface",
        );
        operation.finish(result.is_ok());
        result
    }

    pub(super) fn present_viewport(
        &self,
        request: ZrRuntimeFrameRequestV1,
    ) -> Result<(), GatewayError> {
        self.ensure_session_available("present runtime viewport")?;
        let present = Self::required(self.api.present_viewport, "runtime.viewport.present")?;
        ensure_status(
            unsafe { present(self.session, request) },
            "present runtime viewport",
        )
    }
}

fn viewport_surface_transition_in_flight(error: ViewportSurfaceOperationInFlight) -> GatewayError {
    GatewayError::ViewportSurfaceTransitionInFlight {
        viewport: error.viewport().raw(),
    }
}
