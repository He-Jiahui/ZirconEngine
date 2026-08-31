use zircon_runtime_host::viewport_surface::{
    ViewportSurfaceBindingOperation, ViewportSurfaceOperationInFlight,
    ViewportSurfaceReleaseOperation,
};
use zircon_runtime_interface::ZrRuntimeViewportHandle;

use super::super::RuntimeLibraryError;
use super::RuntimeSession;

impl RuntimeSession {
    pub(super) fn begin_viewport_surface_binding(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<ViewportSurfaceBindingOperation<'_>, RuntimeLibraryError> {
        self.viewport_surface_bindings
            .begin_binding(viewport)
            .map_err(viewport_surface_operation_in_flight_error)
    }

    pub(super) fn finish_viewport_surface_binding(
        &self,
        operation: ViewportSurfaceBindingOperation<'_>,
        succeeded: bool,
    ) {
        operation.finish(succeeded);
    }

    pub(super) fn begin_viewport_surface_release(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<Option<ViewportSurfaceReleaseOperation<'_>>, RuntimeLibraryError> {
        self.viewport_surface_bindings
            .begin_release(viewport)
            .map_err(viewport_surface_operation_in_flight_error)
    }

    pub(super) fn finish_viewport_surface_release(
        &self,
        operation: ViewportSurfaceReleaseOperation<'_>,
        succeeded: bool,
    ) {
        operation.finish(succeeded);
    }

    pub(super) fn bound_viewport_surfaces(&self) -> Vec<ZrRuntimeViewportHandle> {
        self.viewport_surface_bindings.bound_viewports()
    }
}

fn viewport_surface_operation_in_flight_error(
    operation: ViewportSurfaceOperationInFlight,
) -> RuntimeLibraryError {
    RuntimeLibraryError::new(format!(
        "viewport surface binding transition is already in flight for viewport {}",
        operation.viewport().raw()
    ))
}
