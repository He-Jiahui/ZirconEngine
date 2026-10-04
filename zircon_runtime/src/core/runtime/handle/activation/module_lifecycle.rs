use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::{CoreError, LifecycleState, ModuleContext, ModuleLifecycle};

use super::super::super::descriptors::RegistryName;
use super::super::CoreHandle;
use super::service_lifecycle::retire_service_objects;

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
        let Some(deadline) = Instant::now().checked_add(ready_timeout) else {
            return Err(module_ready_timeout(module_name, ready_timeout));
        };
        self.wait_until_module_ready_until(module_name, deadline, ready_timeout)
    }

    /// Poll one module against a caller-owned absolute deadline. Batch callers
    /// pass one deadline so earlier modules cannot multiply the ready budget.
    pub(super) fn wait_until_module_ready_until(
        &self,
        module_name: &str,
        deadline: Instant,
        ready_budget: Duration,
    ) -> Result<(), CoreError> {
        let (lifecycle, context) = self.module_lifecycle_context(module_name)?;
        if lifecycle.ready(&context)? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(module_ready_timeout(module_name, ready_budget));
        }
        loop {
            std::thread::sleep(
                MODULE_READY_POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())),
            );
            if lifecycle.ready(&context)? {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(module_ready_timeout(module_name, ready_budget));
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

    pub(super) fn cleanup_module_until(
        &self,
        module_name: &str,
        deadline: Instant,
    ) -> Result<(), CoreError> {
        let (lifecycle, context) = self.module_lifecycle_context(module_name)?;
        lifecycle.cleanup_until(&context, deadline)
    }

    pub(super) fn reset_started_services(
        &self,
        module_name: &str,
        startup_services: &[RegistryName],
    ) -> Option<CoreError> {
        if startup_services.is_empty() {
            return None;
        }
        // 失败回滚仅重置本次启动过且仍处于启动态的服务；先释放表锁再唤醒解析等待者。
        let mut services = self.lock_services();
        let mut changed = false;
        let mut retired = Vec::new();
        for service_name in startup_services {
            if let Some(entry) = services.get_mut(service_name) {
                if entry.lifecycle == LifecycleState::Running
                    || entry.lifecycle == LifecycleState::Initializing
                {
                    if let Some(instance) = entry.reset_after_failed_activation() {
                        retired.push((module_name.to_owned(), service_name.to_string(), instance));
                    }
                    changed = true;
                }
            }
        }
        drop(services);
        if changed {
            self.notify_service_resolution_changed();
        }
        retire_service_objects(retired, "activate")
    }

    fn module_lifecycle_context(
        &self,
        module_name: &str,
    ) -> Result<(Arc<dyn ModuleLifecycle>, ModuleContext), CoreError> {
        // 只在模块表锁内克隆生命周期对象；释放锁后才调用 trait，并把弱句柄交给回调上下文。
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
#[path = "tests/module_lifecycle_performance_tests.rs"]
mod performance_tests;
