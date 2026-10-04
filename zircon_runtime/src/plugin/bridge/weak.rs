use std::ops::Deref;
use std::sync::{Arc, Weak};

use arc_swap::ArcSwapOption;

use crate::core::framework::bridge::{BridgeError, InterfaceSlot, PluginInterface};

use super::table::FrozenBridgeTable;

/// 可跨 owner 启停持有的调用句柄。缓存只保存弱 provider，避免旧实现被缓存延长寿命。
pub struct WeakBridge<T: ?Sized> {
    table: FrozenBridgeTable,
    slot: Option<InterfaceSlot>,
    cached: Arc<ArcSwapOption<ProviderSnapshot<T>>>,
}

struct ProviderSnapshot<T: ?Sized> {
    generation: u32,
    provider: Weak<T>,
}

impl<T: ?Sized> Clone for WeakBridge<T> {
    fn clone(&self) -> Self {
        Self {
            table: self.table.clone(),
            slot: self.slot,
            cached: Arc::clone(&self.cached),
        }
    }
}

impl<T> WeakBridge<T>
where
    T: PluginInterface + ?Sized,
{
    pub(crate) fn new(table: FrozenBridgeTable, slot: Option<InterfaceSlot>) -> Self {
        Self {
            table,
            slot,
            cached: Arc::new(ArcSwapOption::empty()),
        }
    }

    /// 通过静态接口 ID 一次确定 slot；热更新维持 slot，调用时再查代际。
    pub fn owned(table: FrozenBridgeTable) -> Self {
        let slot = table.resolve_slot(T::INTERFACE_ID);
        Self::new(table, slot)
    }

    /// 每次调用先验证当前代际；匹配时复用弱缓存，否则从表重新获取 provider。
    /// 已取得 Arc 的在途回调允许跨禁用或替换完成，调用方不能以禁用撤销正在运行的回调。
    pub fn call<R>(&self, f: impl FnOnce(&T) -> R) -> Result<R, BridgeError> {
        let slot = self.slot.ok_or(BridgeError::Absent)?;
        match self.current_generation(slot) {
            Ok(generation) => {
                let cached = self.cached.load();
                if let Some(snapshot) = cached
                    .as_ref()
                    .filter(|snapshot| snapshot.generation == generation)
                {
                    if let Some(provider) = snapshot.provider.upgrade() {
                        self.table.record_enabled_call(slot);
                        return Ok(f(provider.as_ref()));
                    }
                }
                drop(cached);

                match self.refresh_provider(slot) {
                    Ok((_, provider)) => {
                        self.table.record_enabled_call(slot);
                        Ok(f(provider.as_ref()))
                    }
                    Err((_, error)) => {
                        self.table.record_not_enabled_call(slot);
                        Err(error)
                    }
                }
            }
            Err(error) => {
                self.table.record_not_enabled_call(slot);
                Err(error)
            }
        }
    }

    /// 系统体批量调用前解析一次；guard 持强 Arc，停用后仍可使用直到 guard 释放。
    pub fn pin(&self) -> Result<BridgeGuard<T>, BridgeError> {
        match self.provider_with_slot() {
            Ok((slot, target)) => {
                self.table.record_enabled_call(slot);
                Ok(BridgeGuard { target })
            }
            Err((Some(slot), error)) => {
                self.table.record_not_enabled_call(slot);
                Err(error)
            }
            Err((None, error)) => Err(error),
        }
    }

    pub fn is_enabled(&self) -> bool {
        let Some(slot) = self.slot else {
            return false;
        };
        let Ok(generation) = self.current_generation(slot) else {
            return false;
        };
        let cached = self.cached.load();
        if cached
            .as_ref()
            .filter(|snapshot| snapshot.generation == generation)
            .and_then(|snapshot| snapshot.provider.upgrade())
            .is_some()
        {
            return true;
        }
        drop(cached);
        self.refresh_provider(slot).is_ok()
    }

    fn provider_with_slot(
        &self,
    ) -> Result<(InterfaceSlot, Arc<T>), (Option<InterfaceSlot>, BridgeError)> {
        let slot = self.slot.ok_or((None, BridgeError::Absent))?;
        let generation = self
            .current_generation(slot)
            .map_err(|error| (Some(slot), error))?;
        let cached = self.cached.load();
        if let Some(snapshot) = cached.as_ref() {
            if snapshot.generation == generation {
                if let Some(provider) = snapshot.provider.upgrade() {
                    return Ok((slot, provider));
                }
            }
        }
        drop(cached);

        self.refresh_provider(slot)
            .map_err(|(_, error)| (Some(slot), error))
    }

    // 代际奇偶是新调用准入的快速判断；真正 provider 与类型仍在慢路径的同一状态快照中核验。
    fn current_generation(&self, slot: InterfaceSlot) -> Result<u32, BridgeError> {
        let generation = self
            .table
            .entry(slot)
            .ok_or(BridgeError::Absent)?
            .generation();
        if generation % 2 == 0 {
            Ok(generation)
        } else {
            Err(BridgeError::NotEnabled)
        }
    }

    fn refresh_provider(
        &self,
        slot: InterfaceSlot,
    ) -> Result<(InterfaceSlot, Arc<T>), (InterfaceSlot, BridgeError)> {
        let (generation, provider) = self
            .table
            .provider::<T>(slot)
            .map_err(|error| (slot, error))?;
        self.cached.store(Some(Arc::new(ProviderSnapshot {
            generation,
            provider: Arc::downgrade(&provider),
        })));
        Ok((slot, provider))
    }
}

/// 批量调用期间显式持有本次解析结果；该结构不再跟随表的代际变化。
pub struct BridgeGuard<T: ?Sized> {
    target: Arc<T>,
}

impl<T: ?Sized> Deref for BridgeGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.target
    }
}
