//! 行为树桥接把包限定的脚本节点引用解析为回调句柄；缓存随管理器代际刷新，使树节点调用已激活脚本而非旧实例。
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::core::framework::script::{
    ScriptBehaviorBridge, ScriptBehaviorCallbackRef, ScriptHostError, ScriptHostValue,
};

use super::{VmCallbackHandle, VmPluginManager};

/// Script-owned implementation exported by the ZrVM runtime plugin through
/// `script.behavior.v1`.
#[derive(Default)]
pub struct VmScriptBehaviorBridge {
    manager: Mutex<Weak<VmPluginManager>>,
    callbacks: Mutex<BTreeMap<ScriptBehaviorCallbackRef, VmCallbackHandle>>,
}

impl VmScriptBehaviorBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds the active script manager without extending its lifecycle.
    pub fn bind_manager(&self, manager: &Arc<VmPluginManager>) {
        let next = Arc::downgrade(manager);
        let changed = {
            let mut current = self.lock_manager();
            let changed = !current.ptr_eq(&next);
            *current = next;
            changed
        };
        if changed {
            self.lock_callbacks().clear();
        }
    }

    fn cache_callback(
        &self,
        callback: &ScriptBehaviorCallbackRef,
        handle: VmCallbackHandle,
    ) -> bool {
        let mut callbacks = self.lock_callbacks();
        if let Some(cached) = callbacks.get_mut(callback) {
            *cached = handle;
            return false;
        }
        callbacks.insert(callback.clone(), handle);
        true
    }

    fn manager(&self) -> Result<Arc<VmPluginManager>, ScriptHostError> {
        self.lock_manager().upgrade().ok_or_else(|| {
            ScriptHostError::new("script behavior bridge is not bound to an active VM manager")
        })
    }

    fn resolve_callback(
        &self,
        manager: &VmPluginManager,
        callback: &ScriptBehaviorCallbackRef,
    ) -> Result<VmCallbackHandle, ScriptHostError> {
        let slot = manager
            .slot_for_package_name(callback.package_id())
            .map_err(|error| ScriptHostError::new(error.to_string()))?;
        let generation = manager
            .active_generation(slot)
            .map_err(|error| ScriptHostError::new(error.to_string()))?;
        if let Some(cached) = self.lock_callbacks().get(callback).copied() {
            if cached.slot == slot && cached.generation == generation {
                return Ok(cached);
            }
        }

        let mut matches = manager
            .registered_behavior_nodes()
            .into_iter()
            .filter(|registration| {
                registration.callback.slot == slot && registration.id == callback.node_id()
            });
        let Some(registration) = matches.next() else {
            return Err(ScriptHostError::new(format!(
                "script behavior callback `{}` is not registered",
                callback.stable_id()
            )));
        };
        if matches.next().is_some() {
            return Err(ScriptHostError::new(format!(
                "script behavior callback `{}` is ambiguous",
                callback.stable_id()
            )));
        }
        self.cache_callback(callback, registration.callback);
        Ok(registration.callback)
    }

    fn lock_manager(&self) -> MutexGuard<'_, Weak<VmPluginManager>> {
        self.manager
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_callbacks(
        &self,
    ) -> MutexGuard<'_, BTreeMap<ScriptBehaviorCallbackRef, VmCallbackHandle>> {
        self.callbacks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl ScriptBehaviorBridge for VmScriptBehaviorBridge {
    fn invoke(
        &self,
        callback: &ScriptBehaviorCallbackRef,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, ScriptHostError> {
        let manager = self.manager()?;
        let mut handle = self.resolve_callback(&manager, callback)?;
        let resolved_handle = handle;
        let result = manager
            .invoke_callback(&mut handle, arguments)
            .map_err(|error| ScriptHostError::new(error.to_string()));
        if result.is_ok() && handle != resolved_handle {
            self.cache_callback(callback, handle);
        }
        result
    }
}

#[cfg(test)]
#[path = "tests/behavior_bridge.rs"]
mod tests;
