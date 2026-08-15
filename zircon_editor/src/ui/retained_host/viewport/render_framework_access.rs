use std::sync::Arc;

use zircon_runtime::core::manager::{
    render_framework_handle, resolve_manager_service, ManagerServiceHandle,
};
use zircon_runtime::core::{CoreError, CoreHandle, CoreWeak};

use crate::scene::viewport::RenderFramework;

/// Viewport-only access to the runtime render-framework service.
///
/// The retained viewport can resolve its own service but cannot use a generic runtime locator.
#[derive(Clone)]
pub(in crate::ui::retained_host) struct ViewportRenderFrameworkAccess {
    core: CoreWeak,
}

impl ViewportRenderFrameworkAccess {
    pub(in crate::ui::retained_host) fn new(core: &CoreHandle) -> Self {
        Self {
            core: core.downgrade(),
        }
    }

    pub(in crate::ui::retained_host) fn render_framework_handle(
        &self,
    ) -> Result<ManagerServiceHandle<dyn RenderFramework>, CoreError> {
        let core = self.core()?;
        render_framework_handle(&core)
    }

    pub(super) fn resolve_render_framework(
        &self,
        handle: ManagerServiceHandle<dyn RenderFramework>,
    ) -> Result<Arc<dyn RenderFramework>, CoreError> {
        let core = self.core()?;
        resolve_manager_service(&core, handle)
    }

    fn core(&self) -> Result<CoreHandle, CoreError> {
        self.core
            .upgrade()
            .ok_or_else(|| CoreError::ServiceUnavailable("CoreRuntime".to_owned()))
    }
}
