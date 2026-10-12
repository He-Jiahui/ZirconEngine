use std::any::Any;
use std::sync::Weak;

use super::handle::{CoreHandle, ServiceCallGuard, ServiceHandle};
use super::state::CoreRuntimeInner;
use crate::core::CoreError;

/// 可保存在模块、服务和插件回调中的非拥有式 Runtime 入口。
///
/// 使用前需升级；Runtime 已释放时解析返回 RuntimeUnavailable，避免注册表引用环。
#[derive(Clone, Debug)]
pub struct CoreWeak {
    pub(crate) inner: Weak<CoreRuntimeInner>,
}

impl CoreWeak {
    pub fn upgrade(&self) -> Option<CoreHandle> {
        let Some(inner) = self.inner.upgrade() else {
            return None;
        };
        Some(CoreHandle { inner })
    }

    pub fn resolve_driver<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceCallGuard<T>, CoreError> {
        self.upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?
            .resolve_driver(name)
    }

    pub fn resolve_manager<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceCallGuard<T>, CoreError> {
        self.upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?
            .resolve_manager(name)
    }

    pub fn resolve_plugin<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceCallGuard<T>, CoreError> {
        self.upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?
            .resolve_plugin(name)
    }
    /// Capture a durable reference; call `enter()` before each service invocation.
    pub fn resolve_driver_handle<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?
            .resolve_driver_handle(name)
    }

    /// Capture a durable reference; call `enter()` before each service invocation.
    pub fn resolve_manager_handle<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?
            .resolve_manager_handle(name)
    }

    /// Capture a durable reference; call `enter()` before each service invocation.
    pub fn resolve_plugin_handle<T: Any + Send + Sync>(
        &self,
        name: &str,
    ) -> Result<ServiceHandle<T>, CoreError> {
        self.upgrade()
            .ok_or(CoreError::RuntimeUnavailable)?
            .resolve_plugin_handle(name)
    }
}
