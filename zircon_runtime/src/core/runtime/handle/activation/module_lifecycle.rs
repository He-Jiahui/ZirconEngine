use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::{CoreError, LifecycleState, ModuleContext, ModuleLifecycle};

use super::super::super::descriptors::RegistryName;
use super::super::CoreHandle;

impl CoreHandle {
    pub(super) fn build_module(&self, module_name: &str) -> Result<(), CoreError> {
        let (lifecycle, context) = self.module_lifecycle_context(module_name)?;
        lifecycle.build(&context)
    }

    pub(super) fn wait_until_module_ready(
        &self,
        module_name: &str,
        ready_timeout: Duration,
    ) -> Result<(), CoreError> {
        let (lifecycle, context) = self.module_lifecycle_context(module_name)?;
        if lifecycle.ready(&context)? {
            return Ok(());
        }
        if ready_timeout.is_zero() {
            return Err(module_ready_timeout(module_name, ready_timeout));
        }

        let Some(deadline) = Instant::now().checked_add(ready_timeout) else {
            return Err(module_ready_timeout(module_name, ready_timeout));
        };
        loop {
            std::thread::yield_now();
            if lifecycle.ready(&context)? {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(module_ready_timeout(module_name, ready_timeout));
            }
        }
    }

    pub(super) fn finish_module(&self, module_name: &str) -> Result<(), CoreError> {
        let (lifecycle, context) = self.module_lifecycle_context(module_name)?;
        lifecycle.finish(&context)
    }

    pub(super) fn cleanup_module(&self, module_name: &str) -> Result<(), CoreError> {
        let (lifecycle, context) = self.module_lifecycle_context(module_name)?;
        lifecycle.cleanup(&context)
    }

    pub(super) fn reset_started_services(&self, startup_services: &[RegistryName]) {
        if startup_services.is_empty() {
            return;
        }
        let mut services = self.lock_services();
        for service_name in startup_services {
            if let Some(entry) = services.get_mut(service_name) {
                if entry.lifecycle == LifecycleState::Running
                    || entry.lifecycle == LifecycleState::Initializing
                {
                    entry.instance = None;
                    entry.lifecycle = LifecycleState::Registered;
                }
            }
        }
    }

    fn module_lifecycle_context(
        &self,
        module_name: &str,
    ) -> Result<(Arc<dyn ModuleLifecycle>, ModuleContext), CoreError> {
        let lifecycle = {
            let modules = self.lock_modules();
            let Some(entry) = modules.get(module_name) else {
                return Err(CoreError::MissingModule(module_name.to_owned()));
            };
            Arc::clone(&entry.descriptor.lifecycle)
        };
        Ok((
            lifecycle,
            ModuleContext {
                module_name: module_name.to_owned(),
                core: self.downgrade(),
            },
        ))
    }
}

fn module_ready_timeout(module_name: &str, ready_timeout: Duration) -> CoreError {
    CoreError::ModuleReadyTimeout {
        module: module_name.to_owned(),
        budget: ready_timeout,
    }
}
