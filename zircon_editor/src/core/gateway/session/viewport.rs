use std::sync::atomic::Ordering;

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
        let bind = Self::required(
            self.api.bind_viewport_surface,
            "runtime.viewport.surface.bind",
        )?;
        ensure_status(
            unsafe { bind(self.session, request) },
            "bind runtime viewport surface",
        )?;
        self.viewport_surface_bound.store(true, Ordering::Release);
        Ok(())
    }

    pub(super) fn unbind_viewport_surface(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<(), GatewayError> {
        let unbind = Self::required(
            self.api.unbind_viewport_surface,
            "runtime.viewport.surface.unbind",
        )?;
        ensure_status(
            unsafe { unbind(self.session, viewport) },
            "unbind runtime viewport surface",
        )?;
        self.viewport_surface_bound.store(false, Ordering::Release);
        Ok(())
    }

    pub(super) fn present_viewport(
        &self,
        request: ZrRuntimeFrameRequestV1,
    ) -> Result<(), GatewayError> {
        let present = Self::required(self.api.present_viewport, "runtime.viewport.present")?;
        ensure_status(
            unsafe { present(self.session, request) },
            "present runtime viewport",
        )
    }
}
