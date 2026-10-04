use std::fmt;
use std::sync::Arc;

use arc_swap::ArcSwapOption;

use crate::core::framework::bridge::{BridgeError, PluginInterface};

use super::{FrozenBridgeTable, WeakBridge};

/// 消费插件在注册阶段拿到此句柄；合并后的注册表才把它接到最终冻结表。
/// 克隆共享绑定槽，owner 撤销会解绑新调用，已进入回调的调用可依其 Arc 完成。
/// Cloneable consumer-side bridge handle bound after the runtime catalog has
/// merged and finalized every plugin contribution.
pub struct BridgeImport<T: ?Sized> {
    binding: Arc<ArcSwapOption<WeakBridge<T>>>,
}

impl<T: ?Sized> Clone for BridgeImport<T> {
    fn clone(&self) -> Self {
        Self {
            binding: Arc::clone(&self.binding),
        }
    }
}

impl<T: ?Sized> fmt::Debug for BridgeImport<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bound = self.binding.load().is_some();
        formatter
            .debug_struct("BridgeImport")
            .field("bound", &bound)
            .finish()
    }
}

impl<T> BridgeImport<T>
where
    T: PluginInterface + ?Sized,
{
    /// 成对交给消费方和注册表：前者发起调用，后者以静态接口 ID 在 finalize/撤销时更新绑定。
    pub(crate) fn new() -> (Self, InterfaceImport) {
        let binding = Arc::new(ArcSwapOption::empty());
        let imported = Self {
            binding: Arc::clone(&binding),
        };
        let erased = InterfaceImport {
            interface_id: T::INTERFACE_ID,
            update: Arc::new(move |table| {
                binding.store(
                    table
                        .map(FrozenBridgeTable::resolve_weak::<T>)
                        .map(Arc::new),
                );
            }),
        };
        (imported, erased)
    }

    /// 未完成绑定时返回 Absent；绑定后仍须检查 provider 的当前代际，不能把一次 is_enabled 当调用许可。
    pub fn call<R>(&self, callback: impl FnOnce(&T) -> R) -> Result<R, BridgeError> {
        let binding = self.binding.load();
        let bridge = binding.as_ref().ok_or(BridgeError::Absent)?;
        bridge.call(callback)
    }

    pub fn is_enabled(&self) -> bool {
        self.binding
            .load()
            .as_deref()
            .is_some_and(WeakBridge::is_enabled)
    }
}

/// 注册表保存的擦除端，只负责把同一消费句柄切换到最终表或解除绑定。
#[derive(Clone)]
pub(crate) struct InterfaceImport {
    interface_id: &'static str,
    update: Arc<dyn Fn(Option<&FrozenBridgeTable>) + Send + Sync>,
}

impl InterfaceImport {
    pub(crate) fn interface_id(&self) -> &str {
        self.interface_id
    }

    pub(crate) fn bind(&self, table: &FrozenBridgeTable) {
        (self.update)(Some(table));
    }

    pub(crate) fn unbind(&self) {
        (self.update)(None);
    }
}

impl fmt::Debug for InterfaceImport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InterfaceImport")
            .field("interface_id", &self.interface_id)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "tests/import.rs"]
mod tests;
