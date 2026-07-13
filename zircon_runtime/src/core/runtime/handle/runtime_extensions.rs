use std::sync::Arc;

use crate::core::{CoreError, RuntimeModuleLifecycleObserver};

use super::CoreHandle;

impl CoreHandle {
    pub fn install_runtime_module_lifecycle_observer(
        &self,
        observer: Arc<dyn RuntimeModuleLifecycleObserver>,
    ) {
        *self.lock_runtime_module_lifecycle_observer() = Some(observer);
    }

    pub fn clear_runtime_module_lifecycle_observer(
        &self,
    ) -> Option<Arc<dyn RuntimeModuleLifecycleObserver>> {
        self.lock_runtime_module_lifecycle_observer().take()
    }

    fn runtime_module_lifecycle_observer(&self) -> Option<Arc<dyn RuntimeModuleLifecycleObserver>> {
        self.lock_runtime_module_lifecycle_observer().clone()
    }

    pub(crate) fn notify_runtime_module_activated(&self, runtime_module_name: &str) {
        if let Some(observer) = self.runtime_module_lifecycle_observer() {
            observer.runtime_module_activated(runtime_module_name);
        }
    }

    pub(crate) fn notify_runtime_module_deactivating(
        &self,
        runtime_module_name: &str,
    ) -> Result<(), CoreError> {
        if let Some(observer) = self.runtime_module_lifecycle_observer() {
            observer
                .runtime_module_deactivating(runtime_module_name)
                .map_err(|blocked| {
                    CoreError::RuntimeModuleLifecycleBlocked(blocked.diagnostic().to_string())
                })?;
        }
        Ok(())
    }
}
