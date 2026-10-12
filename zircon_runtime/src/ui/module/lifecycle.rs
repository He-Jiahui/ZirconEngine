use crate::core::{CoreResult, ModuleCleanupContext, ModuleLifecycle};

use super::{UiRuntimeDriver, UI_RUNTIME_DRIVER_NAME};

#[derive(Debug, Default)]
pub(super) struct UiModuleLifecycle;

impl ModuleLifecycle for UiModuleLifecycle {
    fn cleanup(&self, context: &ModuleCleanupContext<'_>) -> CoreResult<()> {
        context.with_existing_driver::<UiRuntimeDriver>(UI_RUNTIME_DRIVER_NAME, |driver| {
            driver.close_admission();
            Ok(())
        })?;
        Ok(())
    }
}
