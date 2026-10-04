use std::sync::Arc;

use zircon_runtime::core::{CoreError, CoreHandle, CoreWeak};

use super::super::viewport::render_framework_access::ViewportRenderFrameworkAccess;
use super::asset_runtime_access::RetainedHostAssetRuntimeAccess;
use crate::ui::host::module::EDITOR_MANAGER_NAME;
use crate::ui::host::EditorManager;

/// Composition-root runtime lifetime for the retained editor host.
///
/// Children receive only narrowly typed weak access so a pane or background job cannot become a
/// second runtime owner.
pub(super) struct RetainedHostRuntimeLease {
    core: CoreHandle,
}

impl RetainedHostRuntimeLease {
    pub(super) fn new(core: CoreHandle) -> Self {
        Self { core }
    }

    pub(super) fn startup_access(&self) -> RetainedHostStartupRuntimeAccess {
        RetainedHostStartupRuntimeAccess::new(&self.core)
    }

    pub(super) fn viewport_render_framework_access(&self) -> ViewportRenderFrameworkAccess {
        ViewportRenderFrameworkAccess::new(&self.core)
    }
}

/// Startup-only runtime services granted to retained-host assembly.
///
/// The composition root keeps the strong runtime lease. Startup resolves only the manager and
/// asset services required to assemble the host, so no pane or startup callback can acquire a
/// generic runtime handle.
#[derive(Clone)]
pub(super) struct RetainedHostStartupRuntimeAccess {
    core: CoreWeak,
}

impl RetainedHostStartupRuntimeAccess {
    fn new(core: &CoreHandle) -> Self {
        Self {
            core: core.downgrade(),
        }
    }

    pub(super) fn asset_runtime_access(&self) -> Result<RetainedHostAssetRuntimeAccess, CoreError> {
        RetainedHostAssetRuntimeAccess::new(&self.core()?)
    }

    pub(super) fn editor_manager(&self) -> Result<Arc<EditorManager>, CoreError> {
        self.core()?
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
    }

    fn core(&self) -> Result<CoreHandle, CoreError> {
        self.core
            .upgrade()
            .ok_or_else(|| CoreError::ServiceUnavailable("CoreRuntime".to_owned()))
    }
}

#[cfg(test)]
#[path = "tests/runtime_lease.rs"]
mod tests;
