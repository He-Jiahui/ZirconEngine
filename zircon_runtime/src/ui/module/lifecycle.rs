use crate::core::{CoreResult, ModuleCleanupContext, ModuleLifecycle};

use super::{UiRuntimeDriver, UI_RUNTIME_DRIVER_NAME};

/// UI 停止清理钩子：借用当前已初始化驱动并关闭其新项目准入。
/// 驱动尚未初始化时跳过；清理能力的访问错误向 Core 传播，不新建服务。
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
