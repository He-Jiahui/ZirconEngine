use std::any::Any;
use std::sync::Arc;

use crate::core::manager::RegisteredManagerService;
use crate::core::{CoreError, CoreResult, LifecycleState, ServiceKind};

use super::super::{CoreHandle, RegisteredServiceIdentity};

/// Borrowed access to this stopping module's initialized service instances.
///
/// Core issues this capability only after closing admission and draining calls and
/// initialization. It does not initialize lazy services or expose an owning Core,
/// service Arc, or cloneable privileged handle. Service callbacks run outside the
/// registry locks and cannot return a borrowed service through this API.
///
/// 清理能力捕获本模块的注册身份；每次借用仍重验代次、关闭准入和排空状态。
/// `core` 与模块名仅在本上下文的借用期内有效，不把已停止服务重新开放给普通解析。
pub struct ModuleCleanupContext<'a> {
    core: &'a CoreHandle,
    module_name: &'a str,
    identities: Vec<RegisteredServiceIdentity>,
}

impl<'a> ModuleCleanupContext<'a> {
    /// 仅为已进入 Stopping 且服务准入关闭、调用和初始化均排空的模块建立身份快照。
    /// 缺少模块或任一服务不满足条件即返回错误；不创建任何服务实例。
    pub(in crate::core::runtime) fn new(
        core: &'a CoreHandle,
        module_name: &'a str,
    ) -> CoreResult<Self> {
        {
            let modules = core.lock_modules();
            let module = modules
                .get(module_name)
                .ok_or_else(|| CoreError::MissingModule(module_name.to_owned()))?;
            if module.lifecycle != LifecycleState::Stopping {
                return Err(CoreError::ServiceUnavailable(module_name.to_owned()));
            }
        }
        let services = core.lock_services();
        let mut identities = Vec::new();
        for (name, entry) in services.iter() {
            if name.module_name() != module_name {
                continue;
            }
            if entry.admission_open
                || entry.in_flight_calls != 0
                || entry.initialization_owner.is_some()
                || !matches!(
                    entry.lifecycle,
                    LifecycleState::Stopping | LifecycleState::Unloaded
                )
            {
                return Err(CoreError::ServiceUnavailable(name.to_string()));
            }
            identities.push(RegisteredServiceIdentity::new(
                entry.index,
                entry.generation,
                name.clone(),
            ));
        }
        drop(services);
        Ok(Self {
            core,
            module_name,
            identities,
        })
    }

    /// 返回本次停止清理所属模块的借用名称。
    pub fn module_name(&self) -> &str {
        self.module_name
    }

    /// Returns false when this registered owned driver was never initialized.
    /// 仅借用 `name` 指定的本模块驱动；`operation` 在注册表锁外执行。
    /// 成功执行返回 `true`，当前槽位没有实例返回 `false`；后者不会执行回调。
    /// 名称不属于本能力、种类不符、身份过期或状态不允许时返回错误；
    /// 现有实例若为声明视图或类型不符亦返回错误，回调错误原样传播。
    /// 借用只持续到回调结束；本方法不向调用方交出实例的 Arc。
    pub fn with_existing_driver<T: Any + Send + Sync>(
        &self,
        name: &str,
        operation: impl FnOnce(&T) -> CoreResult<()>,
    ) -> CoreResult<bool> {
        self.with_existing(name, ServiceKind::Driver, operation)
    }

    /// Returns false when this registered owned manager was never initialized.
    /// 对本模块 `name` 指定的管理器执行借用回调；参数、布尔结果与错误边界同驱动入口。
    /// `T` 必须是注册表实际存储的具体类型；trait 管理器包装请用专门入口。
    /// 参见 [`Self::with_existing_driver`] 和 [`Self::with_existing_registered_manager`]。
    pub fn with_existing_manager<T: Any + Send + Sync>(
        &self,
        name: &str,
        operation: impl FnOnce(&T) -> CoreResult<()>,
    ) -> CoreResult<bool> {
        self.with_existing(name, ServiceKind::Manager, operation)
    }

    /// Borrows the existing manager wrapper's service without exposing its Arc.
    /// 将现有管理器包装中的 `T` 借给 `operation`，支持不定大小的 trait 对象。
    /// 仅该包装实际存在时才调用回调；返回值和失败条件沿用现有管理器的清理准入，
    /// 且借用不会转交包装内的共享所有者；参见 [`Self::with_existing_manager`]。
    pub fn with_existing_registered_manager<T: ?Sized + Send + Sync + 'static>(
        &self,
        name: &str,
        operation: impl FnOnce(&T) -> CoreResult<()>,
    ) -> CoreResult<bool> {
        self.with_existing::<RegisteredManagerService<T>>(
            name,
            ServiceKind::Manager,
            |registered| operation(registered.borrowed()),
        )
    }

    // 身份快照不是永久许可：取实例前重新检查注册身份和关闭状态，解锁后才下转并回调。
    fn with_existing<T: Any + Send + Sync>(
        &self,
        name: &str,
        kind: ServiceKind,
        operation: impl FnOnce(&T) -> CoreResult<()>,
    ) -> CoreResult<bool> {
        let identity = self
            .identities
            .iter()
            .find(|identity| identity.service().as_str() == name)
            .ok_or_else(|| CoreError::ServiceUnavailable(name.to_owned()))?;
        if identity.service().service_kind() != kind {
            return Err(CoreError::ServiceKindMismatch {
                name: name.to_owned(),
                expected: kind,
                actual: identity.service().service_kind(),
            });
        }
        let instance = {
            let services = self.core.lock_services();
            let entry = services
                .get(identity.service())
                .ok_or_else(|| CoreError::MissingService(name.to_owned()))?;
            if entry.index != identity.index() || entry.generation != identity.generation() {
                return Err(CoreError::StaleServiceHandle {
                    name: name.to_owned(),
                    expected_index: identity.index(),
                    expected_generation: identity.generation(),
                    actual_index: entry.index,
                    actual_generation: entry.generation,
                });
            }
            if entry.admission_open
                || entry.in_flight_calls != 0
                || entry.initialization_owner.is_some()
                || !matches!(
                    entry.lifecycle,
                    LifecycleState::Stopping | LifecycleState::Unloaded
                )
            {
                return Err(CoreError::ServiceUnavailable(name.to_owned()));
            }
            entry.instance.clone()
        };
        let Some(instance) = instance else {
            return Ok(false);
        };
        // Cleanup is reserved for physically owned registrations. A declared
        // view does not grant access to its source after admission closes.
        if CoreHandle::is_service_alias(&instance) {
            return Err(CoreError::ServiceUnavailable(name.to_owned()));
        }
        let service = Arc::downcast::<T>(instance)
            .map_err(|_| CoreError::ServiceDowncast(name.to_owned()))?;
        operation(service.as_ref())?;
        Ok(true)
    }
}
