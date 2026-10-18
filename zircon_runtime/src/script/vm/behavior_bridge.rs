//! 行为树桥接把包限定的脚本节点引用解析为回调句柄；缓存随管理器代际刷新，使树节点调用已激活脚本而非旧实例。
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::script::{
    ScriptBehaviorBridge, ScriptBehaviorCallbackRef, ScriptHostError, ScriptHostValue,
};
use crate::core::runtime::ServiceHandle;

use super::{VmCallbackHandle, VmPluginManager};

/// Script-owned implementation exported by the ZrVM runtime plugin through
/// `script.behavior.v1`.
#[derive(Default)]
pub struct VmScriptBehaviorBridge {
    manager: Mutex<Option<Arc<BoundVmManager>>>,
}

struct BoundVmManager {
    handle: ServiceHandle<VmPluginManager>,
    callbacks: VmBehaviorCallbackCache,
}

#[derive(Default)]
struct VmBehaviorCallbackCache {
    callbacks: Mutex<BTreeMap<ScriptBehaviorCallbackRef, VmCallbackHandle>>,
}

impl VmScriptBehaviorBridge {
    pub fn new() -> Self {
        Self::default()
    }

    /// Binds an exact runtime registration without retaining the service or a call admission.
    pub fn bind_manager(&self, handle: ServiceHandle<VmPluginManager>) {
        let mut current = self.lock_manager();
        if current.as_ref().is_some_and(|bound| bound.handle == handle) {
            return;
        }
        // An in-flight invocation keeps its own cache when a new registration binds.
        *current = Some(Arc::new(BoundVmManager {
            handle,
            callbacks: VmBehaviorCallbackCache::default(),
        }));
    }

    fn manager(&self) -> Result<Arc<BoundVmManager>, ScriptHostError> {
        self.lock_manager().clone().ok_or_else(|| {
            ScriptHostError::new("script behavior bridge is not bound to an active VM manager")
        })
    }

    fn lock_manager(&self) -> MutexGuard<'_, Option<Arc<BoundVmManager>>> {
        self.manager
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl VmBehaviorCallbackCache {
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
        let binding = self.manager()?;
        let manager = binding
            .handle
            .enter()
            .map_err(|error| ScriptHostError::new(error.to_string()))?;
        let mut handle = binding.callbacks.resolve_callback(&manager, callback)?;
        let resolved_handle = handle;
        let result = manager
            .invoke_callback(&mut handle, arguments)
            .map_err(|error| ScriptHostError::new(error.to_string()));
        if result.is_ok() && handle != resolved_handle {
            binding.callbacks.cache_callback(callback, handle);
        }
        result
    }
}

#[cfg(test)]
#[path = "tests/behavior_bridge.rs"]
mod tests;
