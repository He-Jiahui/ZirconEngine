use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::{CoreError, LifecycleState, ModuleContext, ModuleLifecycle};

use super::super::super::descriptors::RegistryName;
use super::super::CoreHandle;

const MODULE_READY_POLL_INTERVAL: Duration = Duration::from_millis(1);

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
            std::thread::sleep(
                MODULE_READY_POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())),
            );
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
        let mut changed = false;
        for service_name in startup_services {
            if let Some(entry) = services.get_mut(service_name) {
                if entry.lifecycle == LifecycleState::Running
                    || entry.lifecycle == LifecycleState::Initializing
                {
                    entry.reset_after_failed_activation();
                    changed = true;
                }
            }
        }
        drop(services);
        if changed {
            self.notify_service_resolution_changed();
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

#[cfg(test)]
mod performance_tests {
    #[test]
    fn module_ready_polling_uses_a_bounded_sleep() {
        let source = include_str!("module_lifecycle.rs");
        let end = source
            .find("mod performance_tests {")
            .expect("performance test module");
        let implementation = &source[..end];

        assert!(implementation.contains("MODULE_READY_POLL_INTERVAL"));
        assert!(implementation.contains("std::thread::sleep"));
        assert!(!implementation.contains("std::thread::yield_now()"));
    }
}
