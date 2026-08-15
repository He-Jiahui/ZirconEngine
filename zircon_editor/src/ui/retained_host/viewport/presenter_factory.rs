use std::sync::Arc;

use zircon_runtime::rhi::{UiSurfaceDescriptor, UiSurfacePresenter};

use crate::ui::retained_host::{
    runtime_factory_error, HostPresenterResult, RuntimeUiSurfacePresenterFactory,
};

use super::retained_viewport_controller::RetainedViewportController;

struct RetainedViewportPresenterFactory {
    controller: RetainedViewportController,
}

impl RuntimeUiSurfacePresenterFactory for RetainedViewportPresenterFactory {
    fn poll_ready(&self) -> HostPresenterResult<bool> {
        let render_framework = {
            let mut shared = self.controller.lock_shared();
            shared
                .poll_or_start_render_framework()
                .map_err(runtime_factory_error)?
        };
        Ok(render_framework.is_some())
    }

    fn create(
        &self,
        descriptor: UiSurfaceDescriptor,
    ) -> HostPresenterResult<Box<dyn UiSurfacePresenter>> {
        let render_framework = {
            let mut shared = self.controller.lock_shared();
            shared.render_framework().map_err(runtime_factory_error)?
        };
        render_framework
            .create_ui_surface_presenter(descriptor)
            .map_err(runtime_factory_error)
    }
}

impl RetainedViewportController {
    pub(crate) fn runtime_presenter_factory(&self) -> Arc<dyn RuntimeUiSurfacePresenterFactory> {
        Arc::new(RetainedViewportPresenterFactory {
            controller: self.clone(),
        })
    }
}
