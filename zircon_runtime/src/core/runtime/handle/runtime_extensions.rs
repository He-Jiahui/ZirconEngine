use std::sync::Arc;
use std::time::Instant;

use crate::core::{CoreError, RuntimeModuleLifecycleObserver};

use super::CoreHandle;

impl CoreHandle {
    /// 替换当前观察器；激活进入 Running 后同步通知，停用提交 Stopping 前可由观察器阻断。
    pub fn install_runtime_module_lifecycle_observer(
        &self,
        observer: Arc<dyn RuntimeModuleLifecycleObserver>,
    ) {
        *self.lock_runtime_module_lifecycle_observer() = Some(observer);
    }

    /// 移除观察器并返回原实例；通知路径已取得的克隆引用仍会完成当前回调。
    pub fn clear_runtime_module_lifecycle_observer(
        &self,
    ) -> Option<Arc<dyn RuntimeModuleLifecycleObserver>> {
        self.lock_runtime_module_lifecycle_observer().take()
    }

    // 通知方先克隆 Arc 快照并释放互斥锁，再执行回调，允许回调回访或替换观察器。
    fn runtime_module_lifecycle_observer(&self) -> Option<Arc<dyn RuntimeModuleLifecycleObserver>> {
        self.lock_runtime_module_lifecycle_observer().clone()
    }

    // 激活事务先记录 Running 再同步通知；回调 panic 由外层事务捕获并触发回滚。
    pub(crate) fn notify_runtime_module_activated(&self, runtime_module_name: &str) {
        if let Some(observer) = self.runtime_module_lifecycle_observer() {
            observer.runtime_module_activated(runtime_module_name);
        }
    }

    pub(crate) fn notify_runtime_module_deactivating(
        &self,
        runtime_module_name: &str,
        deadline: Option<Instant>,
    ) -> Result<(), CoreError> {
        if let Some(observer) = self.runtime_module_lifecycle_observer() {
            let result = match deadline {
                Some(deadline) => {
                    observer.runtime_module_deactivating_until(runtime_module_name, deadline)
                }
                None => observer.runtime_module_deactivating(runtime_module_name),
            };
            result.map_err(|blocked| {
                CoreError::RuntimeModuleLifecycleBlocked(blocked.diagnostic().to_string())
            })?;
        }
        Ok(())
    }
}
