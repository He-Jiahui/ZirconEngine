use crate::core::{CoreError, CoreResult, ModuleContext, ModuleLifecycle};

use super::{UiRuntimeDriver, UI_RUNTIME_DRIVER_NAME};

#[derive(Debug, Default)]
pub(super) struct UiModuleLifecycle;

impl ModuleLifecycle for UiModuleLifecycle {
    fn cleanup(&self, context: &ModuleContext) -> CoreResult<()> {
        let core = context
            .core
            .upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?;
        core.resolve_driver::<UiRuntimeDriver>(UI_RUNTIME_DRIVER_NAME)?
            .close_admission();
        Ok(())
    }
}
